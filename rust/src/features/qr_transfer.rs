use crate::state::AppState;
use crate::ui::{Button as UiButton, Column as UiColumn, Text as UiText};
use base64::Engine;
use image::{codecs::png::PngEncoder, ColorType, ImageBuffer, ImageEncoder, Luma};
use qrcode::{Color, QrCode};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::io::{FromRawFd, RawFd};

const CHUNK_BYTES: usize = 512;

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

fn chunk_bytes(bytes: &[u8]) -> Vec<String> {
    if bytes.is_empty() {
        return Vec::new();
    }
    let mut chunks = Vec::new();
    for (i, chunk) in bytes.chunks(CHUNK_BYTES).enumerate() {
        let total_chunks = (bytes.len() + CHUNK_BYTES - 1) / CHUNK_BYTES;
        let encoded = base64::engine::general_purpose::STANDARD.encode(chunk);
        let payload = format!("QRTX|{}/{}|{}", i + 1, total_chunks, encoded);
        chunks.push(payload);
    }
    chunks
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
}
