use crate::state::{AppState, UnitCategory};
use crate::ui::{
    maybe_push_back, Button as UiButton, Column as UiColumn, Text as UiText,
    TextInput as UiTextInput,
};
use serde_json::Value;
use std::collections::HashMap;

pub fn render_unit_converter_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("Unit Converter").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Convert between different units of measurement.").size(14.0),
        )
        .unwrap(),
    ];

    // Category Selector
    let categories = vec!["Length", "Mass", "Temperature", "DigitalStorage"];
    let current_cat = match state.unit_converter.category {
        UnitCategory::Length => "Length",
        UnitCategory::Mass => "Mass",
        UnitCategory::Temperature => "Temperature",
        UnitCategory::DigitalStorage => "DigitalStorage",
    };

    let mut cat_buttons = Vec::new();
    for cat in categories {
        let label = if cat == current_cat {
            format!("• {} •", cat)
        } else {
            cat.to_string()
        };
        cat_buttons.push(
            serde_json::to_value(UiButton::new(&label, "unit_converter_set_category").id(cat))
                .unwrap(),
        );
    }
    children.push(serde_json::to_value(UiColumn::new(cat_buttons).padding(4)).unwrap());

    // Unit Selectors
    let (units, default_from, default_to) = get_units(state.unit_converter.category);
    let current_from = if state.unit_converter.from_unit.is_empty() {
        default_from
    } else {
        &state.unit_converter.from_unit
    };
    let current_to = if state.unit_converter.to_unit.is_empty() {
        default_to
    } else {
        &state.unit_converter.to_unit
    };

    children.push(serde_json::to_value(UiText::new("From:").size(14.0)).unwrap());
    let mut from_buttons = Vec::new();
    for unit in &units {
        let label = if unit == current_from {
            format!("> {} <", unit)
        } else {
            unit.to_string()
        };
        from_buttons.push(
            serde_json::to_value(
                UiButton::new(&label, "unit_converter_set_from")
                    .id(&format!("from_{}", unit)),
            )
            .unwrap(),
        );
    }
    // Simple wrap or grid logic isn't available in base UI, so we stack them or rely on flow if available.
    // For now, simple Column is safe but tall.
    children.push(serde_json::to_value(UiColumn::new(from_buttons).padding(4)).unwrap());

    children.push(serde_json::to_value(UiText::new("To:").size(14.0)).unwrap());
    let mut to_buttons = Vec::new();
    for unit in &units {
        let label = if unit == current_to {
            format!("> {} <", unit)
        } else {
            unit.to_string()
        };
        to_buttons.push(
            serde_json::to_value(
                UiButton::new(&label, "unit_converter_set_to")
                    .id(&format!("to_{}", unit)),
            )
            .unwrap(),
        );
    }
    children.push(serde_json::to_value(UiColumn::new(to_buttons).padding(4)).unwrap());

    // Input and Output
    children.push(
        serde_json::to_value(
            UiTextInput::new("unit_input")
                .hint("Enter value")
                .text(&state.unit_converter.input_value)
                .single_line(true)
                .debounce_ms(200)
                .action_on_submit("unit_converter_calculate"),
        )
        .unwrap(),
    );

    children.push(serde_json::to_value(UiButton::new("Convert", "unit_converter_calculate")).unwrap());

    if !state.unit_converter.output_value.is_empty() {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Result: {}", state.unit_converter.output_value)).size(16.0),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(UiButton::new("Copy Result", "unit_converter_copy")).unwrap(),
        );
    }

    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

