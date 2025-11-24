use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Screen {
    Home,
    ShaderDemo,
}

pub struct AppState {
    pub counter: i32,
    pub current_screen: Screen,
    pub last_hash: Option<String>,
    pub last_error: Option<String>,
    pub last_shader: Option<String>,
    pub last_hash_algo: Option<String>,
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
        }
    }
}
