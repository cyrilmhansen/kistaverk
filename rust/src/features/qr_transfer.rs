use crate::state::AppState;
use crate::ui::{Button as UiButton, Column as UiColumn, Text as UiText, TextInput as UiTextInput};
use base64::Engine;
use image::{codecs::png::PngEncoder, ColorType, ImageBuffer, ImageEncoder, Luma};
use qrcode::{Color, QrCode};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::io::{FromRawFd, RawFd};
use crate::features::storage::preferred_temp_dir;

const CHUNK_BYTES: usize = 512;
const HEADER_PREFIX: &str = "QRTX";

/// Decode a QR code from a luminance (Y) plane.
///
/// Parameters are shaped for camera analyzers: width/height of the image, row stride in bytes,
/// clockwise rotation degrees (0/90/180/270), and the Y plane buffer.
/// This is intentionally separated so a future rxing-based decoder can drop in without touching JNI.
use rxing::common::HybridBinarizer;
use rxing::{BarcodeFormat, BinaryBitmap, DecodeHintValue, DecodeHints, Luma8LuminanceSource, MultiFormatReader, Reader};
use rxing::Exceptions;
use std::collections::HashSet;

pub fn decode_qr_frame_luma(
    luma_data: &[u8],
    width: u32,
    height: u32,
    _row_stride: u32, // Stride is often width for simple luma planes, but might differ. rxing expects flat data.
    _rotation_deg: u16, // Not directly used by rxing for luma, rotation must be applied by caller or handled in image preparation
) -> Result<Option<String>, String> {
    let hints = DecodeHints::default()
        .with(DecodeHintValue::TryHarder(true))
        .with(DecodeHintValue::PossibleFormats(HashSet::from([BarcodeFormat::QR_CODE])));

    let luma_source = Luma8LuminanceSource::new(luma_data.to_vec(), width, height);
    let binarizer = HybridBinarizer::new(luma_source);
    let mut binary_bitmap = BinaryBitmap::new(binarizer);

    let mut reader = MultiFormatReader::default();
    
    // hints are passed directly in the decode method
    reader.decode_with_hints(&mut binary_bitmap, &hints)
        .map(|result| Some(result.getText().to_string()))
        .map_err(|e| {
            // Treat NotFound as not an error, just no QR code found.
            if let Exceptions::NotFoundException(_) = e {
                return "qr_not_found".to_string(); // Return a specific string for 'not found'
            }
            format!("qr_decode_failed:{:?}", e)
        })
        .or_else(|e_str| {
            // If the error is 'qr_not_found', return Ok(None) instead of an Err.
            if e_str == "qr_not_found" {
                Ok(None)
            } else {
                Err(e_str)
            }
        })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QrSlideshowState {
    pub source_path: Option<String>,
    pub chunks: Vec<String>,
    pub current_index: usize,
    pub is_playing: bool,
    pub interval_ms: u64,
    pub error: Option<String>,
    pub current_qr_base64: Option<String>,
}

impl QrSlideshowState {
    pub const fn new() -> Self {
        Self {
            source_path: None,
            chunks: Vec::new(),
            current_index: 0,
            is_playing: false,
            interval_ms: 200,
            error: None,
            current_qr_base64: None,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

fn qr_png_base64(data: &str) -> Result<String, String> {
    let code = QrCode::new(data.as_bytes()).map_err(|e| format!("qr_encode_failed:{e}"))?;
    let base_size = code.width() as u32;
    let colors = code.to_colors();
    let mut base = ImageBuffer::<Luma<u8>, Vec<u8>>::new(base_size, base_size);
    for y in 0..base_size {
        for x in 0..base_size {
            let idx = (y * base_size + x) as usize;
            let dark = matches!(colors.get(idx), Some(Color::Dark));
            base.put_pixel(x, y, if dark { Luma([0u8]) } else { Luma([255u8]) });
        }
    }

    // Scale up for visibility
    let scale = (256 / base_size.max(1)).max(4);
    let scaled_w = base_size * scale;
    let scaled_h = base_size * scale;
    let mut scaled = ImageBuffer::<Luma<u8>, Vec<u8>>::new(scaled_w, scaled_h);
    for y in 0..scaled_h {
        for x in 0..scaled_w {
            let src_x = x / scale;
            let src_y = y / scale;
            let pix = base.get_pixel(src_x, src_y);
            scaled.put_pixel(x, y, *pix);
        }
    }

    let mut buf = Vec::new();
    let encoder = PngEncoder::new(&mut buf);
    encoder
        .write_image(&scaled, scaled.width(), scaled.height(), ColorType::L8)
        .map_err(|e| format!("qr_png_failed:{e}"))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(buf))
}

pub fn load_slideshow_from_fd(
    state: &mut AppState,
    fd: RawFd,
    path_hint: Option<&str>,
) -> Result<(), String> {
    if fd < 0 {
        return Err("invalid_fd".into());
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| format!("qr_read_failed:{e}"))?;
    // reset position for potential reuse
    let _ = file.seek(SeekFrom::Start(0));
    populate_slideshow_state(state, buf, path_hint)
}

pub fn load_slideshow_from_path(state: &mut AppState, path: &str) -> Result<(), String> {
    let mut file = File::open(path).map_err(|e| format!("qr_open_failed:{e}"))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| format!("qr_read_failed:{e}"))?;
    populate_slideshow_state(state, buf, Some(path))
}

fn populate_slideshow_state(
    state: &mut AppState,
    bytes: Vec<u8>,
    path_hint: Option<&str>,
) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("qr_empty_file".into());
    }
    let chunks = chunk_bytes(&bytes);
    if chunks.is_empty() {
        return Err("qr_no_chunks".into());
    }
    state.qr_slideshow.chunks = chunks;
    state.qr_slideshow.current_index = 0;
    state.qr_slideshow.is_playing = false;
    state.qr_slideshow.source_path = path_hint.map(|p| p.to_string());
    state.qr_slideshow.error = None;
    refresh_current_qr(state)?;
    Ok(())
}

