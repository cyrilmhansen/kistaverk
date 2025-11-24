use crate::state::{AppState, Screen};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImageTarget {
    Webp,
    Png,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConversionResult {
    pub path: Option<String>,
    pub size: Option<String>,
    pub format: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KotlinImageState {
    pub target: Option<ImageTarget>,
    pub result: Option<ImageConversionResult>,
    pub output_dir: Option<String>,
}

impl KotlinImageState {
    pub const fn new() -> Self {
        Self {
            target: None,
            result: None,
            output_dir: None,
        }
    }

    pub fn reset(&mut self) {
        self.target = None;
        self.result = None;
        self.output_dir = None;
    }
}

pub fn handle_screen_entry(state: &mut AppState, target: ImageTarget) {
    state.push_screen(Screen::KotlinImage);
    state.image.target = Some(target);
    state.image.result = None;
    state.image.output_dir = None;
    state.last_error = None;
}

pub fn handle_result(
    state: &mut AppState,
    target: Option<ImageTarget>,
    result: ImageConversionResult,
) {
    state.replace_current(Screen::KotlinImage);
    if let Some(t) = target {
        state.image.target = Some(t);
    }
    state.image.result = Some(result);
}

pub fn handle_output_dir(state: &mut AppState, target: Option<ImageTarget>, dir: Option<String>) {
    state.replace_current(Screen::KotlinImage);
    if let Some(t) = target {
        state.image.target = Some(t);
    }
    state.image.output_dir = dir;
}

pub fn render_kotlin_image_screen(state: &AppState) -> Value {
    let target = state.image.target.unwrap_or(ImageTarget::Webp);
    let (title, short_label) = image_target_labels(target);
    let mut children = vec![
        json!({
            "type": "Text",
            "text": format!("{} (Kotlin pipeline)", title),
            "size": 20.0
        }),
        json!({
            "type": "Text",
            "text": "Conversion happens on the Kotlin side; Rust keeps navigation and status.",
            "size": 14.0
        }),
        json!({
            "type": "Text",
            "text": format!(
                "Output: {}",
                state
                    .image
                    .output_dir
                    .as_deref()
                    .unwrap_or("MediaStore -> Pictures/kistaverk (visible in gallery)")
            ),
            "size": 13.0
        }),
        json!({
            "type": "Button",
            "text": "Choose output folder (optional)",
            "action": "kotlin_image_pick_dir",
            "requires_file_picker": false
        }),
        json!({
            "type": "Button",
            "text": format!("Select image → {}", short_label),
            "action": convert_action_for_target(target),
            "requires_file_picker": true
        }),
    ];

    if let Some(result) = &state.image.result {
        if let Some(err) = &result.error {
            children.push(json!({
                "type": "Text",
                "text": format!("Failed: {err}"),
                "size": 14.0
            }));
        } else {
            if let Some(format) = &result.format {
                children.push(json!({
                    "type": "Text",
                    "text": format!("Saved as {format}"),
                }));
            }
            if let Some(path) = &result.path {
                children.push(json!({
                    "type": "Text",
                    "text": format!("Path: {path}"),
                }));
            }
            if let Some(size) = &result.size {
                children.push(json!({
                    "type": "Text",
                    "text": format!("Size: {size}"),
                }));
            }
        }
    }

    if state.nav_depth() > 1 {
        children.push(json!({
            "type": "Button",
            "text": "Back",
            "action": "back",
            "requires_file_picker": false
        }));
    }

    json!({
        "type": "Column",
        "padding": 24,
        "children": children
    })
}

pub fn parse_image_target(raw: &str) -> Option<ImageTarget> {
    match raw.to_ascii_lowercase().as_str() {
        "webp" => Some(ImageTarget::Webp),
        "png" => Some(ImageTarget::Png),
        _ => None,
    }
}

pub fn image_target_labels(target: ImageTarget) -> (&'static str, &'static str) {
    match target {
        ImageTarget::Webp => ("Image → WebP", "WebP"),
        ImageTarget::Png => ("Image → PNG", "PNG"),
    }
}

pub fn convert_action_for_target(target: ImageTarget) -> &'static str {
    match target {
        ImageTarget::Webp => "kotlin_image_convert_webp",
        ImageTarget::Png => "kotlin_image_convert_png",
    }
}
