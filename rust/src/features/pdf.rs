use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use lopdf::{dictionary, Document, Object, Stream, StringFormat};

use crate::state::{AppState, Screen};
use crate::ui::{Button as UiButton, Column as UiColumn, Text as UiText};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfState {
    pub source_uri: Option<String>,
    pub page_count: Option<u32>,
    pub selected_pages: Vec<u32>,
    pub last_output: Option<String>,
    pub last_error: Option<String>,
    pub signature_base64: Option<String>,
}

impl PdfState {
    pub const fn new() -> Self {
        Self {
            source_uri: None,
            page_count: None,
            selected_pages: Vec::new(),
            last_output: None,
            last_error: None,
            signature_base64: None,
        }
    }

    pub fn reset(&mut self) {
        self.source_uri = None;
        self.page_count = None;
        self.selected_pages.clear();
        self.last_output = None;
        self.last_error = None;
        self.signature_base64 = None;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PdfOperation {
    Extract,
    Delete,
    Merge,
}

pub fn handle_pdf_select(
    state: &mut AppState,
    fd: Option<i32>,
    uri: Option<&str>,
) -> Result<(), String> {
    state.pdf.page_count = None;
    state.pdf.selected_pages.clear();
    state.pdf.last_output = None;
    let raw_fd = fd.ok_or_else(|| "missing_fd".to_string())? as RawFd;
    let doc = load_document(raw_fd)?;
    let pages = doc.get_pages();
    state.pdf.page_count = Some(pages.len() as u32);
    state.pdf.source_uri = uri.map(|u| u.to_string());
    state.pdf.last_error = None;
    state.pdf.last_output = None;
    state.pdf.selected_pages = Vec::new();
    state.replace_current(Screen::PdfTools);
    Ok(())
}

pub fn handle_pdf_operation(
    state: &mut AppState,
    op: PdfOperation,
    primary_fd: Option<i32>,
    secondary_fd: Option<i32>,
    primary_uri: Option<&str>,
    _secondary_uri: Option<&str>,
    selected_pages: &[u32],
) -> Result<String, String> {
    let primary_fd = primary_fd.ok_or_else(|| "missing_fd".to_string())? as RawFd;
    let doc = load_document(primary_fd)?;
    let output_doc = match op {
        PdfOperation::Extract => keep_pages(doc, selected_pages)?,
        PdfOperation::Delete => delete_pages(doc, selected_pages)?,
        PdfOperation::Merge => {
            let secondary_fd = secondary_fd.ok_or_else(|| "missing_fd_secondary".to_string())? as RawFd;
            let secondary = load_document(secondary_fd)?;
            merge_documents(doc, secondary)?
        }
    };
    let page_count = output_doc.get_pages().len() as u32;
    let out_path = write_pdf(output_doc)?;
    state.pdf.last_output = Some(out_path.clone());
    state.pdf.source_uri = primary_uri
        .map(|u| u.to_string())
        .or_else(|| state.pdf.source_uri.clone());
    state.pdf.last_error = None;
    state.pdf.selected_pages = selected_pages.to_vec();
    state.pdf.page_count = Some(page_count);
    state.replace_current(Screen::PdfTools);
    Ok(out_path)
}

pub fn render_pdf_screen(state: &AppState) -> serde_json::Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("PDF tools").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Select a PDF, pick pages, then extract or delete them.")
                .size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Pick PDF", "pdf_select")
                .requires_file_picker(true)
                .content_description("Pick a PDF to edit"),
        )
        .unwrap(),
    ];

    if let Some(uri) = &state.pdf.source_uri {
        children.push(
            serde_json::to_value(
                UiText::new(&format!(
                    "Selected PDF: {}",
                    uri
                ))
                .size(12.0),
            )
            .unwrap(),
        );
    }

    if let (Some(count), Some(uri)) = (state.pdf.page_count, state.pdf.source_uri.as_ref()) {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Pages: {}", count)).size(12.0),
            )
            .unwrap(),
        );

        // Page picker rendered in Kotlin using PdfRenderer.
        children.push(
            serde_json::to_value(
                UiColumn::new(vec![json!({
                    "type": "PdfPagePicker",
                    "page_count": count,
                    "bind_key": "pdf_selected_pages",
                    "selected_pages": state.pdf.selected_pages,
                    "source_uri": uri,
                    "content_description": "PDF page picker"
                })])
                .content_description("pdf_page_picker_container"),
            )
            .unwrap(),
        );

        children.push(
            serde_json::to_value(
                UiButton::new("Extract selected pages", "pdf_extract")
                    .id("pdf_extract_btn"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Delete selected pages", "pdf_delete")
                    .id("pdf_delete_btn"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Merge with another PDF", "pdf_merge")
                    .id("pdf_merge_btn")
                    .requires_file_picker(true),
            )
            .unwrap(),
        );
    }

    // Title editing
    children.push(json!({
        "type": "TextInput",
        "bind_key": "pdf_title",
        "hint": "Document title (metadata)",
        "single_line": true
    }));
    children.push(
        serde_json::to_value(
            UiButton::new("Set PDF title", "pdf_set_title"),
        )
        .unwrap(),
    );

    if let Some(out) = &state.pdf.last_output {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Result saved to: {}", out)).size(12.0),
            )
            .unwrap(),
        );
    }

    // Signature section
    children.push(
        serde_json::to_value(UiText::new("Signature").size(16.0)).unwrap(),
    );
    children.push(
        serde_json::to_value(
            UiText::new("Draw or load a signature, then pick page/position to stamp it.")
                .size(12.0),
        )
        .unwrap(),
    );
    children.push(json!({
        "type": "SignaturePad",
        "bind_key": "signature_base64",
        "height_dp": 200,
        "content_description": "Signature drawing area"
    }));
    children.push(
        serde_json::to_value(
            UiButton::new("Load signature image", "pdf_signature_load")
                .requires_file_picker(true),
        )
        .unwrap(),
    );
    children.push(
        serde_json::to_value(
            UiButton::new("Clear signature", "pdf_signature_clear"),
        )
        .unwrap(),
    );
    if state.pdf.signature_base64.is_some() {
        children.push(
            serde_json::to_value(
                UiText::new("Signature ready").size(12.0),
            )
            .unwrap(),
        );
    }
    children.push(json!({
        "type": "TextInput",
        "bind_key": "pdf_signature_page",
        "hint": "Page number (1-based)",
        "single_line": true
    }));
    children.push(json!({
        "type": "TextInput",
        "bind_key": "pdf_signature_x",
        "hint": "X position (points)",
        "single_line": true
    }));
    children.push(json!({
        "type": "TextInput",
        "bind_key": "pdf_signature_y",
        "hint": "Y position (points)",
        "single_line": true
    }));
    children.push(json!({
        "type": "TextInput",
        "bind_key": "pdf_signature_width",
        "hint": "Width (points)",
        "single_line": true
    }));
    children.push(json!({
        "type": "TextInput",
        "bind_key": "pdf_signature_height",
        "hint": "Height (points)",
        "single_line": true
    }));
    children.push(
        serde_json::to_value(
            UiButton::new("Apply signature", "pdf_sign"),
        )
        .unwrap(),
    );

    if let Some(err) = &state.pdf.last_error {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Error: {}", err))
                    .size(12.0)
                    .content_description("pdf_error"),
            )
            .unwrap(),
        );
    }

    maybe_push_back(&mut children, state);

    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

