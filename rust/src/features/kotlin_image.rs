use crate::state::{AppState, Screen};
use crate::ui::{Button as UiButton, Checkbox as UiCheckbox, Column as UiColumn, Text as UiText};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KotlinImageMode {
    Convert,
    Resize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImageTarget {
    Webp,
    Png,
    Jpeg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConversionResult {
    pub path: Option<String>,
    pub size: Option<String>,
    pub format: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KotlinImageState {
    pub target: Option<ImageTarget>,
    pub result: Option<ImageConversionResult>,
    pub output_dir: Option<String>,
    pub mode: KotlinImageMode,
    pub resize_scale_pct: u32,
    pub resize_quality: u8,
    pub resize_target_kb: Option<u32>,
}

impl Default for KotlinImageState {
    fn default() -> Self {
        Self::new()
    }
}

impl KotlinImageState {
    pub const fn new() -> Self {
        Self {
            target: None,
            result: None,
            output_dir: None,
            mode: KotlinImageMode::Convert,
            resize_scale_pct: 70,
            resize_quality: 85,
            resize_target_kb: Some(200),
        }
    }

    pub fn reset(&mut self) {
        self.target = None;
        self.result = None;
        self.output_dir = None;
        self.mode = KotlinImageMode::Convert;
        self.resize_scale_pct = 70;
        self.resize_quality = 85;
        self.resize_target_kb = Some(200);
    }

    fn apply_resize_bindings(&mut self, bindings: &HashMap<String, String>) {
        if let Some(scale) = parse_u32(bindings, "resize_scale_pct") {
            self.resize_scale_pct = scale.clamp(5, 100);
        }
        if let Some(q) = parse_u32(bindings, "resize_quality") {
            self.resize_quality = q.min(100).max(40) as u8;
        }
        if bindings.contains_key("resize_target_kb") {
            self.resize_target_kb = parse_u32(bindings, "resize_target_kb");
        }
        if let Some(use_webp) = parse_bool(bindings, "resize_use_webp") {
            self.target = Some(if use_webp {
                ImageTarget::Webp
            } else {
                ImageTarget::Jpeg
            });
        }
    }
}

pub fn handle_screen_entry(state: &mut AppState, target: ImageTarget) {
    state.push_screen(Screen::KotlinImage);
    state.image.target = Some(target);
    state.image.result = None;
    state.image.output_dir = None;
    state.image.mode = KotlinImageMode::Convert;
    state.last_error = None;
}

pub fn handle_result(
    state: &mut AppState,
    target: Option<ImageTarget>,
    result: ImageConversionResult,
    bindings: Option<&HashMap<String, String>>,
) {
    state.replace_current(Screen::KotlinImage);
    if let Some(t) = target {
        state.image.target = Some(t);
    }
    state.image.result = Some(result);
    if state.image.mode == KotlinImageMode::Resize {
        if let Some(b) = bindings {
            state.image.apply_resize_bindings(b);
        }
    }
}

pub fn handle_output_dir(state: &mut AppState, target: Option<ImageTarget>, dir: Option<String>) {
    state.replace_current(Screen::KotlinImage);
    if let Some(t) = target {
        state.image.target = Some(t);
    }
    state.image.output_dir = dir;
}

pub fn handle_resize_screen(state: &mut AppState) {
    state.push_screen(Screen::KotlinImage);
    state.image.mode = KotlinImageMode::Resize;
    state.image.target = Some(ImageTarget::Jpeg);
    state.image.result = None;
    state.image.output_dir = None;
    state.image.resize_scale_pct = 70;
    state.image.resize_quality = 85;
    state.image.resize_target_kb = Some(200);
    state.last_error = None;
}

pub fn handle_resize_sync(state: &mut AppState, bindings: &HashMap<String, String>) {
    state.replace_current(Screen::KotlinImage);
    state.image.mode = KotlinImageMode::Resize;
    state.image.apply_resize_bindings(bindings);
}

pub fn render_kotlin_image_screen(state: &AppState) -> Value {
    match state.image.mode {
        KotlinImageMode::Convert => render_convert_screen(state),
        KotlinImageMode::Resize => render_resize_screen(state),
    }
}

fn render_convert_screen(state: &AppState) -> Value {
    let target = state.image.target.unwrap_or(ImageTarget::Webp);
    let (title, short_label) = image_target_labels(target);
    let mut children = vec![
        serde_json::to_value(UiText::new(&format!("{} (Kotlin pipeline)", title)).size(20.0))
            .unwrap(),
        serde_json::to_value(
            UiText::new("Conversion happens on the Kotlin side; Rust keeps navigation and status.")
                .size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiText::new(&format!(
                "Output: {}",
                state
                    .image
                    .output_dir
                    .as_deref()
                    .unwrap_or("MediaStore -> Pictures/kistaverk (visible in gallery)")
            ))
            .size(13.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Choose output folder (optional)", "kotlin_image_pick_dir")
                .requires_file_picker(false),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new(
                &format!("Select image → {}", short_label),
                convert_action_for_target(target),
            )
            .requires_file_picker(true),
        )
        .unwrap(),
    ];

    if let Some(result) = &state.image.result {
        if let Some(err) = &result.error {
            children.push(
                serde_json::to_value(UiText::new(&format!("Failed: {err}")).size(14.0)).unwrap(),
            );
        } else {
            if let Some(format) = &result.format {
                children.push(
                    serde_json::to_value(UiText::new(&format!("Saved as {format}"))).unwrap(),
                );
            }
            if let Some(path) = &result.path {
                children.push(serde_json::to_value(UiText::new(&format!("Path: {path}"))).unwrap());
                children.push(
                    serde_json::to_value(UiButton::new("Save as…", "kotlin_image_save_as"))
                        .unwrap(),
                );
            }
            if let Some(size) = &result.size {
                children.push(serde_json::to_value(UiText::new(&format!("Size: {size}"))).unwrap());
            }
        }
    }

    if state.nav_depth() > 1 {
        children.push(
            serde_json::to_value(UiButton::new("Back", "back").requires_file_picker(false))
                .unwrap(),
        );
    }

    serde_json::to_value(UiColumn::new(children).padding(24)).unwrap()
}

fn render_resize_screen(state: &AppState) -> Value {
    let target = state.image.target.unwrap_or(ImageTarget::Jpeg);
    let use_webp = matches!(target, ImageTarget::Webp);
    let mut children = vec![
        serde_json::to_value(UiText::new("Image resize for sharing").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Scale down and recompress images to fit messaging/email size limits.")
                .size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiText::new(&format!(
                "Output: {}",
                state
                    .image
                    .output_dir
                    .as_deref()
                    .unwrap_or("MediaStore → Pictures/kistaverk (visible in gallery)")
            ))
            .size(13.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiCheckbox::new("Prefer WebP (smaller, modern)", "resize_use_webp")
                .checked(use_webp)
                .action("kotlin_image_resize_sync"),
        )
        .unwrap(),
        serde_json::to_value(
            UiText::new(&format!("Format: {}", image_target_labels(target).0)).size(13.0),
        )
        .unwrap(),
        serde_json::to_value(UiText::new("Scale percent (5-100)").size(13.0)).unwrap(),
        serde_json::to_value(
            crate::ui::TextInput::new("resize_scale_pct")
                .text(&state.image.resize_scale_pct.to_string())
                .hint("70")
                .action_on_submit("kotlin_image_resize_sync"),
        )
        .unwrap(),
        serde_json::to_value(UiText::new("Quality (40-100)").size(13.0)).unwrap(),
        serde_json::to_value(
            crate::ui::TextInput::new("resize_quality")
                .text(&state.image.resize_quality.to_string())
                .hint("85")
                .action_on_submit("kotlin_image_resize_sync"),
        )
        .unwrap(),
        serde_json::to_value(
            UiText::new("Target size KB (optional, lowers quality to fit)").size(13.0),
        )
        .unwrap(),
        serde_json::to_value(
            crate::ui::TextInput::new("resize_target_kb")
                .text(
                    &state
                        .image
                        .resize_target_kb
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                )
                .hint("200")
                .action_on_submit("kotlin_image_resize_sync"),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Choose output folder (optional)", "kotlin_image_pick_dir")
                .requires_file_picker(false),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Select image → Resize", "kotlin_image_resize")
                .requires_file_picker(true),
        )
        .unwrap(),
    ];

    if let Some(result) = &state.image.result {
        if let Some(err) = &result.error {
            children.push(
                serde_json::to_value(UiText::new(&format!("Failed: {err}")).size(14.0)).unwrap(),
            );
        } else {
            if let Some(format) = &result.format {
                children.push(
                    serde_json::to_value(UiText::new(&format!("Saved as {format}"))).unwrap(),
                );
            }
            if let Some(path) = &result.path {
                children.push(serde_json::to_value(UiText::new(&format!("Path: {path}"))).unwrap());
                children.push(
                    serde_json::to_value(UiButton::new("Save as…", "kotlin_image_save_as"))
                        .unwrap(),
                );
            }
            if let Some(size) = &result.size {
                children.push(serde_json::to_value(UiText::new(&format!("Size: {size}"))).unwrap());
            }
        }
    }

    if state.nav_depth() > 1 {
        children.push(
            serde_json::to_value(UiButton::new("Back", "back").requires_file_picker(false))
                .unwrap(),
        );
    }

    serde_json::to_value(UiColumn::new(children).padding(24)).unwrap()
}

pub fn parse_image_target(raw: &str) -> Option<ImageTarget> {
    match raw.to_ascii_lowercase().as_str() {
        "webp" => Some(ImageTarget::Webp),
        "png" => Some(ImageTarget::Png),
        "jpeg" | "jpg" => Some(ImageTarget::Jpeg),
        _ => None,
    }
}

pub fn image_target_labels(target: ImageTarget) -> (&'static str, &'static str) {
    match target {
        ImageTarget::Webp => ("Image → WebP", "WebP"),
        ImageTarget::Png => ("Image → PNG", "PNG"),
        ImageTarget::Jpeg => ("Image → JPEG", "JPEG"),
    }
}

pub fn convert_action_for_target(target: ImageTarget) -> &'static str {
    match target {
        ImageTarget::Webp => "kotlin_image_convert_webp",
        ImageTarget::Png => "kotlin_image_convert_png",
        ImageTarget::Jpeg => "kotlin_image_convert_webp",
    }
}

fn parse_u32(bindings: &HashMap<String, String>, key: &str) -> Option<u32> {
    bindings.get(key)?.trim().parse::<u32>().ok()
}

fn parse_bool(bindings: &HashMap<String, String>, key: &str) -> Option<bool> {
    bindings.get(key)?.trim().parse::<bool>().ok()
}
