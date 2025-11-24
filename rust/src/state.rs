use crate::features::kotlin_image::KotlinImageState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Screen {
    Home,
    ShaderDemo,
    KotlinImage,
    FileInfo,
    TextTools,
}

pub struct AppState {
    pub counter: i32,
    pub current_screen: Screen,
    pub last_hash: Option<String>,
    pub last_error: Option<String>,
    pub last_shader: Option<String>,
    pub last_hash_algo: Option<String>,
    pub image: KotlinImageState,
    pub last_file_info: Option<String>,
    pub text_input: Option<String>,
    pub text_output: Option<String>,
    pub text_operation: Option<String>,
}

impl AppState {
    // Add 'const' here so it can be used in static initialization
    pub const fn new() -> Self {
        Self {
            counter: 0,
            current_screen: Screen::Home,
            last_hash: None,
            last_error: None,
            last_shader: None,
            last_hash_algo: None,
            image: KotlinImageState::new(),
            last_file_info: None,
            text_input: None,
            text_output: None,
            text_operation: None,
        }
    }
}