fn get_units(category: UnitCategory) -> (Vec<String>, &'static str, &'static str) {
    match category {
        UnitCategory::Length => (
            vec![
                "Meter".into(),
                "Kilometer".into(),
                "Centimeter".into(),
                "Millimeter".into(),
                "Mile".into(),
                "Yard".into(),
                "Foot".into(),
                "Inch".into(),
            ],
            "Meter",
            "Foot",
        ),
        UnitCategory::Mass => (
            vec![
                "Kilogram".into(),
                "Gram".into(),
                "Milligram".into(),
                "Pound".into(),
                "Ounce".into(),
                "Ton".into(),
            ],
            "Kilogram",
            "Pound",
        ),
        UnitCategory::Temperature => (
            vec!["Celsius".into(), "Fahrenheit".into(), "Kelvin".into()],
            "Celsius",
            "Fahrenheit",
        ),
        UnitCategory::DigitalStorage => (
            vec![
                "Byte".into(),
                "Kilobyte".into(),
                "Megabyte".into(),
                "Gigabyte".into(),
                "Terabyte".into(),
            ],
            "Megabyte",
            "Gigabyte",
        ),
    }
}

pub fn handle_unit_converter_action(
    state: &mut AppState,
    action: &str,
    bindings: &HashMap<String, String>,
) {
    // Update input value if present in bindings (auto-sync)
    if let Some(val) = bindings.get("unit_input") {
        state.unit_converter.input_value = val.clone();
    }

    match action {
        "unit_converter_set_category" => {
            if let Some(id) = bindings.get("element_id") {
                let new_cat = match id.as_str() {
                    "Length" => UnitCategory::Length,
                    "Mass" => UnitCategory::Mass,
                    "Temperature" => UnitCategory::Temperature,
                    "DigitalStorage" => UnitCategory::DigitalStorage,
                    _ => state.unit_converter.category,
                };
                if new_cat != state.unit_converter.category {
                    state.unit_converter.category = new_cat;
                    let (_, def_from, def_to) = get_units(new_cat);
                    state.unit_converter.from_unit = def_from.to_string();
                    state.unit_converter.to_unit = def_to.to_string();
                    state.unit_converter.output_value.clear();
                }
            }
        }
        "unit_converter_set_from" => {
            if let Some(id) = bindings.get("element_id") {
                if let Some(unit) = id.strip_prefix("from_") {
                    state.unit_converter.from_unit = unit.to_string();
                    calculate(state);
                }
            }
        }
        "unit_converter_set_to" => {
            if let Some(id) = bindings.get("element_id") {
                if let Some(unit) = id.strip_prefix("to_") {
                    state.unit_converter.to_unit = unit.to_string();
                    calculate(state);
                }
            }
        }
        "unit_converter_calculate" => {
            calculate(state);
        }
        "unit_converter_copy" => {
            // In a real app, this would trigger a platform copy action via effect.
            // For this UI state model, we might set a toast or temporary status.
            state.text_output = Some(state.unit_converter.output_value.clone());
        }
        _ => {}
    }
}

fn calculate(state: &mut AppState) {
    let input_str = state.unit_converter.input_value.trim();
    if input_str.is_empty() {
        state.unit_converter.output_value.clear();
        return;
    }

    let Ok(val) = input_str.parse::<f64>() else {
        state.unit_converter.output_value = "Invalid Number".to_string();
        return;
    };

    let from = if state.unit_converter.from_unit.is_empty() {
        let (_, f, _) = get_units(state.unit_converter.category);
        f
    } else {
        &state.unit_converter.from_unit
    };

    let to = if state.unit_converter.to_unit.is_empty() {
        let (_, _, t) = get_units(state.unit_converter.category);
        t
    } else {
        &state.unit_converter.to_unit
    };

    let result = match state.unit_converter.category {
        UnitCategory::Length => convert_length(val, from, to),
        UnitCategory::Mass => convert_mass(val, from, to),
        UnitCategory::Temperature => convert_temp(val, from, to),
        UnitCategory::DigitalStorage => convert_storage(val, from, to),
    };

    state.unit_converter.output_value = format!("{:.6}", result)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string();
}

