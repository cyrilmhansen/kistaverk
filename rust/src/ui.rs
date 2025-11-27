use serde::Serialize;

#[derive(Serialize)]
pub struct Text<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Text<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            kind: "Text",
            text,
            size: None,
            content_description: None,
        }
    }

    pub fn size(mut self, size: f64) -> Self {
        self.size = Some(size);
        self
    }

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
    pub content_description: Option<&'a str>,
}

impl<'a> Column<'a> {
    pub fn new(children: Vec<serde_json::Value>) -> Self {
        Self {
            kind: "Column",
            padding: None,
            scrollable: None,
            children,
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

    #[allow(dead_code)]
    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

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
pub struct DepsList<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> DepsList<'a> {
    pub fn new() -> Self {
        Self {
            kind: "DepsList",
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
}
