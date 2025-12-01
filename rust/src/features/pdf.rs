use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::os::unix::io::{FromRawFd, RawFd};
use crate::features::storage::{output_dir_for, parse_file_uri_path};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use lopdf::{dictionary, Document, Object, Stream, StringFormat};
use memmap2::MmapOptions;
use time::{macros::format_description, OffsetDateTime};

fn append_page_content(
    doc: &mut Document,
    page_id: lopdf::ObjectId,
    content: Vec<u8>,
) -> Result<(), String> {
    let content_stream = Stream::new(dictionary! {}, content);
    let content_id = doc.add_object(content_stream);

    // Copy out existing Contents to avoid overlapping borrows.
    let existing_contents: Option<Object> = {
        let page_dict = doc
            .get_object(page_id)
            .and_then(|o| o.as_dict())
            .map_err(|_| "signature_page_missing_dict")?;
        page_dict.get(b"Contents").cloned().ok()
    };

    let mut new_contents: Vec<Object> = Vec::new();
    if let Some(existing) = existing_contents {
        match existing {
            Object::Array(arr) => new_contents.extend(arr),
            Object::Reference(id) => new_contents.push(Object::Reference(id)),
            Object::Stream(stream) => {
                let stream_id = doc.add_object(stream);
                new_contents.push(Object::Reference(stream_id));
            }
            other => new_contents.push(other),
        }
    }
    new_contents.push(Object::Reference(content_id));

    let page_dict = doc
        .get_object_mut(page_id)
        .and_then(|o| o.as_dict_mut())
        .map_err(|_| "signature_page_missing_dict")?;
    page_dict.set("Contents", Object::Array(new_contents));
    Ok(())
}

fn extract_pdf_title(doc: &Document) -> Option<String> {
    let info_id = doc.trailer.get(b"Info").ok()?.as_reference().ok()?;
    let info_dict = doc.get_object(info_id).ok()?.as_dict().ok()?;
    match info_dict.get(b"Title").ok()? {
        Object::String(bytes, _) => String::from_utf8(bytes.clone()).ok(),
        Object::Name(name) => Some(String::from_utf8_lossy(name).to_string()),
        _ => None,
    }
}

fn timestamp_suffix() -> String {
    const FMT: &[time::format_description::FormatItem<'_>] =
        format_description!("[year repr:last_two][month repr:numerical padding:zero][day padding:zero][hour repr:24 padding:zero][minute padding:zero]");
    OffsetDateTime::now_utc()
        .format(&FMT)
        .unwrap_or_else(|_| "0000000000".to_string())
}

fn output_filename(source_uri: Option<&str>) -> String {
    let suffix = timestamp_suffix();
    let base = source_uri
        .and_then(parse_file_uri_path)
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "kistaverk_pdf".to_string());
    let sanitized = base
        .trim_end_matches(".pdf")
        .trim_end_matches(".PDF")
        .to_string();
    format!("{sanitized}_modified_{suffix}.pdf")
}

use crate::state::{AppState, Screen};
use crate::ui::{
    Button as UiButton, Column as UiColumn, PdfPagePicker as UiPdfPagePicker, Text as UiText, maybe_push_back,
};

#[cfg(target_os = "android")]
fn log_pdf_debug(message: &str) {
    use android_log_sys::__android_log_write;
    use std::ffi::CString;

    const ANDROID_LOG_DEBUG: i32 = 3; // matches android/log.h debug priority

    if let (Ok(tag), Ok(text)) = (CString::new("kistaverk"), CString::new(message)) {
        unsafe {
            __android_log_write(ANDROID_LOG_DEBUG, tag.as_ptr(), text.as_ptr());
        }
    }
}

