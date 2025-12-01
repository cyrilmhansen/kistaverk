use crate::state::{AppState, RegexMatchResult};
use crate::ui::{Button as UiButton, Column as UiColumn, Text as UiText, TextInput as UiTextInput, maybe_push_back};
use regex::Regex;
use serde_json::Value;

pub fn render_regex_tester_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("Regex Tester").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Enter a pattern and sample text to test matches and capture groups.")
                .size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiTextInput::new("regex_pattern")
                .hint("Pattern (Rust syntax)")
                .text(&state.regex_tester.pattern)
                .single_line(true),
        )
        .unwrap(),
        serde_json::to_value(
            UiTextInput::new("regex_sample")
                .hint("Sample text")
                .text(&state.regex_tester.sample_text),
        )
        .unwrap(),
        serde_json::to_value(UiButton::new("Test", "regex_test")).unwrap(),
    ];

    if let Some(err) = &state.regex_tester.error {
        children.push(
            serde_json::to_value(UiText::new(&format!("Error: {err}")).size(12.0)).unwrap(),
        );
    } else if let Some(result) = &state.regex_tester.match_result {
        let status = if result.matched { "Match" } else { "No match" };
        children.push(
            serde_json::to_value(UiText::new(status).size(14.0).content_description("regex_status"))
                .unwrap(),
        );
        if result.matched {
            for (idx, grp) in result.groups.iter().enumerate() {
                let text = grp
                    .as_deref()
                    .map(|g| format!("Group {idx}: {g}"))
                    .unwrap_or_else(|| format!("Group {idx}: <none>"));
                children.push(serde_json::to_value(UiText::new(&text).size(12.0)).unwrap());
            }
        }
    }

    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

pub fn handle_regex_action(state: &mut AppState, bindings: &std::collections::HashMap<String, String>) {
    state.regex_tester.pattern = bindings
        .get("regex_pattern")
        .cloned()
        .unwrap_or_else(|| state.regex_tester.pattern.clone());
    state.regex_tester.sample_text = bindings
        .get("regex_sample")
        .cloned()
        .unwrap_or_else(|| state.regex_tester.sample_text.clone());

    if let Some(result) = test_regex(&state.regex_tester.pattern, &state.regex_tester.sample_text) {
        match result {
            Ok(res) => {
                state.regex_tester.match_result = Some(res);
                state.regex_tester.error = None;
            }
            Err(e) => {
                state.regex_tester.match_result = None;
                state.regex_tester.error = Some(e);
            }
        }
    }
}

pub fn test_regex(pattern: &str, text: &str) -> Option<Result<RegexMatchResult, String>> {
    if pattern.trim().is_empty() {
        return None;
    }
    match Regex::new(pattern) {
        Ok(re) => {
            let capture = re.captures(text);
            let matched = capture.is_some();
            let groups = capture
                .as_ref()
                .map(|caps| {
                    (0..caps.len())
                        .map(|i| caps.get(i).map(|m| m.as_str().to_string()))
                        .collect()
                })
                .unwrap_or_default();
            Some(Ok(RegexMatchResult { matched, groups }))
        }
        Err(e) => Some(Err(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_matches_and_groups() {
        let res = test_regex("(foo)-(\\d+)", "foo-123").unwrap().unwrap();
        assert!(res.matched);
        assert_eq!(res.groups.len(), 3);
        assert_eq!(res.groups[1].as_deref(), Some("foo"));
        assert_eq!(res.groups[2].as_deref(), Some("123"));
    }

    #[test]
    fn regex_invalid_pattern_returns_error() {
        let res = test_regex("(", "x").unwrap();
        assert!(res.is_err());
    }

    #[test]
    fn regex_empty_pattern_skips() {
        let res = test_regex("", "foo");
        assert!(res.is_none());
    }
}
