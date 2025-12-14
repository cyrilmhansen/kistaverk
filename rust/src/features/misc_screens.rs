use crate::features::dependencies::render_dependencies_list;
use crate::state::AppState;
use crate::ui::{
    maybe_push_back, Barometer as UiBarometer, Button as UiButton, Column as UiColumn,
    Compass as UiCompass, Magnetometer as UiMagnetometer, Progress as UiProgress, Text as UiText,
    TextInput as UiTextInput,
};
use serde_json::{json, Value};
use rust_i18n::t;

pub const SAMPLE_SHADER: &str = r#"`
precision mediump float;
uniform float u_time;
uniform vec2 u_resolution;
void main() {
    vec2 uv = gl_FragCoord.xy / u_resolution.xy;
    float t = u_time * 0.2;
    vec3 col = 0.5 + 0.5*cos(t + uv.xyx + vec3(0.0,2.0,4.0));
    gl_FragColor = vec4(col, 1.0);
}
"#;

pub fn render_loading_screen(state: &AppState) -> Value {
    let message = state.loading_message.as_deref().unwrap_or("Working...");
    let mut children = vec![serde_json::to_value(UiText::new(message).size(16.0)).unwrap()];
    if state.loading_with_spinner {
        children.push(
            serde_json::to_value(UiProgress::new().content_description("In progress")).unwrap(),
        );
    }
    serde_json::to_value(UiColumn::new(children).padding(24)).unwrap()
}

pub fn render_shader_screen(state: &AppState) -> Value {
    let fragment = state
        .last_shader
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(SAMPLE_SHADER);

    let mut children = vec![
        serde_json::to_value(UiText::new("Shader toy demo").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Simple fragment shader with time and resolution uniforms."),
        )
        .unwrap(),
        json!({
            "type": "ShaderToy",
            "fragment": fragment
        }),
        serde_json::to_value(
            UiButton::new("Load shader from file", "load_shader_file").requires_file_picker(true),
        )
        .unwrap(),
        serde_json::to_value(
            UiText::new("Sample syntax:\nprecision mediump float;\nuniform float u_time;\nuniform vec2 u_resolution;\nvoid main(){ vec2 uv=gl_FragCoord.xy/u_resolution.xy; vec3 col=0.5+0.5*cos(u_time*0.2+uv.xyx+vec3(0.,2.,4.)); gl_FragColor=vec4(col,1.0); }").size(12.0),
        )
        .unwrap(),
    ];
    maybe_push_back(&mut children, state);

    serde_json::to_value(UiColumn::new(children).padding(16)).unwrap()
}

