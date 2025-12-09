use crate::state::AppState;
use crate::ui::{maybe_push_back, Button as UiButton, Column as UiColumn, Text as UiText, TextInput as UiTextInput, VirtualList as UiVirtualList};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::os::fd::FromRawFd;
use std::os::unix::io::RawFd;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogicTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicState {
    pub triples: Vec<LogicTriple>,
    pub query: Option<(String, String, String)>,
    pub results: Vec<LogicTriple>,
    pub import_error: Option<String>,
}

impl LogicState {
    pub const fn new() -> Self {
        Self {
            triples: Vec::new(),
            query: None,
            results: Vec::new(),
            import_error: None,
        }
    }

    pub fn reset(&mut self) {
        self.triples.clear();
        self.query = None;
        self.results.clear();
        self.import_error = None;
    }
}

#[derive(Default)]
struct TripleStore {
    triples: Vec<LogicTriple>,
}

impl TripleStore {
    fn new(triples: Vec<LogicTriple>) -> Self {
        Self { triples }
    }

    fn add(&mut self, subject: String, predicate: String, object: String) {
        self.triples.push(LogicTriple {
            subject,
            predicate,
            object,
        });
    }

    fn query(&self, s: &str, p: &str, o: &str) -> Vec<LogicTriple> {
        let s_any = s.is_empty() || s == "*";
        let p_any = p.is_empty() || p == "*";
        let o_any = o.is_empty() || o == "*";
        self.triples
            .iter()
            .filter(|t| {
                (s_any || t.subject == s)
                    && (p_any || t.predicate == p)
                    && (o_any || t.object == o)
            })
            .cloned()
            .collect()
    }

    fn import_csv(&mut self, content: &str) -> Result<(), String> {
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let cols: Vec<&str> = trimmed.split(',').map(|c| c.trim_matches('"')).collect();
            if cols.len() != 3 {
                return Err(format!("logic_csv_bad_row:{idx}"));
            }
            self.add(
                cols[0].to_string(),
                cols[1].to_string(),
                cols[2].to_string(),
            );
        }
        Ok(())
    }
}

pub fn render_logic_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("Logical Engine").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Add triples, import CSV, and query with simple patterns.")
                .size(14.0),
        )
        .unwrap(),
    ];

    if let Some(err) = &state.logic.import_error {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Import error: {err}"))
                    .size(12.0)
                    .content_description("logic_import_error"),
            )
            .unwrap(),
        );
    }

    children.push(
        serde_json::to_value(UiText::new("Add triple").size(16.0)).unwrap(),
    );
    children.push(
        serde_json::to_value(UiTextInput::new("logic_add_s").hint("Subject")).unwrap(),
    );
    children.push(
        serde_json::to_value(UiTextInput::new("logic_add_p").hint("Predicate")).unwrap(),
    );
    children.push(
        serde_json::to_value(UiTextInput::new("logic_add_o").hint("Object")).unwrap(),
    );
    children.push(
        serde_json::to_value(UiButton::new("Add", "logic_add_triple")).unwrap(),
    );
    children.push(
        serde_json::to_value(
            UiButton::new("Import CSV", "logic_import")
                .requires_file_picker(true)
                .content_description("logic_import_btn"),
        )
        .unwrap(),
    );

    children.push(
        serde_json::to_value(UiText::new("Query (use * for any)").size(16.0)).unwrap(),
    );
    children.push(
        serde_json::to_value(
            UiTextInput::new("logic_query_s")
                .hint("Subject pattern")
                .debounce_ms(150)
                .action_on_submit("logic_query"),
        )
        .unwrap(),
    );
    children.push(
        serde_json::to_value(
            UiTextInput::new("logic_query_p")
                .hint("Predicate pattern")
                .debounce_ms(150)
                .action_on_submit("logic_query"),
        )
        .unwrap(),
    );
    children.push(
        serde_json::to_value(
            UiTextInput::new("logic_query_o")
                .hint("Object pattern")
                .debounce_ms(150)
                .action_on_submit("logic_query"),
        )
        .unwrap(),
    );
    children.push(
        serde_json::to_value(UiButton::new("Run query", "logic_query")).unwrap(),
    );

    if !state.logic.results.is_empty() {
        let rows: Vec<Value> = state
            .logic
            .results
            .iter()
            .map(|t| {
                serde_json::to_value(UiText::new(&format!(
                    "{}  {}  {}",
                    t.subject, t.predicate, t.object
                )))
                .unwrap()
            })
            .collect();
        children.push(
            serde_json::to_value(
                UiVirtualList::new(rows)
                    .estimated_item_height(20)
                    .id("logic_results"),
            )
            .unwrap(),
        );
    }

    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

pub fn add_triple(state: &mut AppState, s: &str, p: &str, o: &str) {
    let mut store = TripleStore::new(state.logic.triples.clone());
    store.add(s.to_string(), p.to_string(), o.to_string());
    state.logic.triples = store.triples;
}

pub fn run_query(state: &mut AppState, s: &str, p: &str, o: &str) {
    let store = TripleStore::new(state.logic.triples.clone());
    state.logic.results = store.query(s, p, o);
    state.logic.query = Some((s.to_string(), p.to_string(), o.to_string()));
}

pub fn import_csv_from_fd(state: &mut AppState, fd: RawFd) -> Result<(), String> {
    if fd < 0 {
        return Err("logic_import_bad_fd".into());
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|e| format!("logic_import_read_failed:{e}"))?;
    import_csv_content(state, &buf)
}

pub fn import_csv_from_path(state: &mut AppState, path: &str) -> Result<(), String> {
    let mut file = File::open(path).map_err(|e| format!("logic_import_open_failed:{e}"))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|e| format!("logic_import_read_failed:{e}"))?;
    import_csv_content(state, &buf)
}

fn import_csv_content(state: &mut AppState, content: &str) -> Result<(), String> {
    let mut store = TripleStore::new(state.logic.triples.clone());
    store.import_csv(content)?;
    state.logic.triples = store.triples;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triple_store_add_and_query() {
        let mut store = TripleStore::default();
        store.add("a".into(), "b".into(), "c".into());
        store.add("a".into(), "b".into(), "d".into());
        let res = store.query("a", "b", "*");
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn triple_store_import_csv() {
        let mut store = TripleStore::default();
        store.import_csv("s,p,o\nx,y,z").unwrap();
        assert_eq!(store.triples.len(), 2);
        assert_eq!(store.triples[0].subject, "s");
        assert_eq!(store.triples[1].object, "z");
    }

    #[test]
    fn render_logic_screen_renders_results() {
        let mut state = AppState::new();
        state.logic.results.push(LogicTriple {
            subject: "s".into(),
            predicate: "p".into(),
            object: "o".into(),
        });
        let ui = render_logic_screen(&state);
        let ui_str = ui.to_string();
        assert!(ui_str.contains("s  p  o"));
    }
}