fn load_document(fd: RawFd) -> Result<Document, String> {
    if fd < 0 {
        return Err("invalid_fd".into());
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("pdf_read_failed:{e}"))?;
    Document::load_mem(&buffer).map_err(|e| format!("pdf_parse_failed:{e}"))
}

fn keep_pages(mut doc: Document, selection: &[u32]) -> Result<Document, String> {
    let keep: BTreeSet<u32> = selection.iter().copied().collect();
    if keep.is_empty() {
        return Err("no_pages_selected".into());
    }
    let pages: BTreeMap<u32, lopdf::ObjectId> = doc.get_pages().into_iter().collect();
    if keep.iter().any(|p| !pages.contains_key(p)) {
        return Err("page_out_of_range".into());
    }
    let total = pages.len() as u32;
    let mut to_drop: Vec<u32> = (1..=total).filter(|p| !keep.contains(p)).collect();
    to_drop.sort_unstable();
    doc.delete_pages(&to_drop);
    Ok(doc)
}

fn delete_pages(mut doc: Document, selection: &[u32]) -> Result<Document, String> {
    if selection.is_empty() {
        return Err("no_pages_selected".into());
    }
    let pages: BTreeMap<u32, lopdf::ObjectId> = doc.get_pages().into_iter().collect();
    let mut uniq: Vec<u32> = selection.to_vec();
    uniq.sort_unstable();
    uniq.dedup();
    if uniq.iter().any(|p| !pages.contains_key(p)) {
        return Err("page_out_of_range".into());
    }
    if uniq.len() >= pages.len() {
        return Err("cannot_delete_all_pages".into());
    }
    doc.delete_pages(&uniq);
    Ok(doc)
}

