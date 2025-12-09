use crate::state::AppState;
use crate::ui::{maybe_push_back, Button, Checkbox, Column, Grid, Text, TextInput};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KotlinImageState {
    pub active_tool: Option<ImageTool>,
    pub source_path: Option<String>,
    pub result: Option<ImageConversionResult>,
    pub resize_scale_pct: u32,
    pub resize_quality: u32,
    pub resize_target_kb: Option<u64>,
    pub resize_use_webp: bool,
    pub output_dir: Option<String>,
    pub batch_queue: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageTool {
    Convert,
    Resize,
}

impl KotlinImageState {
    pub const fn new() -> Self {
        Self {
            active_tool: None,
            source_path: None,
            result: None,
            resize_scale_pct: 70,
            resize_quality: 85,
            resize_target_kb: None,
            resize_use_webp: false,
            output_dir: None,
            batch_queue: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.active_tool = None;
        self.source_path = None;
        self.result = None;
        self.resize_scale_pct = 70;
        self.resize_quality = 85;
        self.resize_target_kb = None;
        self.resize_use_webp = false;
        self.batch_queue.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

fn to_value_or_text<T: Serialize>(value: T, context: &str) -> Value {
    serde_json::to_value(value).unwrap_or_else(|e| {
        json!({
            "type": "Text",
            "text": format!("{context}_serialize_error:{e}")
        })
    })
}

pub fn handle_screen_entry(state: &mut AppState, _target: ImageTarget) {
    state.image.active_tool = Some(ImageTool::Convert);
    state.image.source_path = None;
    state.image.result = None;
}

pub fn handle_resize_screen(state: &mut AppState) {
    state.image.active_tool = Some(ImageTool::Resize);
    state.image.source_path = None;
    state.image.result = None;
}

pub fn handle_resize_sync(state: &mut AppState, bindings: &HashMap<String, String>) {
    if let Some(val) = bindings.get("resize_scale_pct") {
        if let Ok(v) = val.parse::<u32>() {
            state.image.resize_scale_pct = v.clamp(5, 100);
        }
    }
    if let Some(val) = bindings.get("resize_quality") {
        if let Ok(v) = val.parse::<u32>() {
            state.image.resize_quality = v.clamp(10, 100);
        }
    }
    if let Some(val) = bindings.get("resize_target_kb") {
        if let Ok(v) = val.parse::<u64>() {
            state.image.resize_target_kb = Some(v);
        } else if val.trim().is_empty() {
            state.image.resize_target_kb = None;
        }
    }
    if let Some(val) = bindings.get("resize_use_webp") {
        state.image.resize_use_webp = val == "true";
    }
}

pub fn handle_result(
    state: &mut AppState,
    _target: Option<ImageTarget>,
    result: ImageConversionResult,
    bindings: Option<&HashMap<String, String>>,
) {
    state.image.result = Some(result);

    if let Some(b) = bindings {
        handle_resize_sync(state, b);
    }
}

pub fn handle_output_dir(
    state: &mut AppState,
    _target: Option<ImageTarget>,
    output_dir: Option<String>,
) {
    state.image.output_dir = output_dir;
}

pub fn parse_image_target(s: &str) -> Option<ImageTarget> {
    match s {
        "webp" => Some(ImageTarget::Webp),
        "png" => Some(ImageTarget::Png),
        "jpeg" | "jpg" => Some(ImageTarget::Jpeg),
        _ => None,
    }
}

pub fn render_kotlin_image_screen(state: &AppState) -> Value {
    match state.image.active_tool {
        Some(ImageTool::Convert) => render_converter(state),
        Some(ImageTool::Resize) => render_resizer(state),
        None => render_menu(state),
    }
}

fn render_menu(_state: &AppState) -> Value {
    let children = vec![
        to_value_or_text(Text::new("Image Tools").size(24.0), "title"),
        to_value_or_text(
            Text::new("Select a tool to continue.").size(14.0),
            "subtitle",
        ),
        to_value_or_text(
            Button::new("Format Converter", "kotlin_image_screen_webp")
                .content_description("open_converter"),
            "btn_converter",
        ),
        to_value_or_text(
            Button::new("Resize & Compress", "kotlin_image_resize_screen")
                .content_description("open_resizer"),
            "btn_resizer",
        ),
    ];
    to_value_or_text(Column::new(children).padding(20), "menu_root")
}

fn render_converter(state: &AppState) -> Value {
    let mut children = vec![
        to_value_or_text(Text::new("Format Converter").size(20.0), "title"),
        to_value_or_text(
            Button::new("Select Image", "kotlin_image_pick").requires_file_picker(true),
            "picker",
        ),
        to_value_or_text(
            Button::new("Select Images (batch)", "kotlin_image_batch_pick")
                .requires_file_picker(true)
                .allow_multiple_files(true),
            "picker_batch",
        ),
    ];

    if let Some(path) = &state.image.source_path {
        children.push(to_value_or_text(
            Text::new(&format!("Selected: {}", path)).size(12.0),
            "selected_path",
        ));

        // Hidden input to pass path to Kotlin
        children.push(to_value_or_text(
            TextInput::new("image_source_path") // bind_key
                .text(path)
                .content_description("hidden_source_path"),
            "input_source_path",
        ));

        children.push(to_value_or_text(
            Text::new("Convert to:").size(16.0),
            "label_convert",
        ));

        let grid_children = vec![
            to_value_or_text(Button::new("WebP", "kotlin_image_convert_webp"), "btn_webp"),
            to_value_or_text(Button::new("PNG", "kotlin_image_convert_png"), "btn_png"),
            to_value_or_text(Button::new("JPEG", "kotlin_image_convert_jpeg"), "btn_jpeg"),
        ];
        children.push(to_value_or_text(
            Grid::new(grid_children).columns(3),
            "grid_convert",
        ));
    }

    if !state.image.batch_queue.is_empty() {
        children.push(render_batch_list(&state.image.batch_queue, "convert"));
        let batch_buttons = vec![
            to_value_or_text(
                Button::new("Process batch → WebP", "kotlin_image_batch_process")
                    .payload(json!({
                        "image_batch_paths": state.image.batch_queue,
                        "image_batch_target": "webp",
                        "image_batch_mode": "convert"
                    })),
                "batch_webp",
            ),
            to_value_or_text(
                Button::new("Process batch → PNG", "kotlin_image_batch_process")
                    .payload(json!({
                        "image_batch_paths": state.image.batch_queue,
                        "image_batch_target": "png",
                        "image_batch_mode": "convert"
                    })),
                "batch_png",
            ),
            to_value_or_text(
                Button::new("Process batch → JPEG", "kotlin_image_batch_process")
                    .payload(json!({
                        "image_batch_paths": state.image.batch_queue,
                        "image_batch_target": "jpeg",
                        "image_batch_mode": "convert"
                    })),
                "batch_jpeg",
            ),
        ];
        children.push(to_value_or_text(Column::new(batch_buttons), "batch_actions"));
    }

    render_result_area(&mut children, state);

    maybe_push_back(&mut children, state);
    to_value_or_text(Column::new(children).padding(20), "converter_root")
}

fn render_resizer(state: &AppState) -> Value {
    let mut children = vec![
        to_value_or_text(Text::new("Resize & Compress").size(20.0), "title"),
        to_value_or_text(
            Button::new("Select Image", "kotlin_image_pick").requires_file_picker(true),
            "picker",
        ),
        to_value_or_text(
            Button::new("Select Images (batch)", "kotlin_image_batch_pick")
                .requires_file_picker(true)
                .allow_multiple_files(true),
            "picker_batch",
        ),
    ];

    if let Some(path) = &state.image.source_path {
        children.push(to_value_or_text(
            Text::new(&format!("Selected: {}", path)).size(12.0),
            "selected_path",
        ));

        children.push(to_value_or_text(
            TextInput::new("image_source_path")
                .text(path)
                .content_description("hidden_source_path"),
            "input_source_path",
        ));

        // Scale
        children.push(to_value_or_text(Text::new("Scale % (5-100)"), "lbl_scale"));
        children.push(to_value_or_text(
            TextInput::new("resize_scale_pct")
                .text(&state.image.resize_scale_pct.to_string()),
            "input_scale",
        ));

        // Quality
        children.push(to_value_or_text(
            Text::new("Quality (10-100)"),
            "lbl_quality",
        ));
        children.push(to_value_or_text(
            TextInput::new("resize_quality")
                .text(&state.image.resize_quality.to_string()),
            "input_quality",
        ));

        // Target Size
        children.push(to_value_or_text(
            Text::new("Max Size (KB) - Optional"),
            "lbl_target",
        ));
        let target_val = state
            .image
            .resize_target_kb
            .map(|v| v.to_string())
            .unwrap_or_default();
        children.push(to_value_or_text(
            TextInput::new("resize_target_kb")
                .text(&target_val)
                .hint("e.g. 500"),
            "input_target",
        ));

        // Use WebP
        children.push(to_value_or_text(
            Checkbox::new("Convert to WebP (Efficient)", "resize_use_webp")
                .checked(state.image.resize_use_webp),
            "check_webp",
        ));

        // Action
        children.push(to_value_or_text(
            Button::new("Process Image", "kotlin_image_resize"),
            "btn_process",
        ));
    }

    if !state.image.batch_queue.is_empty() {
        children.push(render_batch_list(&state.image.batch_queue, "resize"));
        children.push(to_value_or_text(
            Button::new("Process batch (resize)", "kotlin_image_batch_process").payload(json!({
                "image_batch_paths": state.image.batch_queue,
                "image_batch_mode": "resize"
            })),
            "batch_resize",
        ));
    }

    render_result_area(&mut children, state);

    maybe_push_back(&mut children, state);
    to_value_or_text(Column::new(children).padding(20), "resizer_root")
}

fn render_result_area(children: &mut Vec<Value>, state: &AppState) {
    if let Some(res) = &state.image.result {
        // Divider
        children.push(to_value_or_text(Text::new("---").size(12.0), "div_res"));

        if let Some(err) = &res.error {
            children.push(to_value_or_text(
                Text::new(&format!("Error: {}", err)),
                "err_msg",
            ));
        } else if let Some(dest) = &res.path {
            children.push(to_value_or_text(
                Text::new("Success!").size(18.0),
                "success_title",
            ));
            children.push(to_value_or_text(
                Text::new(&format!("Saved to: {}", dest)).size(12.0),
                "success_path",
            ));
            if let Some(sz) = &res.size {
                children.push(to_value_or_text(
                    Text::new(&format!("Size: {}", sz)).size(12.0),
                    "success_size",
                ));
            }
            if let Some(fmt) = &res.format {
                children.push(to_value_or_text(
                    Text::new(&format!("Format: {}", fmt)).size(12.0),
                    "success_fmt",
                ));
            }

            children.push(to_value_or_text(
                Button::new("Save As...", "kotlin_image_save_as"),
                "btn_save_as",
            ));
        }
    }
}

fn render_batch_list(paths: &[String], _mode: &str) -> Value {
    let items: Vec<Value> = paths
        .iter()
        .map(|p| {
            to_value_or_text(
                Column::new(vec![
                    to_value_or_text(Text::new(p).size(12.0), "batch_item_text"),
                    to_value_or_text(
                        Button::new("Remove", "kotlin_image_batch_remove")
                            .payload(json!({ "image_batch_path": p })),
                        "batch_remove_btn",
                    ),
                ])
                .padding(4),
                "batch_item",
            )
        })
        .collect();
    to_value_or_text(
        crate::ui::VirtualList::new(items).estimated_item_height(48),
        "batch_list",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    #[test]
    fn test_kotlin_image_state_serialization() {
        let mut state = KotlinImageState::new();
        state.active_tool = Some(ImageTool::Resize);
        state.source_path = Some("/tmp/test.png".into());
        state.resize_scale_pct = 50;

        let json = serde_json::to_string(&state).expect("serialize failed");
        let deserialized: KotlinImageState =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(deserialized.active_tool, Some(ImageTool::Resize));
        assert_eq!(deserialized.source_path, Some("/tmp/test.png".into()));
        assert_eq!(deserialized.resize_scale_pct, 50);
    }

    #[test]
    fn test_render_converter_generates_hidden_input() {
        let mut app_state = AppState::new();
        app_state.image.active_tool = Some(ImageTool::Convert);
        app_state.image.source_path = Some("/path/to/image.jpg".into());

        let ui = render_kotlin_image_screen(&app_state);
        let children = ui
            .get("children")
            .and_then(|v| v.as_array())
            .expect("no children");

        // Search for the hidden input in the children
        let hidden_input = children.iter().find(|child| {
            child.get("type").and_then(|t| t.as_str()) == Some("TextInput")
                && child.get("bind_key").and_then(|k| k.as_str()) == Some("image_source_path")
        });

        assert!(
            hidden_input.is_some(),
            "Hidden input for image_source_path not found"
        );
        let input = hidden_input.unwrap();
        assert_eq!(
            input.get("text").and_then(|t| t.as_str()),
            Some("/path/to/image.jpg")
        );
    }
}
