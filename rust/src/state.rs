use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Screen {
    Home,
    // Add other screens here later
}

pub struct AppState {
    pub counter: i32,
    pub current_screen: Screen,
}

impl AppState {
    // Add 'const' here so it can be used in static initialization
    pub const fn new() -> Self {
        Self {
            counter: 0,
            current_screen: Screen::Home,
        }
    }
}