fn merge_documents(mut primary: Document, mut secondary: Document) -> Result<Document, String> {
    let start_id = primary.max_id + 1;
    secondary.renumber_objects_with(start_id);

    let secondary_page_ids: Vec<lopdf::ObjectId> = secondary.page_iter().collect();

    for (id, obj) in secondary.objects.into_iter() {
        primary.objects.insert(id, obj);
    }
    if secondary.max_id > primary.max_id {
        primary.max_id = secondary.max_id;
    }

    let pages_root_id = primary
        .catalog()
        .map_err(|e| format!("pdf_merge_no_catalog:{e}"))?
        .get(b"Pages")
        .and_then(|o| o.as_reference())
        .map_err(|_| "pdf_merge_missing_pages_root")?;

    {
        let pages_dict = primary
            .get_object_mut(pages_root_id)
            .and_then(|o| o.as_dict_mut())
            .map_err(|_| "pdf_merge_missing_pages_dict")?;
        let kids = pages_dict
            .get_mut(b"Kids")
            .and_then(|o| o.as_array_mut())
            .map_err(|_| "pdf_merge_missing_kids")?;
        for page_id in &secondary_page_ids {
            kids.push(Object::Reference(*page_id));
        }
        let count = pages_dict
            .get(b"Count")
            .and_then(|c| c.as_i64())
            .unwrap_or(0);
        pages_dict.set("Count", count + secondary_page_ids.len() as i64);
    }

    for page_id in secondary_page_ids {
        if let Ok(page_dict) = primary.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
            page_dict.set("Parent", pages_root_id);
        }
    }

    Ok(primary)
}

fn write_pdf(mut doc: Document) -> Result<String, String> {
    let mut path = PathBuf::from(std::env::temp_dir());
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    path.push(format!("kistaverk_pdf_{millis}.pdf"));
    doc.save(&path)
        .map_err(|e| format!("pdf_save_failed:{e}"))?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "path_not_utf8".into())
}

fn maybe_push_back(children: &mut Vec<Value>, state: &AppState) {
    if state.nav_depth() > 1 {
        children.push(json!({
            "type": "Button",
            "text": "Back",
            "action": "back"
        }));
    }
}

