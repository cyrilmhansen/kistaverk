use crate::state::AppState;
use crate::ui::{Card as UiCard, Column as UiColumn, Section as UiSection, Text as UiText, maybe_push_back, format_bytes};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageInfo {
    pub total_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkInfo {
    pub ssid: Option<String>,
    pub ip: Option<String>,
    pub connection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatteryInfo {
    pub level_pct: Option<u8>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub os_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInfoState {
    pub storage: Option<StorageInfo>,
    pub network: Option<NetworkInfo>,
    pub battery: Option<BatteryInfo>,
    pub device: Option<DeviceInfo>,
    pub last_updated: Option<String>,
    pub error: Option<String>,
}

impl SystemInfoState {
    pub const fn new() -> Self {
        Self {
            storage: None,
            network: None,
            battery: None,
            device: None,
            last_updated: None,
            error: None,
        }
    }
}

pub fn apply_system_info_bindings(
    state: &mut AppState,
    bindings: &HashMap<String, String>,
) -> Result<(), String> {
    let storage = StorageInfo {
        total_bytes: parse_u64(bindings.get("storage_total_bytes")),
        free_bytes: parse_u64(bindings.get("storage_free_bytes")),
    };
    let network = NetworkInfo {
        ssid: bindings.get("network_ssid").cloned(),
        ip: bindings.get("network_ip").cloned(),
        connection: bindings.get("network_connection").cloned(),
    };
    let battery = BatteryInfo {
        level_pct: bindings
            .get("battery_level_pct")
            .and_then(|v| v.parse::<u8>().ok()),
        status: bindings.get("battery_status").cloned(),
    };
    let device = DeviceInfo {
        manufacturer: bindings.get("device_manufacturer").cloned(),
        model: bindings.get("device_model").cloned(),
        os_version: bindings.get("device_os_version").cloned(),
    };

    state.system_info.storage = Some(storage);
    state.system_info.network = Some(network);
    state.system_info.battery = Some(battery);
    state.system_info.device = Some(device);
    state.system_info.last_updated = bindings.get("timestamp").cloned();
    state.system_info.error = None;
    Ok(())
}

fn parse_u64(val: Option<&String>) -> Option<u64> {
    val.and_then(|v| v.parse::<u64>().ok())
}

pub fn render_system_info_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("System Panels").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Device snapshot: storage, network, battery, device info.")
                .size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            crate::ui::Button::new("Refresh", "system_info_update").content_description("system_info_refresh"),
        )
        .unwrap(),
    ];

    if let Some(err) = &state.system_info.error {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Error: {}", err))
                    .size(12.0)
                    .content_description("system_info_error"),
            )
            .unwrap(),
        );
    }

    if let Some(ts) = &state.system_info.last_updated {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Last updated: {}", ts))
                    .size(12.0)
                    .content_description("system_info_timestamp"),
            )
            .unwrap(),
        );
    }

    let mut cards: Vec<Value> = Vec::new();

    if let Some(storage) = &state.system_info.storage {
        let mut items = Vec::new();
        if let Some(total) = storage.total_bytes {
            items.push(json!({"type":"Text","text":format!("Total: {}", format_bytes(total)), "size": 12.0}));
        }
        if let Some(free) = storage.free_bytes {
            items.push(json!({"type":"Text","text":format!("Free: {}", format_bytes(free)), "size": 12.0}));
            if let Some(total) = storage.total_bytes {
                if total > 0 {
                    let used_pct = 100.0 - (free as f64 / total as f64 * 100.0);
                    items.push(json!({"type":"Text","text":format!("Used: {:.1}%", used_pct), "size": 12.0}));
                }
            }
        }
        cards.push(
            serde_json::to_value(
                UiCard::new(vec![serde_json::to_value(UiSection::new(items).title("Storage")).unwrap()])
                    .padding(12),
            )
            .unwrap(),
        );
    }

    if let Some(network) = &state.system_info.network {
        let mut items = Vec::new();
        if let Some(conn) = &network.connection {
            items.push(json!({"type":"Text","text":format!("Connection: {}", conn), "size": 12.0}));
        }
        if let Some(ssid) = &network.ssid {
            items.push(json!({"type":"Text","text":format!("SSID: {}", ssid), "size": 12.0}));
        }
        if let Some(ip) = &network.ip {
            items.push(json!({"type":"Text","text":format!("IP: {}", ip), "size": 12.0}));
        }
        cards.push(
            serde_json::to_value(
                UiCard::new(vec![serde_json::to_value(UiSection::new(items).title("Network")).unwrap()])
                    .padding(12),
            )
            .unwrap(),
        );
    }

    if let Some(battery) = &state.system_info.battery {
        let mut items = Vec::new();
        if let Some(level) = battery.level_pct {
            items.push(json!({"type":"Text","text":format!("Level: {}%", level), "size": 12.0}));
        }
        if let Some(status) = &battery.status {
            items.push(json!({"type":"Text","text":format!("Status: {}", status), "size": 12.0}));
        }
        cards.push(
            serde_json::to_value(
                UiCard::new(vec![serde_json::to_value(UiSection::new(items).title("Battery")).unwrap()])
                    .padding(12),
            )
            .unwrap(),
        );
    }

    if let Some(device) = &state.system_info.device {
        let mut items = Vec::new();
        if let Some(m) = &device.manufacturer {
            items.push(json!({"type":"Text","text":format!("Maker: {}", m), "size": 12.0}));
        }
        if let Some(model) = &device.model {
            items.push(json!({"type":"Text","text":format!("Model: {}", model), "size": 12.0}));
        }
        if let Some(os) = &device.os_version {
            items.push(json!({"type":"Text","text":format!("OS: {}", os), "size": 12.0}));
        }
        cards.push(
            serde_json::to_value(
                UiCard::new(vec![serde_json::to_value(UiSection::new(items).title("Device")).unwrap()])
                    .padding(12),
            )
            .unwrap(),
        );
    }

    if !cards.is_empty() {
        children.push(serde_json::to_value(UiColumn::new(cards).padding(8)).unwrap());
    }

    maybe_push_back(&mut children, state);

    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    #[test]
    fn parses_bindings_into_state() {
        let mut state = AppState::new();
        let mut bindings = HashMap::new();
        bindings.insert("storage_total_bytes".into(), "1024".into());
        bindings.insert("storage_free_bytes".into(), "512".into());
        bindings.insert("network_ssid".into(), "MyWiFi".into());
        bindings.insert("network_ip".into(), "192.168.1.10".into());
        bindings.insert("network_connection".into(), "wifi".into());
        bindings.insert("battery_level_pct".into(), "87".into());
        bindings.insert("battery_status".into(), "charging".into());
        bindings.insert("device_manufacturer".into(), "Acme".into());
        bindings.insert("device_model".into(), "X1".into());
        bindings.insert("device_os_version".into(), "13".into());
        bindings.insert("timestamp".into(), "2024-01-01T00:00:00Z".into());

        apply_system_info_bindings(&mut state, &bindings).unwrap();
        let sys = &state.system_info;
        assert_eq!(sys.storage.as_ref().unwrap().total_bytes, Some(1024));
        assert_eq!(sys.storage.as_ref().unwrap().free_bytes, Some(512));
        assert_eq!(sys.network.as_ref().unwrap().ssid.as_deref(), Some("MyWiFi"));
        assert_eq!(sys.network.as_ref().unwrap().ip.as_deref(), Some("192.168.1.10"));
        assert_eq!(sys.battery.as_ref().unwrap().level_pct, Some(87));
        assert_eq!(sys.device.as_ref().unwrap().model.as_deref(), Some("X1"));
        assert_eq!(sys.last_updated.as_deref(), Some("2024-01-01T00:00:00Z"));
    }

    #[test]
    fn render_includes_sections_when_data_present() {
        let mut state = AppState::new();
        state.system_info.storage = Some(StorageInfo {
            total_bytes: Some(2048),
            free_bytes: Some(1024),
        });
        state.system_info.network = Some(NetworkInfo {
            ssid: Some("Net".into()),
            ip: Some("10.0.0.2".into()),
            connection: Some("wifi".into()),
        });
        let view = render_system_info_screen(&state).to_string();
        assert!(view.contains("Storage"));
        assert!(view.contains("Network"));
        assert!(view.contains("Total"));
        assert!(view.contains("10.0.0.2"));
    }
}
