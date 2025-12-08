#[cfg(test)]
mod tests {
    use crate::ui::{TextInput, VirtualList};
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
}