pub fn handle_pdf_sign(
    state: &mut AppState,
    fd: RawFd,
    uri: Option<&str>,
    signature_base64: &str,
    page: Option<u32>,
    pos_x: f64,
    pos_y: f64,
    width: f64,
    height: f64,
    img_width_px: Option<f64>,
    img_height_px: Option<f64>,
    img_dpi: Option<f64>,
) -> Result<(), String> {
    let mut doc = load_document(fd)?;
    let target_page = page
        .or(state.pdf.page_count)
        .unwrap_or(1);
    let pages = doc.get_pages();
    let page_id = *pages
        .get(&target_page)
        .ok_or_else(|| "page_out_of_range".to_string())?;
    let (_, page_height) = page_dimensions(&doc, page_id)?;

    let sig_bytes = B64
        .decode(signature_base64.as_bytes())
        .map_err(|e| format!("signature_decode_failed:{e}"))?;
    let img = image::load_from_memory(&sig_bytes)
        .map_err(|e| format!("signature_image_invalid:{e}"))?
        .to_rgba8();
    let (img_w, img_h) = img.dimensions();
    let mut rgb = Vec::with_capacity((img_w * img_h * 3) as usize);
    let mut alpha = Vec::with_capacity((img_w * img_h) as usize);
    for pixel in img.pixels() {
        rgb.push(pixel[0]);
        rgb.push(pixel[1]);
        rgb.push(pixel[2]);
        alpha.push(pixel[3]);
    }

    let smask_stream = Stream::new(
        dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => img_w as i64,
            "Height" => img_h as i64,
            "ColorSpace" => "DeviceGray",
            "BitsPerComponent" => 8,
        },
        alpha,
    );
    let smask_id = doc.add_object(smask_stream);

    let image_stream = Stream::new(
        dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => img_w as i64,
            "Height" => img_h as i64,
            "ColorSpace" => "DeviceRGB",
            "BitsPerComponent" => 8,
            "SMask" => smask_id,
        },
        rgb,
    );
    let image_id = doc.add_object(image_stream);

    // Inject image into page resources
    let mut resources_obj = {
        let page_dict = doc
            .get_object_mut(page_id)
            .and_then(|o| o.as_dict_mut())
            .map_err(|_| "signature_page_missing_dict")?;
        page_dict
            .remove(b"Resources")
            .unwrap_or_else(|| Object::Dictionary(dictionary! {}))
    };

    match &mut resources_obj {
        Object::Reference(id) => {
            let res_dict = doc
                .get_object_mut(*id)
                .and_then(|o| o.as_dict_mut())
                .map_err(|_| "signature_resources_missing_dict")?;
            let xobj_dict = ensure_xobject_dict(res_dict)?;
            xobj_dict.set("ImSig", image_id);
        }
        Object::Dictionary(ref mut dict) => {
            let xobj_dict = ensure_xobject_dict(dict)?;
            xobj_dict.set("ImSig", image_id);
        }
        _ => return Err("signature_resources_invalid".into()),
    }

    {
        let page_dict = doc
            .get_object_mut(page_id)
            .and_then(|o| o.as_dict_mut())
            .map_err(|_| "signature_page_missing_dict")?;
        page_dict.set("Resources", resources_obj);
    }

    let px_to_pt = img_dpi.filter(|d| *d > 0.0).map(|dpi| 72.0 / dpi);
    let aspect = if img_w > 0 && img_h > 0 {
        Some(img_h as f64 / img_w as f64)
    } else {
        img_width_px.and_then(|w| img_height_px.and_then(|h| if w > 0.0 { Some(h / w) } else { None }))
    };
    let mut target_width = if width > 0.0 {
        width
    } else if let (Some(w_px), Some(scale)) = (img_width_px, px_to_pt) {
        (w_px * scale).max(1.0)
    } else {
        180.0
    };
    let mut target_height = if height > 0.0 {
        height
    } else if let Some(ratio) = aspect {
        (target_width * ratio).max(1.0)
    } else if let (Some(h_px), Some(scale)) = (img_height_px, px_to_pt) {
        (h_px * scale).max(1.0)
    } else {
        60.0
    };
    if target_width <= 0.0 {
        target_width = 1.0;
    }
    if target_height <= 0.0 {
        target_height = 1.0;
    }

    let pos_x_pt = if let Some(scale) = px_to_pt {
        pos_x * scale
    } else {
        pos_x
    };
    let pos_y_top = if let Some(scale) = px_to_pt {
        pos_y * scale
    } else {
        pos_y
    };
    let pdf_y = (page_height - pos_y_top - target_height).max(0.0);

    // Add content stream that draws the image
    let content = format!(
        "q {} 0 0 {} {} {} cm /ImSig Do Q",
        target_width, target_height, pos_x_pt, pdf_y
    );
    doc.add_page_contents(page_id, content.into_bytes())
        .map_err(|e| format!("signature_add_content_failed:{e}"))?;

    let page_count = doc.get_pages().len() as u32;
    let out_path = write_pdf(doc)?;
    state.pdf.last_output = Some(out_path.clone());
    state.pdf.source_uri = uri.map(|u| u.to_string()).or_else(|| state.pdf.source_uri.clone());
    state.pdf.last_error = None;
    state.pdf.signature_base64 = Some(signature_base64.to_string());
    state.pdf.page_count = Some(page_count);
    state.replace_current(Screen::PdfTools);
    Ok(())
}