pub fn refresh_current_qr(state: &mut AppState) -> Result<(), String> {
    if state.qr_slideshow.chunks.is_empty() {
        state.qr_slideshow.current_qr_base64 = None;
        return Ok(());
    }
    let idx = state
        .qr_slideshow
        .current_index
        .min(state.qr_slideshow.chunks.len().saturating_sub(1));
    let payload = &state.qr_slideshow.chunks[idx];
    let image_b64 = qr_png_base64(payload)?;
    state.qr_slideshow.current_index = idx;
    state.qr_slideshow.current_qr_base64 = Some(image_b64);
    Ok(())
}

pub fn advance_frame(state: &mut AppState, step: isize) -> Result<(), String> {
    if state.qr_slideshow.chunks.is_empty() {
        return Ok(());
    }
    let len = state.qr_slideshow.chunks.len() as isize;
    let current = state.qr_slideshow.current_index as isize;
    let mut next = (current + step) % len;
    if next < 0 {
        next += len;
    }
    state.qr_slideshow.current_index = next as usize;
    refresh_current_qr(state)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QrReceiveState {
    pub chunks: Vec<Option<Vec<u8>>>,
    pub total_chunks: Option<usize>,
    pub last_scanned: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
    pub result_path: Option<String>,
}

impl QrReceiveState {
    pub const fn new() -> Self {
        Self {
            chunks: Vec::new(),
            total_chunks: None,
            last_scanned: None,
            status: None,
            error: None,
            result_path: None,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

pub(crate) fn chunk_bytes(bytes: &[u8]) -> Vec<String> {
    if bytes.is_empty() {
        return Vec::new();
    }
    let mut chunks = Vec::new();
    for (i, chunk) in bytes.chunks(CHUNK_BYTES).enumerate() {
        let total_chunks = (bytes.len() + CHUNK_BYTES - 1) / CHUNK_BYTES;
        let encoded = base64::engine::general_purpose::STANDARD.encode(chunk);
        let payload = format!("{}|{}/{}|{}", HEADER_PREFIX, i + 1, total_chunks, encoded);
        chunks.push(payload);
    }
    chunks
}

fn parse_qr_payload(payload: &str) -> Result<(usize, usize, Vec<u8>), String> {
    let mut parts = payload.splitn(3, '|');
    let prefix = parts.next().ok_or_else(|| "qr_invalid_header".to_string())?;
    if prefix != HEADER_PREFIX {
        return Err("qr_invalid_prefix".into());
    }
    let order = parts
        .next()
        .ok_or_else(|| "qr_missing_order".to_string())?;
    let mut order_split = order.split('/');
    let index = order_split
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .ok_or_else(|| "qr_invalid_index".to_string())?;
    let total = order_split
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .ok_or_else(|| "qr_invalid_total".to_string())?;
    let data_b64 = parts
        .next()
        .ok_or_else(|| "qr_missing_payload".to_string())?;
    let data =
        base64::engine::general_purpose::STANDARD.decode(data_b64.as_bytes()).map_err(|_| "qr_b64_decode_failed".to_string())?;
    Ok((index, total, data))
}

pub fn handle_receive_scan(state: &mut AppState, payload: &str) -> Result<(), String> {
    let (index, total, data) = parse_qr_payload(payload)?;
    match state.qr_receive.total_chunks {
        Some(existing_total) if existing_total != total => return Err("qr_total_mismatch".into()),
        None => {
            state.qr_receive.total_chunks = Some(total);
            state.qr_receive.chunks.clear();
            state.qr_receive.chunks.resize(total, None);
        }
        _ => {}
    }
    if index == 0 || index > total {
        return Err("qr_index_out_of_bounds".into());
    }
    if state.qr_receive.chunks.len() < total {
        state.qr_receive.chunks.resize(total, None);
    }
    state.qr_receive.chunks[index - 1] = Some(data);
    state.qr_receive.last_scanned = Some(payload.to_string());
    state.qr_receive.status = Some(format!(
        "Received {}/{}",
        state.qr_receive.chunks.iter().filter(|c| c.is_some()).count(),
        total,
    ));
    state.qr_receive.error = None;
    if state
        .qr_receive
        .chunks
        .iter()
        .filter(|c| c.is_some())
        .count()
        == total
    {
        match finalize_receive(state) {
            Ok(bytes) => {
                state.qr_receive.status = Some(format!("Complete ({} bytes)", bytes.len()));
                state.qr_receive.error = None;
            }
            Err(e) => state.qr_receive.error = Some(e),
        }
    }
    Ok(())
}

pub fn finalize_receive(state: &mut AppState) -> Result<Vec<u8>, String> {
    let total = state
        .qr_receive
        .total_chunks
        .ok_or_else(|| "qr_no_total".to_string())?;
    let mut data = Vec::new();
    if state.qr_receive.chunks.len() < total {
        return Err("qr_incomplete".into());
    }
    for (idx, chunk_opt) in state.qr_receive.chunks.iter().enumerate().take(total) {
        let chunk = chunk_opt
            .as_ref()
            .ok_or_else(|| format!("qr_missing_chunk:{}", idx + 1))?;
        data.extend_from_slice(chunk);
    }
    Ok(data)
}

pub fn save_received_file(state: &mut AppState) -> Result<String, String> {
    let bytes = finalize_receive(state)?;
    let mut path = preferred_temp_dir();
    path.push(format!("qr_receive_{}.bin", time::OffsetDateTime::now_utc().unix_timestamp()));
    std::fs::write(&path, &bytes).map_err(|e| format!("qr_save_failed:{e}"))?;
    let path_str = path
        .to_str()
        .ok_or_else(|| "qr_path_utf8".to_string())?
        .to_string();
    state.qr_receive.result_path = Some(path_str.clone());
    Ok(path_str)
}

pub fn render_qr_slideshow_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("QR Transfer (Sender)").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Pick a file to broadcast via a sequence of QR codes.")
                .size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Pick file", "qr_slideshow_pick")
                .requires_file_picker(true)
                .id("qr_slideshow_pick"),
        )
        .unwrap(),
    ];

    if let Some(path) = &state.qr_slideshow.source_path {
        children.push(
            serde_json::to_value(UiText::new(&format!("Source: {path}")).size(12.0)).unwrap(),
        );
    }

    if let Some(err) = &state.qr_slideshow.error {
        children.push(
            serde_json::to_value(UiText::new(&format!("Error: {err}")).size(12.0)).unwrap(),
        );
    }

    if !state.qr_slideshow.chunks.is_empty() {
        let total = state.qr_slideshow.chunks.len();
        let idx = state.qr_slideshow.current_index + 1;
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Frame {idx}/{total}")).size(14.0),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiText::new(&format!(
                    "Interval: {} ms (playing: {})",
                    state.qr_slideshow.interval_ms,
                    if state.qr_slideshow.is_playing { "yes" } else { "no" }
                ))
                .size(12.0),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new(
                    if state.qr_slideshow.is_playing { "Pause" } else { "Play" },
                    "qr_slideshow_play",
                )
                .id("qr_slideshow_play"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(UiButton::new("Prev", "qr_slideshow_prev").id("qr_prev")).unwrap(),
        );
        children.push(
            serde_json::to_value(UiButton::new("Next", "qr_slideshow_next").id("qr_next")).unwrap(),
        );
        for speed in [100u64, 200, 400, 800] {
            children.push(json!({
                "type": "Button",
                "text": format!("{} ms", speed),
                "action": "qr_slideshow_set_speed",
                "payload": { "interval_ms": speed },
                "id": format!("qr_speed_{speed}")
            }));
        }
        if let Some(img) = &state.qr_slideshow.current_qr_base64 {
            children.push(
                serde_json::to_value(
                    crate::ui::ImageBase64::new(img).content_description("QR frame"),
                )
                .unwrap(),
            );
        }
    }

    if state.nav_depth() > 1 {
        children.push(serde_json::to_value(UiButton::new("Back", "back")).unwrap());
    }

    let mut root = json!(UiColumn::new(children).padding(20));
    if let Some(obj) = root.as_object_mut() {
        if state.qr_slideshow.is_playing && !state.qr_slideshow.chunks.is_empty() {
            obj.insert(
                "auto_refresh_ms".into(),
                serde_json::Value::Number(state.qr_slideshow.interval_ms.into()),
            );
            obj.insert(
                "auto_refresh_action".into(),
                serde_json::Value::String("qr_slideshow_tick".into()),
            );
        }
    }
    root
}

