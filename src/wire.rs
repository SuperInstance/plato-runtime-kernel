//! Wire Protocol Bridge — connect the spatial kernel to engine blocks.
//!
//! Provides conversions between Baton/RoomContract and the PLATO Wire Protocol v0.1
//! JSON message format. This lets a baton carry tick data from engine blocks
//! through the spatial topology, and lets the kernel parse welcome messages
//! from engine blocks it connects to.
//!
//! See: PLATO_WIRE_PROTOCOL.md §Bridge Pattern

use crate::{Baton, RoomContract, RoomDepth, RoomIdentity, RoomTopology, RuntimeAssets};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// Tick JSON — Baton ↔ Wire Protocol
// ============================================================

/// PLATO Wire Protocol v0.1 tick response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireTick {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub t: f64,
    pub seq: u64,
    pub data: HashMap<String, f64>,
}

impl WireTick {
    /// Convert a Baton's state into a wire protocol tick JSON string.
    ///
    /// The baton's `tick` field maps to `seq`, `timestamp_f64()` to `t`,
    /// and any numeric `operational_data` values are extracted into `data`.
    pub fn from_baton(baton: &Baton) -> Self {
        let mut data = HashMap::new();
        for (key, val) in &baton.operational_data {
            if let Ok(f) = val.parse::<f64>() {
                data.insert(key.clone(), f);
            }
        }
        Self {
            msg_type: "tick".to_string(),
            t: baton.timestamp_f64(),
            seq: baton.tick,
            data,
        }
    }

    /// Serialize to a JSON string (single-line, wire protocol format).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"type":"error","message":"tick serialization failed"}"#.to_string()
        })
    }
}

// ============================================================
// Welcome JSON — Engine Block → RoomContract
// ============================================================

/// PLATO Wire Protocol v0.1 welcome message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireWelcome {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub room_id: String,
    pub tick_hz: f64,
    pub sensors: Vec<String>,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_protocol_version")]
    pub protocol_version: String,
}

fn default_format() -> String { "json".to_string() }
fn default_protocol_version() -> String { "0.1".to_string() }

impl WireWelcome {
    /// Parse a welcome JSON line from an engine block.
    pub fn from_json(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }

    /// Convert a welcome message into a RoomContract for spatial tracking.
    ///
    /// The engine block's room becomes a Floor-depth cell in the tensor grid.
    /// Sensors are registered as reflex bindings so the kernel knows what
    /// data streams the engine block provides.
    pub fn to_room_contract(&self, grid_pos: (usize, usize)) -> RoomContract {
        let mut reflex_bindings = HashMap::new();
        for sensor in &self.sensors {
            reflex_bindings.insert(sensor.clone(), "stream".to_string());
        }
        reflex_bindings.insert("tick_hz".to_string(), self.tick_hz.to_string());
        reflex_bindings.insert("protocol_version".to_string(), self.protocol_version.clone());
        reflex_bindings.insert("format".to_string(), self.format.clone());

        RoomContract {
            room_id: format!("/rooms/{}", self.room_id),
            identity: RoomIdentity {
                room_id: format!("/rooms/{}", self.room_id),
                tensor_hash: format!("{:x}", (self.tick_hz * 1000.0) as u64),
                grid_position: grid_pos,
                depth: RoomDepth::Floor,
            },
            topology: RoomTopology {
                parent_room: None,
                adjacent_rooms: vec![],
                traversal_history: vec![],
            },
            runtime_assets: RuntimeAssets {
                specification: "ROOM.json".to_string(),
                reflex_bindings,
            },
        }
    }
}

// ============================================================
// Wire command helpers — format commands from kernel context
// ============================================================

/// Build a `tick` command string.
pub fn cmd_tick() -> &'static str { "tick" }

/// Build a `history N` command string.
pub fn cmd_history(n: usize) -> String { format!("history {}", n) }

/// Build an `actuator` command string.
pub fn cmd_actuator(name: &str, value: f64) -> String { format!("actuator {} {}", name, value) }

/// Build a `subscribe` command string.
pub fn cmd_subscribe() -> &'static str { "subscribe" }

/// Build an `unsubscribe` command string.
pub fn cmd_unsubscribe() -> &'static str { "unsubscribe" }

/// Build a `quit` command string.
pub fn cmd_quit() -> &'static str { "quit" }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Baton;

    #[test]
    fn test_baton_to_tick_json() {
        let mut baton = Baton::new("watchdog", "/engine_room");
        baton.set_data("coolant_temp_c", "96.3");
        baton.set_data("rpm", "1790");
        baton.advance_to("/wheelhouse"); // tick = 1

        let tick = WireTick::from_baton(&baton);
        let json = tick.to_json();

        assert!(json.contains(r#""type":"tick""#));
        assert!(json.contains(r#""seq":1"#));
        assert!(json.contains("coolant_temp_c"));
        assert!(json.contains("96.3"));
        assert!(json.contains("1790"));
    }

    #[test]
    fn test_baton_to_tick_ignores_non_numeric() {
        let mut baton = Baton::new("agent", "/room");
        baton.set_data("temp", "50.5");
        baton.set_data("label", "engine"); // non-numeric, should be skipped
        let tick = WireTick::from_baton(&baton);
        assert!(tick.data.contains_key("temp"));
        assert!(!tick.data.contains_key("label"));
    }

    #[test]
    fn test_welcome_parse() {
        let line = r#"{"type":"welcome","room_id":"engine_room","tick_hz":0.2,"sensors":["coolant_temp_c","bilge_cm","rpm"]}"#;
        let welcome = WireWelcome::from_json(line).unwrap();
        assert_eq!(welcome.room_id, "engine_room");
        assert_eq!(welcome.tick_hz, 0.2);
        assert_eq!(welcome.sensors.len(), 3);
        assert_eq!(welcome.format, "json");
        assert_eq!(welcome.protocol_version, "0.1");
    }

    #[test]
    fn test_welcome_to_contract() {
        let line = r#"{"type":"welcome","room_id":"engine_room","tick_hz":0.2,"sensors":["coolant_temp_c","rpm"]}"#;
        let welcome = WireWelcome::from_json(line).unwrap();
        let contract = welcome.to_room_contract((0, 0));

        assert_eq!(contract.room_id, "/rooms/engine_room");
        assert_eq!(contract.identity.depth, RoomDepth::Floor);
        assert_eq!(contract.identity.grid_position, (0, 0));
        assert!(contract.runtime_assets.reflex_bindings.contains_key("coolant_temp_c"));
        assert!(contract.runtime_assets.reflex_bindings.contains_key("rpm"));
        assert!(contract.runtime_assets.reflex_bindings.contains_key("tick_hz"));
        assert!(contract.runtime_assets.reflex_bindings.contains_key("protocol_version"));
    }

    #[test]
    fn test_welcome_with_protocol_version() {
        let line = r#"{"type":"welcome","room_id":"test","tick_hz":1.0,"sensors":[],"protocol_version":"0.2"}"#;
        let welcome = WireWelcome::from_json(line).unwrap();
        assert_eq!(welcome.protocol_version, "0.2");
    }

    #[test]
    fn test_cmd_builders() {
        assert_eq!(cmd_tick(), "tick");
        assert_eq!(cmd_history(20), "history 20");
        assert_eq!(cmd_actuator("pump", 1.0), "actuator pump 1");
        assert_eq!(cmd_subscribe(), "subscribe");
        assert_eq!(cmd_unsubscribe(), "unsubscribe");
        assert_eq!(cmd_quit(), "quit");
    }
}
