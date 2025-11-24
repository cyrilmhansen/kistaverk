pub mod file_info;
pub mod hashes;
pub mod kotlin_image;
pub mod color_tools;
pub mod qr;
pub mod text_tools;
pub mod pdf;

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
    use std::collections::BTreeMap;
    use crate::ui::{Button as UiButton, Column as UiColumn, Grid as UiGrid, Text as UiText};

    let mut children = vec![
        serde_json::to_value(UiText::new("🧰 Tool menu").size(22.0)).unwrap(),
        serde_json::to_value(UiText::new("✨ Select a tool. Hash tools prompt for a file.").size(14.0)).unwrap(),
    ];

    let mut grouped: BTreeMap<&str, Vec<&Feature>> = BTreeMap::new();
    for feature in catalog.iter() {
        grouped.entry(feature.category).or_default().push(feature);
    }

    for (category, feats) in grouped {
        children.push(serde_json::to_value(UiText::new(category).size(16.0)).unwrap());
        let cards: Vec<Value> = feats
            .iter()
            .map(|f| {
                serde_json::to_value(
                    UiButton::new(&format!("{} – {}", f.name, f.description), f.action)
                        .id(f.id)
                        .requires_file_picker(f.requires_file_picker),
                )
                .unwrap()
            })
            .collect();
        children.push(
            serde_json::to_value(UiGrid::new(cards).columns(2).padding(8)).unwrap(),
        );
    }

    if let Some(hash) = &state.last_hash {
        children.push(
            serde_json::to_value(
                UiText::new(&format!(
                    "{}: {}",
                    state
                        .last_hash_algo
                        .clone()
                        .unwrap_or_else(|| "Hash".into()),
                    hash
                ))
                .size(14.0),
            )
            .unwrap(),
        );
    }

    if let Some(err) = &state.last_error {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Error: {}", err))
                    .size(14.0)
                    .content_description("error_text"),
            )
            .unwrap(),
        );
    }

    serde_json::to_value(UiColumn::new(children).padding(32)).unwrap()
}
