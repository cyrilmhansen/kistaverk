use crate::state::AppState;

use crate::ui::{
    self, maybe_push_back, Button as UiButton, Column as UiColumn, Text as UiText,
    Warning as UiWarning,
};
use rust_i18n::t;
/// Represents which sensors the user wants to capture.
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensorSelection {
    pub accel: bool,
    pub gyro: bool,
    pub mag: bool,
    pub pressure: bool,
    pub gps: bool,
    pub battery: bool,
}

impl SensorSelection {
    pub fn any(self) -> bool {
        self.accel || self.gyro || self.mag || self.pressure || self.gps || self.battery
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensorConfig {
    pub selection: SensorSelection,
    pub interval_ms: u64,
}

/// Parse bindings coming from Kotlin UI to a typed sensor config.
pub fn parse_bindings(
    bindings: &std::collections::HashMap<String, String>,
) -> Result<SensorConfig, String> {
    let sel = SensorSelection {
        accel: bindings
            .get("sensor_accel")
            .map(|v| v == "true")
            .unwrap_or(true),
        gyro: bindings
            .get("sensor_gyro")
            .map(|v| v == "true")
            .unwrap_or(true),
        mag: bindings
            .get("sensor_mag")
            .map(|v| v == "true")
            .unwrap_or(true),
        pressure: bindings
            .get("sensor_pressure")
            .map(|v| v == "true")
            .unwrap_or(false),
        gps: bindings
            .get("sensor_gps")
            .map(|v| v == "true")
            .unwrap_or(false),
        battery: bindings
            .get("sensor_battery")
            .map(|v| v == "true")
            .unwrap_or(true),
    };

    if !sel.any() {
        return Err("no_sensor_selected".into());
    }

    let interval_ms = bindings
        .get("sensor_interval_ms")
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v >= 50 && *v <= 10_000)
        .unwrap_or(200);

    Ok(SensorConfig {
        selection: sel,
        interval_ms,
    })
}

pub fn apply_status_from_bindings(
    state: &mut AppState,
    bindings: &std::collections::HashMap<String, String>,
) {
    if let Some(s) = bindings.get("sensor_status") {
        state.sensor_status = Some(s.clone());
    }
    if let Some(p) = bindings.get("sensor_path") {
        state.last_sensor_log = Some(p.clone());
    }
}

pub fn render_sensor_logger_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new(&t!("sensor_logger_title")).size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(&t!("sensor_logger_description")).size(14.0),
        )
        .unwrap(),
        serde_json::to_value(UiText::new(&t!("sensor_logger_sensors_section")).size(14.0)).unwrap(),
        serde_json::to_value(
            UiColumn::new(vec![
                serde_json::to_value(
                    ui::Checkbox::new(&t!("sensor_accelerometer"), "sensor_accel")
                        .checked(state.sensor_selection.map(|s| s.accel).unwrap_or(true)),
                )
                .unwrap(),
                serde_json::to_value(
                    ui::Checkbox::new(&t!("sensor_gyroscope"), "sensor_gyro")
                        .checked(state.sensor_selection.map(|s| s.gyro).unwrap_or(true)),
                )
                .unwrap(),
                serde_json::to_value(
                    ui::Checkbox::new(&t!("sensor_magnetometer"), "sensor_mag")
                        .checked(state.sensor_selection.map(|s| s.mag).unwrap_or(true)),
                )
                .unwrap(),
                serde_json::to_value(
                    ui::Checkbox::new(&t!("sensor_barometer"), "sensor_pressure")
                        .checked(state.sensor_selection.map(|s| s.pressure).unwrap_or(false)),
                )
                .unwrap(),
                serde_json::to_value(
                    ui::Checkbox::new(&t!("sensor_gps"), "sensor_gps")
                        .checked(state.sensor_selection.map(|s| s.gps).unwrap_or(false)),
                )
                .unwrap(),
                serde_json::to_value(
                    ui::Checkbox::new(&t!("sensor_battery"), "sensor_battery")
                        .checked(state.sensor_selection.map(|s| s.battery).unwrap_or(true)),
                )
                .unwrap(),
            ])
            .padding(8),
        )
        .unwrap(),
        serde_json::to_value(
            ui::TextInput::new("sensor_interval_ms")
                .hint(&t!("sensor_interval_ms_hint"))
                .text(
                    &state
                        .sensor_interval_ms
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "200".into()),
                )
                .content_description(&t!("sensor_interval_ms_content_description")),
        )
        .unwrap(),
        serde_json::to_value(UiButton::new(&t!("sensor_start_logging_button"), "sensor_logger_start")).unwrap(),
        serde_json::to_value(UiButton::new(&t!("sensor_stop_logging_button"), "sensor_logger_stop")).unwrap(),
    ];

    if let Some(status) = &state.sensor_status {
        children.push(
            serde_json::to_value(UiText::new(&format!("{}{}", t!("sensor_status_prefix"), status)).size(12.0)).unwrap(),
        );
    }
    if state.sensor_status.as_deref() == Some("logging") {
        children.push(
            serde_json::to_value(
                UiWarning::new(&t!("sensor_logging_foreground_service"))
                    .content_description("sensor_logger_foreground_status"),
            )
            .unwrap(),
        );
    }
    if let Some(err) = &state.last_error {
        children.push(
            serde_json::to_value(UiText::new(&format!("{}{}", t!("multi_hash_error_prefix"), err)).size(12.0)).unwrap(),
        );
    }
    if let Some(path) = &state.last_sensor_log {
        children.push(
            serde_json::to_value(UiText::new(&format!("{}{}", t!("sensor_last_log_prefix"), path)).size(12.0)).unwrap(),
        );
        children.push(
            serde_json::to_value(UiButton::new(&t!("sensor_share_last_log_button"), "sensor_logger_share")).unwrap(),
        );
    }

    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}