pub fn render_compass_screen(state: &AppState) -> Value {
    let degrees = state.compass_angle_radians.to_degrees();
    let mut children = vec![
        serde_json::to_value(UiText::new("Compass (AGSL)").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Compass dial driven by device sensors. Heading auto-updates when sensors are available.")
                .size(12.0),
        )
        .unwrap(),
        serde_json::to_value(UiText::new(&format!("Heading: {:.1}°", degrees)).size(14.0)).unwrap(),
        serde_json::to_value(UiCompass::new(state.compass_angle_radians)).unwrap(),
        serde_json::to_value(
            UiText::new(
                state
                    .compass_error
                    .as_deref()
                    .unwrap_or("Sensor updates will appear automatically.")
            )
            .size(12.0),
        )
        .unwrap(),
    ];
    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

pub fn render_barometer_screen(state: &AppState) -> Value {
    let reading = state.barometer_hpa.map(|v| format!("{:.1} hPa", v));
    let mut children = vec![
        serde_json::to_value(UiText::new("Barometer").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(
                state
                    .barometer_error
                    .as_deref()
                    .unwrap_or("Live pressure readout from the device barometer (if present)."),
            )
            .size(12.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiText::new(reading.as_deref().unwrap_or("Waiting for sensor...")).size(14.0),
        )
        .unwrap(),
        serde_json::to_value(UiBarometer::new(state.barometer_hpa.unwrap_or(0.0))).unwrap(),
    ];
    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

pub fn render_magnetometer_screen(state: &AppState) -> Value {
    let reading = state
        .magnetometer_ut
        .map(|v| format!("{:.1} µT", v))
        .unwrap_or_else(|| "Waiting for sensor...".into());
    let mut children = vec![
        serde_json::to_value(UiText::new("Magnetometer").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(
                state
                    .magnetometer_error
                    .as_deref()
                    .unwrap_or("Live magnetic field strength (device sensor)."),
            )
            .size(12.0),
        )
        .unwrap(),
        serde_json::to_value(UiText::new(&reading).size(14.0)).unwrap(),
        serde_json::to_value(UiMagnetometer::new(state.magnetometer_ut.unwrap_or(0.0))).unwrap(),
    ];
    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

pub fn render_progress_demo_screen(state: &AppState) -> Value {
    let mut children = vec![
        json!({
            "type": "Text",
            "text": "Progress demo",
            "size": 20.0
        }),
        json!({
            "type": "Text",
            "text": "Tap start to show a 10 second simulated progress and return here when done.",
            "size": 14.0
        }),
        json!({
            "type": "Button",
            "text": "Start 10s work",
            "action": "progress_demo_start"
        }),
    ];

    if let Some(status) = &state.progress_status {
        children.push(json!({
            "type": "Text",
            "text": format!("Status: {}", status),
            "size": 14.0
        }));
    }

    maybe_push_back(&mut children, state);

    json!({
        "type": "Column",
        "padding": 24,
        "children": children
    })
}

pub fn render_about_screen(state: &AppState) -> Value {
    let filter_value = state.dependencies.query.as_str();
    let mut children = vec![
        serde_json::to_value(UiText::new("About Kistaverk").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(&format!("Version: {}", env!("CARGO_PKG_VERSION"))).size(14.0),
        )
        .unwrap(),
        serde_json::to_value(UiText::new("Copyright © 2025 Kistaverk").size(14.0)).unwrap(),
        serde_json::to_value(UiText::new("License: AGPL-3.0-or-later").size(14.0)).unwrap(),
        serde_json::to_value(
            UiText::new("This app is open-source under AGPL-3.0-or-later; contributions welcome.")
                .size(12.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiTextInput::new("deps_filter")
                .hint("Filter dependencies")
                .text(filter_value)
                .single_line(true)
                .debounce_ms(200)
                .action_on_submit("deps_filter"),
        )
        .unwrap(),
        serde_json::to_value(UiText::new("Open source licenses").size(16.0)).unwrap(),
        render_dependencies_list(&state.dependencies),
    ];
    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(24).scrollable(false)).unwrap()
}

pub fn render_settings_screen(state: &AppState) -> Value {
    use crate::ui::{Button as UiButton, Card as UiCard, Column as UiColumn};
    
    let settings_title = t!("settings_locale");
    let settings_description = t!("settings_locale_description");
    let system_default = t!("settings_system_default");
    let english = t!("locale_english");
    let french = t!("locale_french");
    let german = t!("locale_german");
    let icelandic = t!("locale_icelandic");
    let spanish = t!("locale_spanish");
    let portuguese = t!("locale_portuguese");
    let chinese = t!("locale_chinese");
    let latin = t!("locale_latin");
    
    let current_locale = state.locale.clone();
    let preferred_locale = state.preferred_locale.clone();
    
    // Check if we're using the system locale (no preferred locale set or it's empty)
    let is_using_system = preferred_locale.is_empty() || preferred_locale == "system";
    
    let locale_buttons = vec![
        {
            let mut button = UiButton::new(&system_default, "set_locale")
                .payload(json!({"locale": ""}))  // Empty string means use system
            ;
            if is_using_system {
                button = button.content_description("selected_locale");
            }
            serde_json::to_value(button).unwrap()
        },
        {
            let mut button = UiButton::new(&english, "set_locale")
                .payload(json!({"locale": "en"}))
            ;
            if current_locale == "en" && !is_using_system {
                button = button.content_description("selected_locale");
            }
            serde_json::to_value(button).unwrap()
        },
        {
            let mut button = UiButton::new(&french, "set_locale")
                .payload(json!({"locale": "fr"}))
            ;
            if current_locale == "fr" && !is_using_system {
                button = button.content_description("selected_locale");
            }
            serde_json::to_value(button).unwrap()
        },
        {
            let mut button = UiButton::new(&german, "set_locale")
                .payload(json!({"locale": "de"}))
            ;
            if current_locale == "de" && !is_using_system {
                button = button.content_description("selected_locale");
            }
            serde_json::to_value(button).unwrap()
        },
        {
            let mut button = UiButton::new(&icelandic, "set_locale")
                .payload(json!({"locale": "is"}))
            ;
            if current_locale == "is" && !is_using_system {
                button = button.content_description("selected_locale");
            }
            serde_json::to_value(button).unwrap()
        },
        {
            let mut button = UiButton::new(&spanish, "set_locale")
                .payload(json!({"locale": "es"}))
            ;
            if current_locale == "es" && !is_using_system {
                button = button.content_description("selected_locale");
            }
            serde_json::to_value(button).unwrap()
        },
        {
            let mut button = UiButton::new(&portuguese, "set_locale")
                .payload(json!({"locale": "pt"}))
            ;
            if current_locale == "pt" && !is_using_system {
                button = button.content_description("selected_locale");
            }
            serde_json::to_value(button).unwrap()
        },
        {
            let mut button = UiButton::new(&chinese, "set_locale")
                .payload(json!({"locale": "zn"})) // Using 'zn' as the locale code for Chinese
            ;
            if current_locale == "zn" && !is_using_system {
                button = button.content_description("selected_locale");
            }
            serde_json::to_value(button).unwrap()
        },
        {
            let mut button = UiButton::new(&latin, "set_locale")
                .payload(json!({"locale": "la"}))
            ;
            if current_locale == "la" && !is_using_system {
                button = button.content_description("selected_locale");
            }
            serde_json::to_value(button).unwrap()
        },
    ];
    
    let locale_card = UiCard::new(vec![
        serde_json::to_value(UiColumn::new(locale_buttons).padding(8)).unwrap()
    ])
    .title(&settings_title)
    .subtitle(&settings_description)
    .padding(16);
    
    let mut children = vec![
        serde_json::to_value(locale_card).unwrap(),
    ];
    
    maybe_push_back(&mut children, state);
    
    serde_json::to_value(UiColumn::new(children).padding(20).scrollable(false)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::render_settings_screen;
    use crate::state::AppState;
    
    #[test]
    fn test_settings_screen_renders() {
        let state = AppState::new();
        let result = render_settings_screen(&state);
        
        // Basic check that the function returns a valid JSON value
        assert!(result.is_object());
        
        // Check that it has the expected structure
        if let Some(obj) = result.as_object() {
            assert_eq!(obj.get("type").and_then(|v| v.as_str()), Some("Column"));
        }
    }
    
    #[test]
    fn test_settings_screen_with_different_locales() {
        let mut state = AppState::new();
        
        // Test with English locale
        state.locale = "en".to_string();
        state.preferred_locale = "en".to_string();
        let result_en = render_settings_screen(&state);
        assert!(result_en.is_object());
        
        // Test with French locale
        state.locale = "fr".to_string();
        state.preferred_locale = "fr".to_string();
        let result_fr = render_settings_screen(&state);
        assert!(result_fr.is_object());
        
        // Test with system locale (empty preferred_locale)
        state.locale = "en".to_string();
        state.preferred_locale = String::new();
        let result_system = render_settings_screen(&state);
        assert!(result_system.is_object());
    }
}