#[cfg(not(target_os = "android"))]
fn log_pdf_debug(message: &str) {
    eprintln!("[kistaverk][pdf][debug] {message}");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfState {
    pub source_uri: Option<String>,
    pub page_count: Option<u32>,
    pub selected_pages: Vec<u32>,
    pub last_output: Option<String>,
    pub last_error: Option<String>,
    pub preview_page: Option<u32>,
    pub current_title: Option<String>,
    pub signature_target_page: Option<u32>,
    pub signature_x_pct: Option<f64>,
    pub signature_y_pct: Option<f64>,
    pub signature_base64: Option<String>,
    pub signature_width_pt: Option<f64>,
    pub signature_height_pt: Option<f64>,
    pub signature_grid_selection: Option<(u32, f64, f64)>,
}

impl PdfState {
    pub const fn new() -> Self {
        Self {
            source_uri: None,
            page_count: None,
            selected_pages: Vec::new(),
            last_output: None,
            last_error: None,
            current_title: None,
            signature_target_page: None,
            signature_x_pct: None,
            signature_y_pct: None,
            signature_base64: None,
            signature_width_pt: None,
            signature_height_pt: None,
            signature_grid_selection: None,
            preview_page: None,
        }
    }

    pub fn reset(&mut self) {
        self.source_uri = None;
        self.page_count = None;
        self.selected_pages.clear();
        self.last_output = None;
        self.last_error = None;
        self.current_title = None;
        self.signature_target_page = None;
        self.signature_x_pct = None;
        self.signature_y_pct = None;
        self.signature_base64 = None;
        self.signature_width_pt = None;
        self.signature_height_pt = None;
        self.signature_grid_selection = None;
        self.preview_page = None;
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
    log_pdf_debug(&format!("pdf_select: fd={fd:?} uri={uri:?}"));
    state.pdf.page_count = None;
    state.pdf.selected_pages.clear();
    state.pdf.last_output = None;
    state.pdf.current_title = None;
    state.pdf.signature_target_page = None;
    state.pdf.signature_x_pct = None;
    state.pdf.signature_y_pct = None;
    let raw_fd = fd.ok_or_else(|| "missing_fd".to_string())? as RawFd;
    let doc = load_document(raw_fd)?;
    state.pdf.current_title = extract_pdf_title(&doc);
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
    log_pdf_debug(&format!(
        "pdf_operation: op={op:?} primary_fd={primary_fd:?} secondary_fd={secondary_fd:?} primary_uri={primary_uri:?} selection={selected_pages:?}"
    ));
    let primary_fd = primary_fd.ok_or_else(|| "missing_fd".to_string())? as RawFd;
    let doc = load_document(primary_fd)?;
    let output_doc = match op {
        PdfOperation::Extract => keep_pages(doc, selected_pages)?,
        PdfOperation::Delete => delete_pages(doc, selected_pages)?,
        PdfOperation::Merge => {
            let secondary_fd =
                secondary_fd.ok_or_else(|| "missing_fd_secondary".to_string())? as RawFd;
            let secondary = load_document(secondary_fd)?;
            merge_documents(doc, secondary)?
        }
    };
    let page_count = output_doc.get_pages().len() as u32;
    let new_title = extract_pdf_title(&output_doc);
    let out_path = write_pdf(output_doc, primary_uri)?;
    log_pdf_debug(&format!(
        "pdf_operation_complete: op={op:?} page_count={page_count} output_path={out_path}"
    ));
    state.pdf.last_output = Some(out_path.clone());
    state.pdf.source_uri = primary_uri
        .map(|u| u.to_string())
        .or_else(|| state.pdf.source_uri.clone());
    state.pdf.last_error = None;
    state.pdf.selected_pages = selected_pages.to_vec();
    state.pdf.page_count = Some(page_count);
    state.pdf.current_title = new_title;
    state.replace_current(Screen::PdfTools);
    Ok(out_path)
}

pub fn render_pdf_screen(state: &AppState) -> serde_json::Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("PDF tools").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Select a PDF, pick pages, then extract or delete them.").size(14.0),
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
            serde_json::to_value(UiText::new(&format!("Selected PDF: {}", uri)).size(12.0))
                .unwrap(),
        );
    }

    if let (Some(count), Some(uri)) = (state.pdf.page_count, state.pdf.source_uri.as_ref()) {
        children.push(
            serde_json::to_value(UiText::new(&format!("Pages: {}", count)).size(12.0)).unwrap(),
        );

        // Page picker rendered in Kotlin using PdfRenderer.
        children.push(
            serde_json::to_value(
                UiColumn::new(vec![serde_json::to_value(
                    UiPdfPagePicker::new(count, "pdf_selected_pages", uri)
                        .selected_pages(&state.pdf.selected_pages)
                        .content_description("PDF page picker"),
                )
                .unwrap()])
                .content_description("pdf_page_picker_container"),
            )
            .unwrap(),
        );
        let selected_len = state.pdf.selected_pages.len();
        children.push(
            serde_json::to_value(
                UiText::new(&format!(
                    "Selected pages: {} / {}",
                    selected_len, count
                ))
                .size(12.0)
                .content_description("pdf_selected_summary"),
            )
            .unwrap(),
        );

        children.push(
            serde_json::to_value(
                UiButton::new("Extract selected pages", "pdf_extract").id("pdf_extract_btn"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Delete selected pages", "pdf_delete").id("pdf_delete_btn"),
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
        children.push(
            serde_json::to_value(UiButton::new("Open viewer", "pdf_preview_screen").id("pdf_preview_btn"))
                .unwrap(),
        );
    }

    // Title editing
    if let Some(title) = &state.pdf.current_title {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Current title: {}", title))
                    .size(12.0)
                    .content_description("pdf_current_title"),
            )
            .unwrap(),
        );
    }
    children.push(
        serde_json::to_value(
            crate::ui::TextInput::new("pdf_title")
                .hint("Document title (metadata)")
                .text(state.pdf.current_title.as_deref().unwrap_or_default())
                .single_line(true),
        )
        .unwrap(),
    );
    children.push(serde_json::to_value(UiButton::new("Set PDF title", "pdf_set_title")).unwrap());
    if let (Some(count), Some(uri)) = (state.pdf.page_count, state.pdf.source_uri.as_ref()) {
        children.push(json!({
            "type": "PdfSignPlacement",
            "source_uri": uri,
            "page_count": count,
            "selected_page": state.pdf.signature_target_page.unwrap_or(1),
            "bind_key_page": "pdf_signature_page",
            "bind_key_x_pct": "pdf_signature_x_pct",
            "bind_key_y_pct": "pdf_signature_y_pct",
            "selected_x_pct": state.pdf.signature_x_pct,
            "selected_y_pct": state.pdf.signature_y_pct,
            "content_description": "Signature placement picker"
        }));

        // Compact thumbnail preview with overlay marker; mirrors placement state
        children.push(json!({
            "type": "PdfSignPreview",
            "source_uri": uri,
            "page_count": count,
            "selected_page": state.pdf.signature_target_page.unwrap_or(1),
            "bind_key_page": "pdf_signature_page",
            "bind_key_x_pct": "pdf_signature_x_pct",
            "bind_key_y_pct": "pdf_signature_y_pct",
            "selected_x_pct": state.pdf.signature_x_pct,
            "selected_y_pct": state.pdf.signature_y_pct,
            "content_description": "Signature preview thumbnail"
        }));

        // Quick placement grid (3x3 preset normalized coords)
        let grid_positions: [(&str, f64, f64); 9] = [
            ("↖ Top-left", 0.1, 0.1),
            ("↑ Top", 0.5, 0.1),
            ("↗ Top-right", 0.9, 0.1),
            ("← Left", 0.1, 0.5),
            ("· Center", 0.5, 0.5),
            ("→ Right", 0.9, 0.5),
            ("↙ Bottom-left", 0.1, 0.9),
            ("↓ Bottom", 0.5, 0.9),
            ("↘ Bottom-right", 0.9, 0.9),
        ];
        let mut grid_children: Vec<Value> = Vec::new();
        for (label, x, y) in grid_positions {
            grid_children.push(json!({
                "type": "Button",
                "text": label,
                "action": "pdf_sign_grid",
                "id": format!("pdf_sign_grid_{}_{}", (x*100.0) as u32, (y*100.0) as u32),
                "content_description": "pdf_sign_grid_button",
                "requires_file_picker": false,
                "payload": {
                    "pdf_signature_page": state.pdf.signature_target_page.unwrap_or(1),
                    "pdf_signature_x_pct": x,
                    "pdf_signature_y_pct": y
                }
            }));
        }
        children.push(
            serde_json::to_value(
                UiColumn::new(vec![
                    serde_json::to_value(
                        UiText::new("Quick placement").size(13.0).content_description("pdf_sign_grid_label"),
                    )
                    .unwrap(),
                    json!({
                        "type": "Grid",
                        "columns": 3,
                        "padding": 4,
                        "children": grid_children
                    })
                ])
                .padding(8),
            )
            .unwrap(),
        );
    }

    if let Some(out) = &state.pdf.last_output {
        children.push(
            serde_json::to_value(UiText::new(&format!("Result saved to: {}", out)).size(12.0))
                .unwrap(),
        );
        children.push(
            serde_json::to_value(UiButton::new("Save as…", "pdf_save_as").id("pdf_save_as_btn"))
                .unwrap(),
        );
    }

    // Signature section
    children.push(serde_json::to_value(UiText::new("Signature").size(16.0)).unwrap());
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
            UiButton::new("Load signature image", "pdf_signature_load").requires_file_picker(true),
        )
        .unwrap(),
    );
    children.push(
        serde_json::to_value(UiButton::new("Clear signature", "pdf_signature_clear")).unwrap(),
    );
    if state.pdf.signature_base64.is_some() {
        children.push(serde_json::to_value(UiText::new("Signature ready").size(12.0)).unwrap());
    }
    children.push(
        serde_json::to_value(
            crate::ui::TextInput::new("pdf_signature_page")
                .hint("Page number (1-based)")
                .single_line(true),
        )
        .unwrap(),
    );
    children.push(
        serde_json::to_value(
            crate::ui::TextInput::new("pdf_signature_x")
                .hint("X position (points)")
                .single_line(true),
        )
        .unwrap(),
    );
    children.push(
        serde_json::to_value(
            crate::ui::TextInput::new("pdf_signature_y")
                .hint("Y position (points)")
                .single_line(true),
        )
        .unwrap(),
    );
    children.push(
        serde_json::to_value(
            crate::ui::TextInput::new("pdf_signature_width")
                .hint("Width (points)")
                .text(
                    &state
                        .pdf
                        .signature_width_pt
                        .map(|w| format!("{:.1}", w))
                        .unwrap_or_default(),
                )
                .single_line(true),
        )
        .unwrap(),
    );
    children.push(
        serde_json::to_value(
            crate::ui::TextInput::new("pdf_signature_height")
                .hint("Height (points)")
                .text(
                    &state
                        .pdf
                        .signature_height_pt
                        .map(|h| format!("{:.1}", h))
                        .unwrap_or_default(),
                )
                .single_line(true),
        )
        .unwrap(),
    );
    children.push(serde_json::to_value(UiButton::new("Apply signature", "pdf_sign")).unwrap());

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

