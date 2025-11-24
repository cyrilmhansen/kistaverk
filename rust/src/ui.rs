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
            requires_file_picker: None,
            content_description: None,
        }
    }

    pub fn requires_file_picker(mut self, needs: bool) -> Self {
        self.requires_file_picker = Some(needs);
        self
    }

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
    pub children: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_description: Option<&'a str>,
}

impl<'a> Column<'a> {
    pub fn new(children: Vec<serde_json::Value>) -> Self {
        Self {
            kind: "Column",
            padding: None,
            children,
            content_description: None,
        }
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = Some(padding);
        self
    }

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

    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}

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

    pub fn text(mut self, text: &'a str) -> Self {
        self.text = Some(text);
        self
    }

    pub fn content_description(mut self, cd: &'a str) -> Self {
        self.content_description = Some(cd);
        self
    }
}
