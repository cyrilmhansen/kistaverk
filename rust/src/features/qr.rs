use crate::state::{AppState, Screen};
use base64::Engine;
use image::{codecs::png::PngEncoder, ColorType, ImageBuffer, ImageEncoder, Luma};
use qrcode::{Color, QrCode};
use serde_json::json;

pub fn handle_qr_action(state: &mut AppState, input: &str) -> Result<(), String> {
    if input.is_empty() {
        state.last_error = Some("qr_empty_input".into());
        state.last_qr_base64 = None;
        state.replace_current(Screen::Qr);
        return Ok(());
    }

    let code = QrCode::new(input.as_bytes()).map_err(|e| format!("qr_encode_failed:{e}"))?;
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

    // Scale up to 256px-ish while keeping square pixels
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
        .write_image(
            &scaled,
            scaled.width(),
            scaled.height(),
            ColorType::L8,
        )
        .map_err(|e| format!("qr_png_failed:{e}"))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(buf);
    state.last_error = None;
    state.last_qr_base64 = Some(b64);
    state.replace_current(Screen::Qr);
    Ok(())
}

pub fn render_qr_screen(state: &AppState) -> serde_json::Value {
    let mut children = vec![
        json!({ "type": "Text", "text": "QR Code Generator", "size": 20.0 }),
        json!({
            "type": "Text",
            "text": "Enter text to generate a QR code.",
            "size": 14.0
        }),
        json!({
            "type": "TextInput",
            "bind_key": "qr_input",
            "hint": "Text or URL",
            "action_on_submit": "qr_generate"
        }),
        json!({
            "type": "Button",
            "text": "Generate QR",
            "action": "qr_generate"
        }),
    ];

    if let Some(b64) = &state.last_qr_base64 {
        children.push(json!({
            "type": "Text",
            "text": "Result:",
            "size": 14.0
        }));
        children.push(json!({
            "type": "ImageBase64",
            "base64": b64,
            "content_description": "Generated QR"
        }));
    }

    json!({
        "type": "Column",
        "padding": 24,
        "children": children
    })
}