pub fn render_pdf_preview_screen(state: &AppState) -> serde_json::Value {
    let mut children = vec![serde_json::to_value(UiText::new("PDF viewer").size(20.0)).unwrap()];
    let pdf = &state.pdf;

    match (&pdf.source_uri, pdf.page_count) {
        (Some(uri), Some(count)) if count > 0 => {
            if let Some(page) = pdf.preview_page {
                children.push(json!({
                    "type": "PdfSinglePage",
                    "source_uri": uri,
                    "page": page
                }));
                if page > 1 {
                    children.push(json!({
                        "type": "Button",
                        "text": "Prev",
                        "action": "pdf_page_open",
                        "payload": { "page": (page - 1) }
                    }));
                }
                if page < count {
                    children.push(json!({
                        "type": "Button",
                        "text": "Next",
                        "action": "pdf_page_open",
                        "payload": { "page": (page + 1) }
                    }));
                }
                children.push(json!({
                    "type": "Button",
                    "text": "Grid",
                    "action": "pdf_page_close"
                }));
                children.push(json!({
                    "type": "Button",
                    "text": "Back",
                    "action": "back"
                }));
            } else {
                children.push(json!({
                    "type": "PdfPreviewGrid",
                    "source_uri": uri,
                    "page_count": count,
                    "action": "pdf_page_open"
                }));
                children.push(json!({
                    "type": "Button",
                    "text": "Back",
                    "action": "back"
                }));
            }
        }
        _ => {
            children.push(
                serde_json::to_value(
                    UiText::new("No PDF selected. Open a PDF from the PDF pages tool first.")
                        .size(14.0),
                )
                .unwrap(),
            );
            children.push(json!({
                "type": "Button",
                "text": "Back",
                "action": "back"
            }));
        }
    }

    serde_json::to_value(UiColumn::new(children).padding(16)).unwrap()
}



