pub mod archive;
pub mod color_tools;
pub mod file_info;
pub mod hashes;
pub mod kotlin_image;
pub mod pdf;
pub mod qr;
pub mod sensor_logger;
pub mod text_tools;
pub mod text_viewer;

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
    use crate::ui::{
        Button as UiButton, Card as UiCard, Column as UiColumn, Section as UiSection,
        Text as UiText,
    };
    use std::collections::BTreeMap;

    let mut children = vec![
        serde_json::to_value(UiText::new("🧰 Tool menu").size(22.0)).unwrap(),
        serde_json::to_value(
            UiText::new("✨ Select a tool. Hash tools prompt for a file.").size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiText::new("Legacy notice: MD5 and SHA-1 are not suitable for security; prefer SHA-256 or BLAKE3.")
                .size(12.0),
        )
        .unwrap(),
    ];

    // Quick access row (static for now; prefer high-traffic tools).
    let quick_ids = ["pdf_tools", "text_tools", "text_viewer", "hash_sha256"];
    let quick_buttons: Vec<Value> = catalog
        .iter()
        .filter(|f| quick_ids.contains(&f.id))
        .map(|f| {
            serde_json::to_value(
                UiButton::new(f.name, f.action)
                    .id(f.id)
                    .requires_file_picker(f.requires_file_picker),
            )
            .unwrap()
        })
        .collect();
    if !quick_buttons.is_empty() {
        let quick = UiCard::new(vec![
            serde_json::to_value(UiColumn::new(quick_buttons)).unwrap()
        ])
        .title("⚡ Quick access")
        .padding(12);
        children.push(serde_json::to_value(quick).unwrap());
    }

    let mut grouped: BTreeMap<&str, Vec<&Feature>> = BTreeMap::new();
    for feature in catalog.iter() {
        grouped.entry(feature.category).or_default().push(feature);
    }

    for (category, feats) in grouped {
        let mut section_children: Vec<Value> = Vec::new();
        if category.contains("Hash") {
            section_children.push(
                serde_json::to_value(
                    UiText::new("MD5/SHA-1 are legacy. Prefer SHA-256 or BLAKE3.").size(12.0),
                )
                .unwrap(),
            );
        }
        let list: Vec<Value> = feats
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
        section_children.push(
            serde_json::to_value(UiColumn::new(list).padding(4).content_description(category))
                .unwrap(),
        );

        let subtitle = format!("{} tools", feats.len());
        let mut section = UiSection::new(section_children)
            .title(category)
            .subtitle(&subtitle)
            .padding(12);
        if let Some(first) = category.split_whitespace().next() {
            if first.chars().all(|c| !c.is_ascii_alphanumeric()) {
                section = section.icon(first);
            }
        }
        children.push(serde_json::to_value(section).unwrap());
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
        children.push(
            serde_json::to_value(
                UiButton::new("Copy last hash", "noop")
                    .copy_text(hash)
                    .id("copy_last_hash_home"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Paste reference (clipboard)", "hash_paste_reference")
                    .id("hash_paste_reference_btn"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Show QR for last hash", "hash_qr_last")
                    .id("hash_qr_last_btn"),
            )
            .unwrap(),
        );
        if let Some(matches) = state.hash_match {
            let status = if matches {
                "Reference match ✅"
            } else {
                "Reference mismatch ❌"
            };
            children.push(
                serde_json::to_value(
                    UiText::new(status)
                        .size(12.0)
                        .content_description("hash_ref_status"),
                )
                .unwrap(),
            );
        }
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
