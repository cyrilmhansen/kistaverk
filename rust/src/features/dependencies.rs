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
        let needle_lower = needle.to_ascii_lowercase();
        let haystack = format!(
            "{} {} {} {} {}",
            self.name,
            self.version.as_deref().unwrap_or(""),
            self.license.as_deref().unwrap_or(""),
            self.repository.as_deref().unwrap_or(""),
            self.homepage.as_deref().unwrap_or(""),
        )
        .to_ascii_lowercase();
        haystack.contains(&needle_lower)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::DependencyState;

    #[test]
    fn test_dependency_matching() {
        let dep = DependencyEntry {
            name: "test-dep".to_string(),
            version: Some("1.0.0".to_string()),
            license: Some("MIT".to_string()),
            repository: Some("https://github.com/test/repo".to_string()),
            homepage: Some("https://test.com".to_string()),
        };

        // Should match empty query
        assert!(dep.matches(""));

        // Should match name
        assert!(dep.matches("test-dep"));
        assert!(dep.matches("TEST-DEP"));

        // Should match version
        assert!(dep.matches("1.0.0"));

        // Should match license
        assert!(dep.matches("MIT"));

        // Should match repository
        assert!(dep.matches("github"));

        // Should match homepage
        assert!(dep.matches("test.com"));

        // Should not match unrelated text
        assert!(!dep.matches("unrelated"));
    }

    #[test]
    fn test_dependency_display_name() {
        let dep_with_version = DependencyEntry {
            name: "test".to_string(),
            version: Some("1.0.0".to_string()),
            license: None,
            repository: None,
            homepage: None,
        };
        assert_eq!(dep_with_version.display_name(), "test 1.0.0");

        let dep_without_version = DependencyEntry {
            name: "test".to_string(),
            version: Some("".to_string()),
            license: None,
            repository: None,
            homepage: None,
        };
        assert_eq!(dep_without_version.display_name(), "test");

        let dep_none_version = DependencyEntry {
            name: "test".to_string(),
            version: None,
            license: None,
            repository: None,
            homepage: None,
        };
        assert_eq!(dep_none_version.display_name(), "test");
    }

    #[test]
    fn test_dependency_license_label() {
        let dep_with_license = DependencyEntry {
            name: "test".to_string(),
            version: None,
            license: Some("MIT".to_string()),
            repository: None,
            homepage: None,
        };
        assert_eq!(dep_with_license.license_label(), "MIT");

        let dep_without_license = DependencyEntry {
            name: "test".to_string(),
            version: None,
            license: None,
            repository: None,
            homepage: None,
        };
        assert_eq!(dep_without_license.license_label(), "unknown");
    }

    #[test]
    fn test_group_dependencies() {
        let deps = vec![
            DependencyEntry {
                name: "dep1".to_string(),
                version: Some("1.0".to_string()),
                license: Some("MIT".to_string()),
                repository: None,
                homepage: None,
            },
            DependencyEntry {
                name: "dep2".to_string(),
                version: Some("2.0".to_string()),
                license: Some("MIT".to_string()),
                repository: None,
                homepage: None,
            },
            DependencyEntry {
                name: "dep3".to_string(),
                version: Some("3.0".to_string()),
                license: Some("Apache-2.0".to_string()),
                repository: None,
                homepage: None,
            },
        ];

        // Test grouping without query
        let grouped = group_dependencies(&deps, "");
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get("MIT").unwrap().len(), 2);
        assert_eq!(grouped.get("Apache-2.0").unwrap().len(), 1);

        // Test filtering
        let filtered = group_dependencies(&deps, "dep1");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered.get("MIT").unwrap().len(), 1);
        assert_eq!(filtered.get("MIT").unwrap()[0].name, "dep1");

        // Test case insensitive filtering
        let filtered_case = group_dependencies(&deps, "DEP1");
        assert_eq!(filtered_case.len(), 1);
        assert_eq!(filtered_case.get("MIT").unwrap().len(), 1);
    }

    #[test]
    fn test_json_parsing() {
        let json_data = r#"
        {
            "packages": [
                {
                    "name": "serde",
                    "version": "1.0.0",
                    "license": "MIT",
                    "repository": "https://github.com/serde-rs/serde",
                    "homepage": "https://serde.rs"
                },
                {
                    "name": "serde_json",
                    "version": "1.0.0",
                    "license": "MIT"
                }
            ]
        }
        "#;

        let result: DependenciesFile = serde_json::from_str(json_data).unwrap();
        assert_eq!(result.packages.len(), 2);
        assert_eq!(result.packages[0].name, "serde");
        assert_eq!(result.packages[1].name, "serde_json");
    }

    #[test]
    fn test_render_dependencies_list_with_real_data() {
        let state = DependencyState {
            query: "".to_string(),
        };

        let result = render_dependencies_list(&state);
        
        // Should contain a VirtualList
        let result_obj = result.as_object().unwrap();
        assert_eq!(result_obj.get("type").unwrap().as_str().unwrap(), "VirtualList");
        
        let children = result_obj.get("children").unwrap().as_array().unwrap();
        assert!(!children.is_empty());
        
        // Should have estimated item height
        assert_eq!(result_obj.get("estimated_item_height").unwrap().as_u64(), Some(32));
    }

    #[test]
    fn test_render_dependencies_list_with_filter() {
        let state = DependencyState {
            query: "MIT".to_string(),
        };

        let result = render_dependencies_list(&state);
        
        // Should still contain a VirtualList even with filter
        let result_obj = result.as_object().unwrap();
        assert_eq!(result_obj.get("type").unwrap().as_str().unwrap(), "VirtualList");
    }

    #[test]
    fn test_render_dependencies_list_with_query() {
        let state = DependencyState {
            query: "MIT".to_string(),
        };

        let result = render_dependencies_list(&state);
        
        // Should contain a VirtualList
        let result_obj = result.as_object().unwrap();
        assert_eq!(result_obj.get("type").unwrap().as_str().unwrap(), "VirtualList");
    }
}
