pub mod hashes;
pub mod kotlin_image;
pub mod file_info;

use crate::state::AppState;
use serde_json::Value;

/// A feature entry for the home menu.
pub struct Feature {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub action: &'static str,
    pub requires_file_picker: bool,
    pub description: &'static str,
}

/// Render the home screen using a catalog of features.
pub fn render_menu(state: &AppState, catalog: &[Feature]) -> Value {
    use serde_json::json;
    use std::collections::BTreeMap;

    let mut children = vec![
        json!({
            "type": "Text",
            "text": "🧰 Tool menu",
            "size": 22.0
        }),
        json!({
            "type": "Text",
            "text": "✨ Select a tool. Hash tools prompt for a file.",
            "size": 14.0
        }),
    ];

    let mut grouped: BTreeMap<&str, Vec<&Feature>> = BTreeMap::new();
    for feature in catalog.iter() {
        grouped.entry(feature.category).or_default().push(feature);
    }

    for (category, feats) in grouped {
        children.push(json!({
            "type": "Text",
            "text": category,
            "size": 16.0
        }));
        for f in feats {
            children.push(json!({
                "type": "Button",
                "id": f.id,
                "text": format!("{} – {}", f.name, f.description),
                "action": f.action,
                "requires_file_picker": f.requires_file_picker
            }));
        }
    }

    if let Some(hash) = &state.last_hash {
        children.push(json!({
            "type": "Text",
            "text": format!("{}: {}", state.last_hash_algo.clone().unwrap_or_else(|| "Hash".into()), hash),
            "size": 14.0
        }));
    }

    if let Some(err) = &state.last_error {
        children.push(json!({
            "type": "Text",
            "text": format!("Error: {}", err),
            "size": 14.0
        }));
    }

    json!({
        "type": "Column",
        "padding": 32,
        "children": children
    })
}
