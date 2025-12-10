use crate::state::DependencyState;
use crate::ui::{Column as UiColumn, Text as UiText, VirtualList as UiVirtualList};
use serde::Deserialize;
use serde_json::{to_value, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

const DEPS_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../app/app/src/main/assets/deps.json"));

#[derive(Debug, Clone, Deserialize)]
struct DependenciesFile {
    packages: Vec<DependencyEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct DependencyEntry {
    name: String,
    version: Option<String>,
    license: Option<String>,
    repository: Option<String>,
    homepage: Option<String>,
}

impl DependencyEntry {
    fn display_name(&self) -> String {
        match self.version.as_deref() {
            Some(v) if !v.is_empty() => format!("{} {}", self.name, v),
            _ => self.name.clone(),
        }
    }

    fn license_label(&self) -> &str {
        self.license.as_deref().unwrap_or("unknown")
    }

    fn matches(&self, needle: &str) -> bool {
        if needle.is_empty() {
            return true;
        }
        let haystack = format!(
            "{} {} {} {} {}",
            self.name,
            self.version.as_deref().unwrap_or(""),
            self.license.as_deref().unwrap_or(""),
            self.repository.as_deref().unwrap_or(""),
            self.homepage.as_deref().unwrap_or(""),
        )
        .to_ascii_lowercase();
        haystack.contains(needle)
    }
}

static CACHED_DEPENDENCIES: OnceLock<Vec<DependencyEntry>> = OnceLock::new();

fn load_dependencies() -> &'static [DependencyEntry] {
    CACHED_DEPENDENCIES.get_or_init(|| {
        serde_json::from_str::<DependenciesFile>(DEPS_JSON)
            .map(|data| data.packages)
            .unwrap_or_default()
    })
}

fn group_dependencies<'a>(
    deps: &'a [DependencyEntry],
    query: &str,
) -> BTreeMap<String, Vec<&'a DependencyEntry>> {
    let needle = query.trim().to_ascii_lowercase();
    let mut grouped: BTreeMap<String, Vec<&DependencyEntry>> = BTreeMap::new();
    for dep in deps.iter().filter(|d| d.matches(&needle)) {
        grouped
            .entry(dep.license_label().to_string())
            .or_default()
            .push(dep);
    }
    grouped
}

pub fn render_dependencies_list(state: &DependencyState) -> Value {
    let deps = load_dependencies();
    let grouped = group_dependencies(deps, &state.query);

    let mut items: Vec<Value> = Vec::new();

    if deps.is_empty() {
        items.push(
            to_value(UiText::new("Dependencies unavailable").size(12.0)).unwrap(),
        );
    } else if grouped.is_empty() {
        let query = state.query.trim();
        let message = if query.is_empty() {
            "No dependencies available".to_string()
        } else {
            format!("No dependencies match \"{query}\"")
        };
        items.push(to_value(UiText::new(&message).size(12.0)).unwrap());
    } else {
        for (license, mut entries) in grouped {
            entries.sort_by(|a, b| a.display_name().cmp(&b.display_name()));
            let mut section_children = vec![to_value(
                UiText::new(&format!("{license} ({})", entries.len())).size(14.0),
            )
            .unwrap()];
            for dep in entries {
                section_children
                    .push(to_value(UiText::new(&format!("• {}", dep.display_name())).size(12.0)).unwrap());
            }
            items.push(
                to_value(UiColumn::new(section_children).padding(8)).unwrap(),
            );
        }
    }

    to_value(UiVirtualList::new(items).estimated_item_height(32)).unwrap()
}
