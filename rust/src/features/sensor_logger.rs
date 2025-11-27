use crate::state::AppState;

/// Represents which sensors the user wants to capture.
use serde::{Deserialize, Serialize};

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
