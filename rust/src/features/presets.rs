use crate::features::storage::preferred_temp_dir;
use crate::state::{AppState, Screen};
use crate::ui::{Button, Column, Text, maybe_push_back};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub tool_id: String,
    pub data: Value,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresetState {
    pub presets: Vec<Preset>,
    pub current_tool_id: Option<String>,
    pub name_input: String,
    pub is_saving: bool,
    pub error: Option<String>,
    pub last_message: Option<String>,
}

impl PresetState {
    pub const fn new() -> Self {
        Self {
            presets: Vec::new(),
            current_tool_id: None,
            name_input: String::new(),
            is_saving: false,
            error: None,
            last_message: None,
        }
    }

    pub fn reset(&mut self) {
        self.presets.clear();
        self.current_tool_id = None;
        self.name_input.clear();
        self.is_saving = false;
        self.error = None;
        self.last_message = None;
    }
}

pub fn presets_dir() -> PathBuf {
    let mut path = preferred_temp_dir();
    // Go up one level from "tmp" to get to the app's cache/files root, then into "presets"
    if let Some(parent) = path.parent() {
        path = parent.to_path_buf();
    }
    path.push("presets");
    path
}

pub fn load_presets() -> Result<Vec<Preset>, String> {
    let dir = presets_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut presets = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| format!("read_dir_failed:{e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("entry_error:{e}"))?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "json") {
            let content = fs::read_to_string(&path).map_err(|e| format!("read_failed:{e}"))?;
            match serde_json::from_str::<Preset>(&content) {
                Ok(p) => presets.push(p),
                Err(_) => {
                    // Ignore malformed files
                }
            }
        }
    }

    // Sort by newest first
    presets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(presets)
}

pub fn save_preset(tool_id: &str, name: &str, data: Value) -> Result<Preset, String> {
    let dir = presets_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir_failed:{e}"))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("clock_err:{e:?}"))?;
    let id = format!("{}_{}", tool_id, now.as_millis());
    let preset = Preset {
        id: id.clone(),
        name: name.to_string(),
        tool_id: tool_id.to_string(),
        data,
        created_at: now.as_secs(),
    };

    let path = dir.join(format!("{}.json", id));
    let content = serde_json::to_string_pretty(&preset).map_err(|e| format!("json_err:{e}"))?;
    fs::write(&path, content).map_err(|e| format!("write_failed:{e}"))?;

    Ok(preset)
}

pub fn delete_preset(id: &str) -> Result<(), String> {
    let dir = presets_dir();
    let path = dir.join(format!("{}.json", id));
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("delete_failed:{e}"))?;
    }
    Ok(())
}

pub fn render_preset_manager(state: &AppState) -> Value {
    let mut children = vec![to_value_or_text(Text::new("Presets").size(20.0), "presets_title")];

    if let Some(tool) = &state.preset_state.current_tool_id {
        children.push(to_value_or_text(
            Text::new(&format!("Managing presets for: {}", tool)).size(14.0),
            "presets_tool",
        ));
    }

    if let Some(msg) = &state.preset_state.last_message {
        children.push(to_value_or_text(
            Text::new(msg).size(12.0),
            "presets_message",
        ));
    }
    if let Some(err) = &state.preset_state.error {
        children.push(to_value_or_text(
            Text::new(&format!("Error: {}", err)).size(12.0),
            "presets_error",
        ));
    }

    let filtered: Vec<&Preset> = if let Some(tid) = &state.preset_state.current_tool_id {
        state.preset_state.presets.iter().filter(|p| &p.tool_id == tid).collect()
    } else {
        state.preset_state.presets.iter().collect()
    };

    if filtered.is_empty() {
        children.push(to_value_or_text(
            Text::new("No presets found.").size(14.0),
            "presets_empty",
        ));
    } else {
        for preset in filtered {
            let mut row_items = vec![
                to_value_or_text(Text::new(&preset.name).size(16.0), "preset_name"),
                to_value_or_text(
                    Text::new(&format!("({})", preset.tool_id)).size(10.0),
                    "preset_tool",
                ),
            ];
            
            let load_btn = Button::new("Load", "preset_load")
                .payload(json!({ "id": preset.id }));
            row_items.push(to_value_or_text(load_btn, "preset_load_btn"));

            let del_btn = Button::new("Delete", "preset_delete")
                .payload(json!({ "id": preset.id }));
            row_items.push(to_value_or_text(del_btn, "preset_delete_btn"));

            children.push(json!({
                "type": "Card",
                "child": {
                    "type": "Column",
                    "children": row_items
                },
                "padding": 8
            }));
        }
    }

    children.push(
        to_value_or_text(Button::new("Create New Preset", "preset_save_dialog"), "preset_create_btn"),
    );

    maybe_push_back(&mut children, state);
    to_value_or_text(Column::new(children).padding(16), "presets_root")
}