fn page_dimensions(doc: &Document, page_id: lopdf::ObjectId) -> Result<(f64, f64), String> {
    let mut current = Some(page_id);
    while let Some(id) = current {
        let dict = doc
            .get_object(id)
            .and_then(|o| o.as_dict())
            .map_err(|_| "signature_page_missing_dict")?;
        if let Some((w, h)) = extract_media_box(doc, dict) {
            return Ok((w, h));
        }
        current = dict
            .get(b"Parent")
            .and_then(|p| p.as_reference())
            .ok();
    }
    // Fallback to a reasonable page size (A4-ish) if metadata is missing.
    Ok((595.0, 842.0))
}

fn extract_media_box(doc: &Document, dict: &lopdf::Dictionary) -> Option<(f64, f64)> {
    let raw = dict.get(b"MediaBox").ok()?;
    let resolved = match raw {
        Object::Reference(id) => doc.get_object(*id).ok()?,
        other => other,
    };
    let arr = resolved.as_array().ok()?;
    if arr.len() != 4 {
        return None;
    }
    let llx = obj_to_f64(&arr[0])?;
    let lly = obj_to_f64(&arr[1])?;
    let urx = obj_to_f64(&arr[2])?;
    let ury = obj_to_f64(&arr[3])?;
    Some((urx - llx, ury - lly))
}

fn obj_to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Integer(i) => Some(*i as f64),
        Object::Real(f) => Some((*f).into()),
        _ => None,
    }
}

fn ensure_xobject_dict<'a>(
    res_dict: &'a mut lopdf::Dictionary,
) -> Result<&'a mut lopdf::Dictionary, String> {
    let xobj_owned = res_dict
        .remove(b"XObject")
        .unwrap_or_else(|| Object::Dictionary(dictionary! {}));

    let sanitized = match xobj_owned {
        Object::Dictionary(dict) => Object::Dictionary(dict),
        Object::Reference(_) => Object::Dictionary(dictionary! {}),
        _ => return Err("signature_xobject_invalid".into()),
    };

    res_dict.set("XObject", sanitized);
    match res_dict.get_mut(b"XObject") {
        Ok(Object::Dictionary(ref mut dict)) => Ok(dict),
        _ => Err("signature_xobject_invalid".into()),
    }
}

pub fn handle_pdf_title(
    state: &mut AppState,
    fd: RawFd,
    uri: Option<&str>,
    title: Option<&str>,
) -> Result<(), String> {
    let title = title
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| "missing_title".to_string())?
        .trim()
        .to_string();

    let mut doc = load_document(fd)?;
    let info_id = match doc
        .trailer
        .get(b"Info")
        .and_then(|o| o.as_reference())
    {
        Ok(id) => id,
        Err(_) => {
            let new_info = doc.add_object(Object::Dictionary(dictionary! {}));
            doc.trailer.set("Info", new_info);
            new_info
        }
    };

    {
        let info_dict = doc
            .get_object_mut(info_id)
            .and_then(|o| o.as_dict_mut())
            .map_err(|_| "pdf_info_missing_dict".to_string())?;
        info_dict.set(
            "Title",
            Object::String(title.into_bytes(), StringFormat::Literal),
        );
    }

    let page_count = doc.get_pages().len() as u32;
    let out_path = write_pdf(doc)?;
    state.pdf.last_output = Some(out_path.clone());
    state.pdf.source_uri = uri.map(|u| u.to_string()).or_else(|| state.pdf.source_uri.clone());
    state.pdf.last_error = None;
    state.pdf.page_count = Some(page_count);
    state.replace_current(Screen::PdfTools);
    Ok(())
}