fn convert_length(val: f64, from: &str, to: &str) -> f64 {
    // Convert everything to meters first
    let meters = match from {
        "Meter" => val,
        "Kilometer" => val * 1000.0,
        "Centimeter" => val / 100.0,
        "Millimeter" => val / 1000.0,
        "Mile" => val * 1609.344,
        "Yard" => val * 0.9144,
        "Foot" => val * 0.3048,
        "Inch" => val * 0.0254,
        _ => val,
    };

    // Convert meters to target
    match to {
        "Meter" => meters,
        "Kilometer" => meters / 1000.0,
        "Centimeter" => meters * 100.0,
        "Millimeter" => meters * 1000.0,
        "Mile" => meters / 1609.344,
        "Yard" => meters / 0.9144,
        "Foot" => meters / 0.3048,
        "Inch" => meters / 0.0254,
        _ => meters,
    }
}

fn convert_mass(val: f64, from: &str, to: &str) -> f64 {
    // Convert everything to kilograms first
    let kg = match from {
        "Kilogram" => val,
        "Gram" => val / 1000.0,
        "Milligram" => val / 1_000_000.0,
        "Pound" => val * 0.45359237,
        "Ounce" => val * 0.02834952,
        "Ton" => val * 1000.0, // Metric ton
        _ => val,
    };

    match to {
        "Kilogram" => kg,
        "Gram" => kg * 1000.0,
        "Milligram" => kg * 1_000_000.0,
        "Pound" => kg / 0.45359237,
        "Ounce" => kg / 0.02834952,
        "Ton" => kg / 1000.0,
        _ => kg,
    }
}

fn convert_temp(val: f64, from: &str, to: &str) -> f64 {
    // Convert to Celsius first
    let c = match from {
        "Celsius" => val,
        "Fahrenheit" => (val - 32.0) * 5.0 / 9.0,
        "Kelvin" => val - 273.15,
        _ => val,
    };

    match to {
        "Celsius" => c,
        "Fahrenheit" => (c * 9.0 / 5.0) + 32.0,
        "Kelvin" => c + 273.15,
        _ => c,
    }
}

fn convert_storage(val: f64, from: &str, to: &str) -> f64 {
    // Convert to Bytes first (using 1024 base)
    let bytes = match from {
        "Byte" => val,
        "Kilobyte" => val * 1024.0,
        "Megabyte" => val * 1024.0 * 1024.0,
        "Gigabyte" => val * 1024.0 * 1024.0 * 1024.0,
        "Terabyte" => val * 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => val,
    };

    match to {
        "Byte" => bytes,
        "Kilobyte" => bytes / 1024.0,
        "Megabyte" => bytes / (1024.0 * 1024.0),
        "Gigabyte" => bytes / (1024.0 * 1024.0 * 1024.0),
        "Terabyte" => bytes / (1024.0 * 1024.0 * 1024.0 * 1024.0),
        _ => bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_conversion() {
        assert!((convert_length(1000.0, "Meter", "Kilometer") - 1.0).abs() < 1e-6);
        assert!((convert_length(1.0, "Inch", "Centimeter") - 2.54).abs() < 1e-6);
    }

    #[test]
    fn test_mass_conversion() {
        assert!((convert_mass(1.0, "Kilogram", "Gram") - 1000.0).abs() < 1e-6);
        assert!((convert_mass(1.0, "Pound", "Kilogram") - 0.45359237).abs() < 1e-6);
    }

    #[test]
    fn test_temp_conversion() {
        assert!((convert_temp(0.0, "Celsius", "Fahrenheit") - 32.0).abs() < 1e-6);
        assert!((convert_temp(100.0, "Celsius", "Fahrenheit") - 212.0).abs() < 1e-6);
        assert!((convert_temp(0.0, "Kelvin", "Celsius") - -273.15).abs() < 1e-6);
    }

    #[test]
    fn test_storage_conversion() {
        assert!((convert_storage(1.0, "Kilobyte", "Byte") - 1024.0).abs() < 1e-6);
        assert!((convert_storage(1.0, "Gigabyte", "Megabyte") - 1024.0).abs() < 1e-6);
    }
}