pub fn render_qr_receive_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("QR Transfer (Receiver)").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Paste scanned frames or use the camera. Grant permission and keep the QRs in view; the preview runs behind this panel.")
                .size(14.0)
                .content_description("qr_receive_subtitle"),
        )
        .unwrap(),
        serde_json::to_value(
            UiTextInput::new("qr_scan_input")
                .hint("QRTX|1/3|...")
                .action_on_submit("qr_receive_scan"),
        )
        .unwrap(),
        serde_json::to_value(UiButton::new("Submit chunk", "qr_receive_scan").id("qr_receive_scan_btn")).unwrap(),
        serde_json::to_value(UiButton::new("Paste from clipboard", "qr_receive_paste").id("qr_receive_paste")).unwrap(),
        // Re-rendering the screen is enough to (re)start the camera preview via MainActivity.
        serde_json::to_value(UiButton::new("Resume camera", "qr_receive_screen").id("qr_receive_camera_resume")).unwrap(),
    ];

    if let Some(status) = &state.qr_receive.status {
        children.push(serde_json::to_value(UiText::new(status).size(12.0)).unwrap());
    }
    if let Some(err) = &state.qr_receive.error {
        children.push(serde_json::to_value(UiText::new(&format!("Error: {err}")).size(12.0)).unwrap());
    }
    if let Some(total) = state.qr_receive.total_chunks {
        children.push(
            serde_json::to_value(
                UiText::new(&format!(
                    "Progress: {}/{}",
                    state.qr_receive.chunks.iter().filter(|c| c.is_some()).count(),
                    total
                ))
                .size(12.0),
            )
            .unwrap(),
        );
    }
    if let Some(last) = &state.qr_receive.last_scanned {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Last: {}", last))
                    .size(10.0)
                    .content_description("qr_last"),
            )
            .unwrap(),
        );
    }
    if let Some(path) = &state.qr_receive.result_path {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Saved: {}", path))
                    .size(12.0)
                    .content_description("qr_receive_path"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(UiButton::new("Copy path", "copy_clipboard").copy_text(path)).unwrap(),
        );
    } else if state
        .qr_receive
        .total_chunks
        .map(|t| state.qr_receive.chunks.iter().filter(|c| c.is_some()).count() == t)
        .unwrap_or(false)
    {
        children.push(
            serde_json::to_value(UiButton::new("Save file", "qr_receive_save").id("qr_receive_save"))
                .unwrap(),
        );
    }

    if state.nav_depth() > 1 {
        children.push(serde_json::to_value(UiButton::new("Back", "back")).unwrap());
    }

    serde_json::to_value(UiColumn::new(children).padding(20).id("QrReceiveScreen")).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    #[test]
    fn chunking_produces_header_and_counts() {
        let data = vec![1u8; 1200];
        let chunks = chunk_bytes(&data);
        assert_eq!(chunks.len(), 3);
        assert!(chunks[0].starts_with("QRTX|1/3|"));
        assert!(chunks[2].starts_with("QRTX|3/3|"));
    }

    #[test]
    fn advance_wraps_and_refreshes() {
        let mut state = AppState::new();
        state.qr_slideshow.chunks = vec!["QRTX|1/2|AAA".into(), "QRTX|2/2|BBB".into()];
        state.qr_slideshow.current_index = 0;
        refresh_current_qr(&mut state).unwrap();
        advance_frame(&mut state, 1).unwrap();
        assert_eq!(state.qr_slideshow.current_index, 1);
        advance_frame(&mut state, 1).unwrap();
        assert_eq!(state.qr_slideshow.current_index, 0);
    }

    #[test]
    fn receive_out_of_order_reassembles() {
        let mut state = AppState::new();
        let data = vec![42u8; CHUNK_BYTES + 50];
        let chunks = chunk_bytes(&data);
        assert!(chunks.len() >= 2);
        handle_receive_scan(&mut state, &chunks[1]).unwrap();
        handle_receive_scan(&mut state, &chunks[0]).unwrap();
        let assembled = finalize_receive(&mut state).unwrap();
        assert_eq!(assembled, data);
    }

    #[test]
    fn decode_qr_frame_stub_returns_error() {
        let buf = vec![0u8; 16];
        let err = decode_qr_frame_luma(4, 4, 4, 0, &buf).unwrap_err();
        assert_eq!(err, "qr_decoder_unavailable_offline");
    }
}
