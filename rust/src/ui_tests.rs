#[cfg(test)]
mod tests {
    use crate::features::misc_screens::render_about_screen;
    use crate::state::AppState;
    use crate::ui::{DepsList, HtmlView, TextInput, VirtualList};
    use serde_json::json;

    #[test]
    fn virtual_list_serializes_debounce() {
        let list = VirtualList::new(vec![json!({"type": "Text", "text": "row"})])
            .estimated_item_height(64)
            .id("list");
        let val = serde_json::to_value(list).unwrap();
        assert_eq!(val.get("type").and_then(|v| v.as_str()), Some("VirtualList"));
        assert_eq!(val.get("estimated_item_height").and_then(|v| v.as_u64()), Some(64));
        assert_eq!(val.get("id").and_then(|v| v.as_str()), Some("list"));
    }

    #[test]
    fn text_input_serializes_debounce() {
        let input = TextInput::new("search")
            .hint("Find")
            .debounce_ms(150)
            .single_line(true);
        let val = serde_json::to_value(input).unwrap();
        assert_eq!(val.get("bind_key").and_then(|v| v.as_str()), Some("search"));
        assert_eq!(val.get("debounce_ms").and_then(|v| v.as_u64()), Some(150));
        assert_eq!(val.get("single_line").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn text_input_omits_debounce_when_none() {
        let input = TextInput::new("plain").hint("Type");
        let val = serde_json::to_value(input).unwrap();
        assert!(val.get("debounce_ms").is_none());
    }

    #[test]
    fn html_view_serializes_height() {
        let html = HtmlView::new("<p>ok</p>").height_dp(200);
        let val = serde_json::to_value(html).unwrap();
        assert_eq!(val.get("height_dp").and_then(|v| v.as_u64()), Some(200));
        assert_eq!(val.get("html").and_then(|v| v.as_str()), Some("<p>ok</p>"));
    }

    #[test]
    fn deps_list_serializes_query() {
        let deps = DepsList::new().query("serde");
        let val = serde_json::to_value(deps).unwrap();
        assert_eq!(val.get("query").and_then(|v| v.as_str()), Some("serde"));
    }

    #[test]
    fn about_screen_forwards_filter_query() {
        let mut state = AppState::new();
        state.deps_filter_query = Some("openssl".to_string());
        let ui = render_about_screen(&state);
        let children = ui
            .get("children")
            .and_then(|v| v.as_array())
            .expect("column children");
        assert!(
            children
                .iter()
                .any(|c| c.get("type").and_then(|t| t.as_str()) == Some("TextInput")),
            "expected filter input"
        );
        let deps = children
            .iter()
            .find(|c| c.get("type").and_then(|t| t.as_str()) == Some("DepsList"))
            .expect("deps list present");
        assert_eq!(
            deps.get("query").and_then(|v| v.as_str()),
            Some("openssl")
        );
    }
}
