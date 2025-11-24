use crate::features::kotlin_image::KotlinImageState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Screen {
    Home,
    ShaderDemo,
    KotlinImage,
    FileInfo,
    TextTools,
    Loading,
    ProgressDemo,
}

pub struct AppState {
    pub counter: i32,
    pub nav_stack: Vec<Screen>,
    pub last_hash: Option<String>,
    pub last_error: Option<String>,
    pub last_shader: Option<String>,
    pub last_hash_algo: Option<String>,
    pub image: KotlinImageState,
    pub last_file_info: Option<String>,
    pub text_input: Option<String>,
    pub text_output: Option<String>,
    pub text_operation: Option<String>,
    pub text_aggressive_trim: bool,
    pub loading_message: Option<String>,
    pub progress_status: Option<String>,
    pub loading_with_spinner: bool,
}

impl AppState {
    // const so it can be used in static initialization
    pub const fn new() -> Self {
        Self {
            counter: 0,
            nav_stack: Vec::new(),
            last_hash: None,
            last_error: None,
            last_shader: None,
            last_hash_algo: None,
            image: KotlinImageState::new(),
            last_file_info: None,
            text_input: None,
            text_output: None,
            text_operation: None,
            text_aggressive_trim: false,
            loading_message: None,
            progress_status: None,
            loading_with_spinner: true,
        }
    }

    pub fn ensure_navigation(&mut self) {
        if self.nav_stack.is_empty() {
            self.nav_stack.push(Screen::Home);
        }
    }

    pub fn current_screen(&self) -> Screen {
        self.nav_stack
            .last()
            .cloned()
            .unwrap_or(Screen::Home)
    }

    pub fn nav_depth(&self) -> usize {
        let depth = self.nav_stack.len();
        if depth == 0 { 1 } else { depth }
    }

    pub fn push_screen(&mut self, screen: Screen) {
        self.ensure_navigation();
        self.nav_stack.push(screen);
    }

    pub fn replace_current(&mut self, screen: Screen) {
        self.ensure_navigation();
        if let Some(last) = self.nav_stack.last_mut() {
            *last = screen;
        } else {
            self.nav_stack.push(screen);
        }
    }

    pub fn pop_screen(&mut self) {
        self.ensure_navigation();
        if self.nav_stack.len() > 1 {
            self.nav_stack.pop();
        }
    }

    pub fn reset_navigation(&mut self) {
        self.nav_stack.clear();
        self.nav_stack.push(Screen::Home);
    }

    pub fn reset_runtime(&mut self) {
        self.counter = 0;
        self.last_hash = None;
        self.last_error = None;
        self.last_shader = None;
        self.last_hash_algo = None;
        self.image.reset();
        self.last_file_info = None;
        self.text_input = None;
        self.text_output = None;
        self.text_operation = None;
        self.text_aggressive_trim = false;
        self.loading_message = None;
        self.progress_status = None;
        self.loading_with_spinner = true;
    }
}
