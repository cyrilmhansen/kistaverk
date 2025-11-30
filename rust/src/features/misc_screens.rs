use crate::state::AppState;
use crate::ui::{
    Barometer as UiBarometer,
    Button as UiButton,
    Column as UiColumn,
    Compass as UiCompass,
    DepsList as UiDepsList,
    Magnetometer as UiMagnetometer,
    Progress as UiProgress,
    Text as UiText,
    maybe_push_back,
};
use serde_json::{json, Value};

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
    let mut children = vec![
        serde_json::to_value(UiText::new("About Kistaverk").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(&format!("Version: {}", env!("CARGO_PKG_VERSION"))).size(14.0),
        )
        .unwrap(),
        serde_json::to_value(UiText::new("Copyright © 2025 Kistaverk").size(14.0)).unwrap(),
        serde_json::to_value(UiText::new("License: GPLv3").size(14.0)).unwrap(),
        serde_json::to_value(
            UiText::new("This app is open-source under GPL-3.0; contributions welcome.").size(12.0),
        )
        .unwrap(),
        serde_json::to_value(UiDepsList::new()).unwrap(),
    ];
    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(24)).unwrap()
}