fn load_document(fd: RawFd) -> Result<Document, String> {
    if fd < 0 {
        return Err("invalid_fd".into());
    }
    log_pdf_debug(&format!("load_document: fd={fd}"));
    let file = unsafe { File::from_raw_fd(fd) };
    let file_len = file
        .metadata()
        .map_err(|e| {
            log_pdf_debug(&format!("pdf_read_failed_metadata: fd={fd} err={e}"));
            format!("pdf_read_failed:{e}")
        })?
        .len();
    if file_len == 0 {
        return Err("pdf_read_failed:empty_file".into());
    }
    let mmap = unsafe {
        MmapOptions::new()
            .len(file_len as usize)
            .map(&file)
            .map_err(|e| {
                log_pdf_debug(&format!("pdf_read_failed_mmap: fd={fd} err={e}"));
                format!("pdf_read_failed:{e}")
            })?
    };
    Document::load_mem(&mmap).map_err(|e| {
        log_pdf_debug(&format!("pdf_parse_failed: fd={fd} err={e}"));
        format!("pdf_parse_failed:{e}")
    })
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
        if let Ok(page_dict) = primary
            .get_object_mut(page_id)
            .and_then(|o| o.as_dict_mut())
        {
            page_dict.set("Parent", pages_root_id);
        }
    }

    Ok(primary)
}