pub fn render_save_preset_dialog(state: &AppState) -> Value {
    let mut children = vec![
        to_value_or_text(Text::new("Save Preset").size(20.0), "presets_save_title"),
        to_value_or_text(
            Text::new("Enter a name for your preset:").size(14.0),
            "presets_save_subtitle",
        ),
        json!({
            "type": "Input",
            "bind_key": "preset_name",
            "hint": "Preset Name",
            "value": state.preset_state.name_input
        }),
    ];

    if let Some(err) = &state.preset_state.error {
        children.push(to_value_or_text(
            Text::new(&format!("Error: {}", err)).size(12.0),
            "presets_save_error",
        ));
    }

    children.push(to_value_or_text(Button::new("Save", "preset_save"), "presets_save_btn"));

    maybe_push_back(&mut children, state);
    to_value_or_text(Column::new(children).padding(16), "presets_save_root")
}

fn to_value_or_text<T: Serialize>(value: T, context: &str) -> Value {
    serde_json::to_value(value).unwrap_or_else(|e| {
        json!({
            "type": "Text",
            "text": format!("{context}_serialize_error:{e}")
        })
    })
}

// Helper to extract state for saving
pub fn preset_payload_for_tool(state: &AppState, tool_id: &str) -> Result<Value, String> {
    match tool_id {
        "dithering" => Ok(json!({
            "mode": state.dithering_mode,
            "palette": state.dithering_palette
        })),
        "pixel_art" => Ok(json!({
            "scale_factor": state.pixel_art.scale_factor
        })),
        _ => Err(format!("Tool '{}' does not support presets", tool_id)),
    }
}

// Helper to apply state from preset
pub fn apply_preset_to_state(state: &mut AppState, preset: &Preset) -> Result<(), String> {
    if preset.tool_id == "dithering" {
        state.dithering_mode = serde_json::from_value(preset.data["mode"].clone())
            .map_err(|e| format!("bad_mode:{e}"))?;
        state.dithering_palette = serde_json::from_value(preset.data["palette"].clone())
            .map_err(|e| format!("bad_palette:{e}"))?;
        Ok(())
    } else if preset.tool_id == "pixel_art" {
        state.pixel_art.scale_factor = serde_json::from_value(preset.data["scale_factor"].clone())
            .map_err(|e| format!("bad_scale:{e}"))?;
        Ok(())
    } else {
        Err(format!("Unknown tool id in preset: {}", preset.tool_id))
    }
}

pub fn tool_id_for_screen(screen: Screen) -> Option<&'static str> {
    match screen {
        Screen::Dithering => Some("dithering"),
        Screen::PixelArt => Some("pixel_art"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{DitheringMode, DitheringPalette};

    #[test]
    fn test_preset_payload_dithering() {
        let mut state = AppState::new();
        state.dithering_mode = DitheringMode::Bayer8x8;
        state.dithering_palette = DitheringPalette::GameBoy;
        
        let payload = preset_payload_for_tool(&state, "dithering").unwrap();
        assert_eq!(payload["mode"], json!(DitheringMode::Bayer8x8));
        assert_eq!(payload["palette"], json!(DitheringPalette::GameBoy));
    }

    #[test]
    fn test_apply_preset_dithering() {
        let mut state = AppState::new();
        let preset = Preset {
            id: "test".into(),
            name: "Test".into(),
            tool_id: "dithering".into(),
            data: json!({
                "mode": DitheringMode::Sierra,
                "palette": DitheringPalette::Cga
            }),
            created_at: 0,
        };
        
        apply_preset_to_state(&mut state, &preset).unwrap();
        assert_eq!(state.dithering_mode, DitheringMode::Sierra);
        assert_eq!(state.dithering_palette, DitheringPalette::Cga);
    }

    #[test]
    fn test_persistence_cycle() {
        use std::env;
        use tempfile::tempdir;

        // Setup a mock directory structure: /tmp/mock_app/cache
        let root_dir = tempdir().expect("failed to create temp dir");
        let cache_dir = root_dir.path().join("cache");
        fs::create_dir(&cache_dir).expect("failed to create cache dir");
        
        // Set the env var so preferred_temp_dir returns our mock cache
        // unsafe block needed for set_var in some contexts, but here it is standard lib
        env::set_var("KISTAVERK_TEMP_DIR", &cache_dir);
        
        let tool_id = "test_tool";
        let name = "Test Preset";
        let data = json!({"foo": "bar"});
        
        // 1. Save
        let saved = save_preset(tool_id, name, data.clone()).expect("save failed");
        assert_eq!(saved.name, name);
        assert_eq!(saved.tool_id, tool_id);
        
        // 2. Load
        let all = load_presets().expect("load failed");
        let loaded = all.iter().find(|p| p.id == saved.id).expect("preset not found");
        assert_eq!(loaded.data, data);
        
        // 3. Delete
        delete_preset(&saved.id).expect("delete failed");
        
        // 4. Verify deleted
        let all_after = load_presets().expect("load failed");
        assert!(all_after.iter().find(|p| p.id == saved.id).is_none());

        env::remove_var("KISTAVERK_TEMP_DIR");
    }
}
