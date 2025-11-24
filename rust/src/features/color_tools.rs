use crate::state::{AppState, Screen};
use serde_json::json;

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Hsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

pub fn handle_color_action(state: &mut AppState, action: &str, input: &str) {
    state.replace_current(Screen::ColorTools);
    match action {
        "color_from_hex" => match parse_hex(input) {
            Ok(rgb) => apply_color_result(state, rgb),
            Err(e) => state.last_error = Some(e),
        },
        "color_from_rgb" => match parse_rgb_triplet(input) {
            Ok(rgb) => apply_color_result(state, rgb),
            Err(e) => state.last_error = Some(e),
        },
        "color_copy_hex_input" => {
            if let Some(hex) = state.text_input.clone() {
                state.text_input = Some(hex);
            } else {
                state.last_error = Some("no_color".into());
            }
        }
        "color_copy_rgb_input" => {
            if let Some(csv) = state.last_hash_algo.clone() {
                state.text_input = Some(csv);
            } else {
                state.last_error = Some("no_color".into());
            }
        }
        "color_copy_hsl_input" => {
            if let Some(hsl) = state.text_operation.clone() {
                state.text_input = Some(hsl);
            } else {
                state.last_error = Some("no_color".into());
            }
        }
        _ => state.last_error = Some("unknown_color_action".into()),
    }
}

fn apply_color_result(state: &mut AppState, rgb: Rgb) {
    let hsl = rgb_to_hsl(rgb);
    state.last_error = None;
    state.text_input = Some(format!("#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b));
    state.text_output = Some(format!(
        "Result: Hex #{:02X}{:02X}{:02X} | RGB {}, {}, {} | HSL {:.0}°, {:.0}%, {:.0}%",
        rgb.r,
        rgb.g,
        rgb.b,
        rgb.r,
        rgb.g,
        rgb.b,
        hsl.h,
        hsl.s * 100.0,
        hsl.l * 100.0
    ));
    state.last_hash_algo = Some(format!("{},{},{}", rgb.r, rgb.g, rgb.b)); // reuse slot to carry swatch color / rgb csv
    state.text_operation = Some(format!(
        "{:.0},{:.0},{:.0}",
        hsl.h,
        hsl.s * 100.0,
        hsl.l * 100.0
    ));
}

pub fn render_color_screen(state: &AppState) -> serde_json::Value {
    let mut children = vec![
        json!({ "type": "Text", "text": "Color Converter", "size": 20.0 }),
        json!({ "type": "Text", "text": "Convert Hex <-> RGB with HSL hint. Enter #RRGGBB or \"255,128,0\".", "size": 14.0 }),
        json!({ "type": "TextInput", "bind_key": "color_input", "hint": "#1A2B3C or 26,43,60", "action_on_submit": "color_from_hex" }),
        json!({ "type": "Button", "text": "Hex → RGB/HSL", "action": "color_from_hex" }),
        json!({ "type": "Button", "text": "RGB → Hex/HSL", "action": "color_from_rgb" }),
    ];

    if let Some(out) = &state.text_output {
        let (hex, rgb, hsl_text) = color_strings(state, out);
        children.push(json!({
            "type": "Text",
            "text": out,
            "size": 14.0
        }));
        children.push(json!({
            "type": "Button",
            "text": "Copy Hex",
            "copy_text": hex
        }));
        children.push(json!({
            "type": "Button",
            "text": "Copy RGB",
            "copy_text": rgb
        }));
        children.push(json!({
            "type": "Button",
            "text": "Copy HSL",
            "copy_text": hsl_text
        }));
    }

    if let Some(rgb_csv) = &state.last_hash_algo {
        let parts: Vec<_> = rgb_csv
            .split(',')
            .filter_map(|p| p.parse::<u8>().ok())
            .collect();
        if parts.len() == 3 {
            let color = (0xFF000000u32
                | ((parts[0] as u32) << 16)
                | ((parts[1] as u32) << 8)
                | parts[2] as u32) as i64;
            children.push(json!({
                "type": "ColorSwatch",
                "color": color,
                "content_description": "Color preview"
            }));
            children.push(json!({
                "type": "Button",
                "text": "Put Hex in input",
                "action": "color_copy_hex_input"
            }));
            if let Some(hsl) = &state.text_operation {
                children.push(json!({
                    "type": "Text",
                    "text": format!("HSL: {}", hsl),
                    "size": 12.0
                }));
            }
        }
    }

    json!({
        "type": "Column",
        "padding": 24,
        "children": children
    })
}

fn color_strings(state: &AppState, fallback: &str) -> (String, String, String) {
    let hex = state.text_input.clone().unwrap_or_else(|| fallback.to_string());
    let rgb = state.last_hash_algo.clone().unwrap_or_else(|| fallback.to_string());
    let hsl = state.text_operation.clone().unwrap_or_else(|| fallback.to_string());
    (hex, rgb, hsl)
}

fn parse_hex(raw: &str) -> Result<Rgb, String> {
    let trimmed = raw.trim().trim_start_matches('#');
    if trimmed.len() != 6 {
        return Err("invalid_hex_length".into());
    }
    let r = u8::from_str_radix(&trimmed[0..2], 16).map_err(|_| "invalid_hex")?;
    let g = u8::from_str_radix(&trimmed[2..4], 16).map_err(|_| "invalid_hex")?;
    let b = u8::from_str_radix(&trimmed[4..6], 16).map_err(|_| "invalid_hex")?;
    Ok(Rgb { r, g, b })
}

fn parse_rgb_triplet(raw: &str) -> Result<Rgb, String> {
    let parts: Vec<_> = raw.split(',').map(|p| p.trim()).collect();
    if parts.len() != 3 {
        return Err("invalid_rgb".into());
    }
    let r = parts[0].parse::<u8>().map_err(|_| "invalid_rgb")?;
    let g = parts[1].parse::<u8>().map_err(|_| "invalid_rgb")?;
    let b = parts[2].parse::<u8>().map_err(|_| "invalid_rgb")?;
    Ok(Rgb { r, g, b })
}

fn rgb_to_hsl(rgb: Rgb) -> Hsl {
    let r = rgb.r as f32 / 255.0;
    let g = rgb.g as f32 / 255.0;
    let b = rgb.b as f32 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let l = (max + min) / 2.0;
    let s = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    Hsl { h: h.rem_euclid(360.0), s, l }
}