fn write_pdf(mut doc: Document, source_uri: Option<&str>) -> Result<String, String> {
    let mut path = output_dir_for(source_uri);
    let filename = output_filename(source_uri);
    log_pdf_debug(&format!(
        "write_pdf: using_dir={:?} filename={}",
        path, filename
    ));
    path.push(filename);
    doc.save(&path).map_err(|e| {
        log_pdf_debug(&format!("pdf_save_failed: path={:?} err={e}", path));
        format!("pdf_save_failed:{e}")
    })?;
    let path_str: String = path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| String::from("path_not_utf8"))?;
    log_pdf_debug(&format!("write_pdf_complete: path={path_str}"));
    Ok(path_str)
}



pub fn handle_pdf_sign(
    state: &mut AppState,
    fd: RawFd,
    uri: Option<&str>,
    signature_base64: &str,
    page: Option<u32>,
    page_x_pct: Option<f64>,
    page_y_pct: Option<f64>,
    pos_x: f64,
    pos_y: f64,
    width: f64,
    height: f64,
    img_width_px: Option<f64>,
    img_height_px: Option<f64>,
    img_dpi: Option<f64>,
) -> Result<(), String> {
    log_pdf_debug(&format!(
        "pdf_sign: fd={fd} uri={uri:?} page={page:?} pos=({pos_x},{pos_y}) pct=({page_x_pct:?},{page_y_pct:?}) size=({width}x{height}) img_px=({img_width_px:?}x{img_height_px:?}) dpi={img_dpi:?}"
    ));
    // TODO: add unit tests for normalized coord mapping (corners/edges) and ensure clamping to page bounds for taps outside the overlay.
    let mut doc = load_document(fd)?;
    let target_page = page
        .or(state.pdf.signature_target_page)
        .or(state.pdf.page_count)
        .unwrap_or(1);
    let pages = doc.get_pages();
    let page_id = *pages
        .get(&target_page)
        .ok_or_else(|| "page_out_of_range".to_string())?;
    let (page_width, page_height) = page_dimensions(&doc, page_id)?;

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
        img_width_px
            .and_then(|w| img_height_px.and_then(|h| if w > 0.0 { Some(h / w) } else { None }))
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

    let norm_x = page_x_pct
        .filter(|v| v.is_finite())
        .map(|v| v.clamp(0.0, 1.0));
    let norm_y = page_y_pct
        .filter(|v| v.is_finite())
        .map(|v| v.clamp(0.0, 1.0));
    let pos_x_pt = if let Some(nx) = norm_x {
        nx * page_width
    } else if let Some(scale) = px_to_pt {
        pos_x * scale
    } else {
        pos_x
    };
    let pos_y_top = if let Some(ny) = norm_y {
        ny * page_height
    } else if let Some(scale) = px_to_pt {
        pos_y * scale
    } else {
        pos_y
    };
    let pdf_y = (page_height - pos_y_top - target_height).max(0.0);
    let final_norm_x = norm_x.or_else(|| {
        if page_width > 0.0 {
            Some((pos_x_pt / page_width).clamp(0.0, 1.0))
        } else {
            None
        }
    });
    let final_norm_y = norm_y.or_else(|| {
        if page_height > 0.0 {
            Some((pos_y_top / page_height).clamp(0.0, 1.0))
        } else {
            None
        }
    });

    // Add content stream that draws the image
    let content = format!(
        "q {} 0 0 {} {} {} cm /ImSig Do Q",
        target_width, target_height, pos_x_pt, pdf_y
    );
    append_page_content(&mut doc, page_id, content.into_bytes())
        .map_err(|e| format!("signature_add_content_failed:{e}"))?;

    let page_count = doc.get_pages().len() as u32;
    let new_title = extract_pdf_title(&doc);
    let out_path = write_pdf(doc, uri)?;
    log_pdf_debug(&format!(
        "pdf_sign_complete: output_path={out_path} page_count={page_count} target_page={target_page}"
    ));
    state.pdf.last_output = Some(out_path.clone());
    state.pdf.source_uri = uri
        .map(|u| u.to_string())
        .or_else(|| state.pdf.source_uri.clone());
    state.pdf.last_error = None;
    state.pdf.current_title = new_title;
    state.pdf.signature_target_page = Some(target_page);
    state.pdf.signature_x_pct = final_norm_x;
    state.pdf.signature_y_pct = final_norm_y;
    state.pdf.signature_base64 = Some(signature_base64.to_string());
    state.pdf.signature_width_pt = Some(target_width);
    state.pdf.signature_height_pt = Some(target_height);
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
        current = dict.get(b"Parent").and_then(|p| p.as_reference()).ok();
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
    log_pdf_debug(&format!(
        "pdf_set_title: fd={fd} uri={uri:?} title_present={}",
        title.map(|t| !t.trim().is_empty()).unwrap_or(false)
    ));
    let title = title
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| "missing_title".to_string())?
        .trim()
        .to_string();

    let mut doc = load_document(fd)?;
    let info_id = match doc.trailer.get(b"Info").and_then(|o| o.as_reference()) {
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
            Object::String(title.clone().into_bytes(), StringFormat::Literal),
        );
    }

    let page_count = doc.get_pages().len() as u32;
    let out_path = write_pdf(doc, uri)?;
    log_pdf_debug(&format!(
        "pdf_set_title_complete: output_path={out_path} page_count={page_count}"
    ));
    state.pdf.last_output = Some(out_path.clone());
    state.pdf.source_uri = uri
        .map(|u| u.to_string())
        .or_else(|| state.pdf.source_uri.clone());
    state.pdf.last_error = None;
    state.pdf.current_title = Some(title);
    state.pdf.page_count = Some(page_count);
    state.replace_current(Screen::PdfTools);
    Ok(())
}
