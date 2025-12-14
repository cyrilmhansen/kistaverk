use crate::state::AppState;
use serde::Serialize;
use serde_json::{json, Value};
use rust_i18n::t;

#[derive(Serialize)]
pub struct Text<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Text<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            kind: "Text",
            text,
            id: None,
            size: None,
            color: None,
            content_description: None,
        }
    }

    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn size(mut self, size: f64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn color(mut self, color: &'a str) -> Self {
        self.color = Some(color);
        self
    }

    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct Warning<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Warning<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            kind: "Warning",
            text,
            content_description: None,
        }
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct Button<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub text: &'a str,
    pub action: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_file_picker: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_multiple_files: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Button<'a> {
    pub fn new(text: &'a str, action: &'a str) -> Self {
        Self {
            kind: "Button",
            text,
            action,
            copy_text: None,
            id: None,
            requires_file_picker: None,
            allow_multiple_files: None,
            payload: None,
            content_description: None,
        }
    }

    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn requires_file_picker(mut self, needs: bool) -> Self {
        self.requires_file_picker = Some(needs);
        self
    }

    #[allow(dead_code)]
    pub fn allow_multiple_files(mut self, allow: bool) -> Self {
        self.allow_multiple_files = Some(allow);
        self
    }

    pub fn payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn copy_text(mut self, text: &'a str) -> Self {
        self.copy_text = Some(text);
        self
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct Column<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scrollable: Option<bool>,
    pub children: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Column<'a> {
    pub fn new(children: Vec<serde_json::Value>) -> Self {
        Self {
            kind: "Column",
            padding: None,
            scrollable: None,
            children,
            id: None,
            content_description: None,
        }
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = Some(scrollable);
        self
    }

    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct Row<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<u32>,
    pub children: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

#[allow(dead_code)]
impl<'a> Row<'a> {
    pub fn new(children: Vec<serde_json::Value>) -> Self {
        Self {
            kind: "Row",
            padding: None,
            children,
            id: None,
            content_description: None,
        }
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct VirtualList<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub children: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_item_height: Option<u32>,
}

impl<'a> VirtualList<'a> {
    pub fn new(children: Vec<serde_json::Value>) -> Self {
        Self {
            kind: "VirtualList",
            children,
            id: None,
            estimated_item_height: None,
        }
    }

    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn estimated_item_height(mut self, height: u32) -> Self {
        self.estimated_item_height = Some(height);
        self
    }
}

#[derive(Serialize)]
pub struct Section<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub children: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Section<'a> {
    pub fn new(children: Vec<serde_json::Value>) -> Self {
        Self {
            kind: "Section",
            children,
            title: None,
            subtitle: None,
            icon: None,
            padding: None,
            content_description: None,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    #[allow(dead_code)]
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = Some(padding);
        self
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct Card<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub children: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Card<'a> {
    pub fn new(children: Vec<serde_json::Value>) -> Self {
        Self {
            kind: "Card",
            children,
            title: None,
            subtitle: None,
            icon: None,
            padding: None,
            content_description: None,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    #[allow(dead_code)]
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    #[allow(dead_code)]
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = Some(padding);
        self
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct Grid<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub children: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

#[allow(dead_code)]
impl<'a> Grid<'a> {
    pub fn new(children: Vec<serde_json::Value>) -> Self {
        Self {
            kind: "Grid",
            children,
            columns: None,
            padding: None,
            content_description: None,
        }
    }

    pub fn columns(mut self, cols: u32) -> Self {
        self.columns = Some(cols);
        self
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = Some(padding);
        self
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct Checkbox<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub text: &'a str,
    pub bind_key: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

#[allow(dead_code)]
impl<'a> Checkbox<'a> {
    pub fn new(text: &'a str, bind_key: &'a str) -> Self {
        Self {
            kind: "Checkbox",
            text,
            bind_key,
            checked: None,
            action: None,
            content_description: None,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn action(mut self, action: &'a str) -> Self {
        self.action = Some(action);
        self
    }

    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct Progress<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Progress<'a> {
    pub fn new() -> Self {
        Self {
            kind: "Progress",
            text: None,
            content_description: None,
        }
    }

    #[allow(dead_code)]
    pub fn text(mut self, text: &'a str) -> Self {
        self.text = Some(text);
        self
    }

    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct TextInput<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub bind_key: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_on_submit: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_line: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_lines: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debounce_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_mask: Option<bool>,
}

impl<'a> TextInput<'a> {
    pub fn new(bind_key: &'a str) -> Self {
        Self {
            kind: "TextInput",
            bind_key,
            text: None,
            hint: None,
            action_on_submit: None,
            content_description: None,
            single_line: None,
            max_lines: None,
            debounce_ms: None,
            password_mask: None,
        }
    }

    pub fn text(mut self, value: &'a str) -> Self {
        self.text = Some(value);
        self
    }

    pub fn hint(mut self, value: &'a str) -> Self {
        self.hint = Some(value);
        self
    }

    pub fn action_on_submit(mut self, value: &'a str) -> Self {
        self.action_on_submit = Some(value);
        self
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, value: &'a str) -> Self {
        self.content_description = Some(value);
        self
    }

    #[allow(dead_code)]
    pub fn single_line(mut self, value: bool) -> Self {
        self.single_line = Some(value);
        self
    }

    #[allow(dead_code)]
    pub fn max_lines(mut self, value: u32) -> Self {
        self.max_lines = Some(value);
        self
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn debounce_ms(mut self, value: u32) -> Self {
        self.debounce_ms = Some(value);
        self
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn password_mask(mut self, value: bool) -> Self {
        self.password_mask = Some(value);
        self
    }
}

#[derive(Serialize)]
pub struct ImageBase64<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub base64: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> ImageBase64<'a> {
    pub fn new(base64: &'a str) -> Self {
        Self {
            kind: "ImageBase64",
            base64,
            content_description: None,
        }
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct ColorSwatch {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub color: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'static str>,
}

impl ColorSwatch {
    pub fn new(color: i64) -> Self {
        Self {
            kind: "ColorSwatch",
            color,
            content_description: None,
        }
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'static str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct PdfPagePicker<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub page_count: u32,
    pub bind_key: &'a str,
    pub source_uri: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_pages: Option<&'a [u32]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> PdfPagePicker<'a> {
    pub fn new(page_count: u32, bind_key: &'a str, source_uri: &'a str) -> Self {
        Self {
            kind: "PdfPagePicker",
            page_count,
            bind_key,
            source_uri,
            selected_pages: None,
            content_description: None,
        }
    }

    #[allow(dead_code)]
    pub fn selected_pages(mut self, pages: &'a [u32]) -> Self {
        self.selected_pages = Some(pages);
        self
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct Compass<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub angle_radians: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Compass<'a> {
    pub fn new(angle_radians: f64) -> Self {
        Self {
            kind: "Compass",
            angle_radians,
            content_description: None,
        }
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct Barometer<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub hpa: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Barometer<'a> {
    pub fn new(hpa: f64) -> Self {
        Self {
            kind: "Barometer",
            hpa,
            content_description: None,
        }
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct Magnetometer<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub magnitude_ut: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Magnetometer<'a> {
    pub fn new(magnitude_ut: f64) -> Self {
        Self {
            kind: "Magnetometer",
            magnitude_ut,
            content_description: None,
        }
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct DepsList<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

#[allow(dead_code)]
impl<'a> DepsList<'a> {
    pub fn new() -> Self {
        Self {
            kind: "DepsList",
            query: None,
            content_description: None,
        }
    }

    pub fn query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

#[derive(Serialize)]
pub struct CodeView<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_numbers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<&'a str>,
}

impl<'a> CodeView<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            kind: "CodeView",
            text,
            language: None,
            wrap: None,
            theme: None,
            line_numbers: None,
            content_description: None,
            id: None,
        }
    }

    pub fn language(mut self, lang: &'a str) -> Self {
        self.language = Some(lang);
        self
    }

    #[allow(dead_code)]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
        self
    }

    #[allow(dead_code)]
    pub fn theme(mut self, theme: &'a str) -> Self {
        self.theme = Some(theme);
        self
    }

    #[allow(dead_code)]
    pub fn line_numbers(mut self, enabled: bool) -> Self {
        self.line_numbers = Some(enabled);
        self
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }

    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }
}

#[derive(Serialize)]
pub struct HtmlView<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub html: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_dp: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

#[derive(Serialize)]
pub struct Ruler {
    #[serde(rename = "type")]
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_dp: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_height: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'static str>,
}

impl Ruler {
    pub fn new() -> Self {
        Self {
            kind: "Ruler",
            orientation: None,
            height_dp: None,
            fill_height: None,
            content_description: None,
        }
    }

    #[allow(dead_code)]
    pub fn orientation(mut self, value: &'static str) -> Self {
        self.orientation = Some(value);
        self
    }

    #[allow(dead_code)]
    pub fn height_dp(mut self, value: u32) -> Self {
        self.height_dp = Some(value);
        self
    }

    pub fn fill_height(mut self, value: bool) -> Self {
        self.fill_height = Some(value);
        self
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, value: &'static str) -> Self {
        self.content_description = Some(value);
        self
    }
}

impl<'a> HtmlView<'a> {
    pub fn new(html: &'a str) -> Self {
        Self {
            kind: "HtmlView",
            html,
            height_dp: None,
            content_description: None,
        }
    }

    #[allow(dead_code)]
    pub fn height_dp(mut self, value: u32) -> Self {
        self.height_dp = Some(value);
        self
    }

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

pub fn maybe_push_back(children: &mut Vec<Value>, state: &AppState) {
    if state.nav_depth() > 1 {
        children.push(json!({
            "type": "Button",
            "text": t!("button_back"),
            "action": "back"
        }));
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    if bytes as f64 >= MB {
        format!("{:.1} MB", bytes as f64 / MB)
    } else if bytes as f64 >= KB {
        format!("{:.1} KB", bytes as f64 / KB)
    } else {
        format!("{bytes} B")
    }
}

pub fn render_multi_hash_screen(state: &AppState) -> Value {
    let mut children = vec![
        to_value_or_text(
            Text::new(&t!("multi_hash_title")).size(20.0),
            "multi_hash_title",
        ),
        to_value_or_text(
            Text::new(&t!("multi_hash_subtitle")).size(14.0),
            "multi_hash_subtitle",
        ),
        json!({
            "type": "Button",
            "text": t!("multi_hash_pick_file_button"),
            "action": "hash_all", // New action for multi-hash
            "requires_file_picker": true,
            "id": "pick_file_to_hash_btn",
            "content_description": t!("multi_hash_pick_file_description")
        }),
    ];

    if let Some(err) = &state.multi_hash_error {
        children.push(to_value_or_text(
            Text::new(&format!("{}{}", t!("multi_hash_error_prefix"), err)).size(14.0),
            "multi_hash_error",
        ));
    }

    if let Some(results) = &state.multi_hash_results {
        children.push(to_value_or_text(
            Text::new(&format!("{}{}", t!("multi_hash_hashed_file_prefix"), results.file_path)).size(12.0),
            "multi_hash_path",
        ));

        let hash_display = |label: &str, value: &str| {
            json!({
                "type": "Column",
                "padding": 8,
                "children": [
                    to_value_or_text(Text::new(label).size(12.0), "multi_hash_label"),
                    to_value_or_text(Text::new(value).size(10.0), "multi_hash_value"),
                    to_value_or_text(Button::new(&t!("button_copy"), "noop").copy_text(value), "multi_hash_copy"),
                ]
            })
        };

        children.push(hash_display(&t!("multi_hash_label_md5"), &results.md5));
        children.push(hash_display(&t!("multi_hash_label_sha1"), &results.sha1));
        children.push(hash_display(&t!("multi_hash_label_sha256"), &results.sha256));
        children.push(hash_display(&t!("multi_hash_label_blake3"), &results.blake3));
    }

    to_value_or_text(Column::new(children).padding(24), "multi_hash_root")
}

fn to_value_or_text<T: Serialize>(value: T, context: &str) -> Value {
    serde_json::to_value(value).unwrap_or_else(|e| {
        json!({
            "type": "Text",
            "text": format!("{context}_serialize_error:{e}")
        })
    })
}

