#![forbid(unsafe_code)]
//! PLATO Runtime Kernel — the spatial spreadsheet engine.
//!
//! Rooms are cells. Cells are tensors. Markdown is the AST.
//! Plain-English bullets are runtime assertions.
//! Delta compression for sync. Three-way merge for conflict.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod delta;
pub mod merge;

// ============================================================
// Room Identity — a cell in the tensor
// ============================================================

/// A room's spatial identity within the tensor grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomIdentity {
    pub room_id: String,
    pub tensor_hash: String,
    pub grid_position: (usize, usize),
    pub depth: RoomDepth,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoomDepth {
    Floor,    // Top level — dancers/agents
    Board,    // DJ board — instruments/tools
    Panel,    // Instrument panel — settings/presets
    Code,     // Code level — functions
    Metal,    // Metal level — transistors/bits
}

// ============================================================
// Room Contract — the ROOM.json schema
// ============================================================

/// The full ROOM.json contract that defines a room's spatial borders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomContract {
    pub room_id: String,
    pub identity: RoomIdentity,
    pub topology: RoomTopology,
    pub runtime_assets: RuntimeAssets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomTopology {
    pub parent_room: Option<String>,
    pub adjacent_rooms: Vec<String>,
    pub traversal_history: Vec<TraversalRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalRecord {
    pub target_room: String,
    pub weight: f64,
    pub last_baton: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAssets {
    pub specification: String,
    pub reflex_bindings: HashMap<String, String>,
}

impl RoomContract {
    /// Parse from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Check if a room is adjacent to this one.
    pub fn is_adjacent(&self, room_id: &str) -> bool {
        self.topology.adjacent_rooms.iter().any(|r| r == room_id)
    }

    /// Record a traversal to another room.
    pub fn record_traversal(&mut self, target: &str, baton_id: &str, tick: u64) {
        if let Some(record) = self.topology.traversal_history.iter_mut()
            .find(|r| r.target_room == target)
        {
            record.weight += 0.1;
            record.last_baton = baton_id.to_string();
            record.timestamp = tick;
        } else {
            self.topology.traversal_history.push(TraversalRecord {
                target_room: target.to_string(),
                weight: 0.1,
                last_baton: baton_id.to_string(),
                timestamp: tick,
            });
        }
    }
}

// ============================================================
// Baton — the state carrier passing through rooms
// ============================================================

/// A baton payload — immutable execution state passing through rooms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baton {
    pub baton_id: String,
    pub current_scope: String,
    pub cognitive_state: String,
    pub operational_data: HashMap<String, String>,
    pub tick: u64,
}

impl Baton {
    pub fn new(id: &str, scope: &str) -> Self {
        Self {
            baton_id: id.to_string(),
            current_scope: scope.to_string(),
            cognitive_state: String::new(),
            operational_data: HashMap::new(),
            tick: 0,
        }
    }

    /// Advance baton to a new room scope.
    pub fn advance_to(&mut self, room_id: &str) {
        self.current_scope = room_id.to_string();
        self.tick += 1;
    }

    /// Set a data field.
    pub fn set_data(&mut self, key: &str, value: &str) {
        self.operational_data.insert(key.to_string(), value.to_string());
    }

    /// Get a data field.
    pub fn get_data(&self, key: &str) -> Option<&str> {
        self.operational_data.get(key).map(|s| s.as_str())
    }

    /// Get the Unix timestamp (seconds since epoch) for cross-layer interop.
    /// Matches the `t` field in PLATO Wire Protocol v0.1 tick responses.
    pub fn timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Convert baton tick to a floating-point Unix timestamp for protocol JSON.
    /// Uses the current system time, matching engine block `t` field format.
    pub fn timestamp_f64(&self) -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    }
}

// ============================================================
// Assertion Trap — plain-English behavioral constraints
// ============================================================

/// Result of validating output against plain-English assertions.
#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub passed: bool,
    pub violations: Vec<String>,
    pub summary: String,
}

/// Extract assertions from a Markdown spec's Behavioral Constraints section.
pub fn extract_assertions(markdown: &str) -> Vec<String> {
    let mut assertions = Vec::new();
    let mut in_constraints = false;

    for line in markdown.lines() {
        if line.contains("Behavioral Constraints") || line.contains("🛑") {
            in_constraints = true;
            continue;
        }
        if in_constraints && line.starts_with('#') {
            break;
        }
        if in_constraints && (line.trim().starts_with('*') || line.trim().starts_with('-')) {
            let clean = line.replace('*', "").replace('-', "").trim().to_string();
            if !clean.is_empty() {
                assertions.push(clean);
            }
        }
    }
    assertions
}

/// Validate an output payload against plain-English assertions.
pub fn validate_payload(payload: &str, assertions: &[String]) -> AssertionResult {
    let mut violations = Vec::new();

    for assertion in assertions {
        let lower = assertion.to_lowercase();

        // "must contain X"
        if lower.contains("must contain") {
            let keyword = assertion.split("must contain")
                .last().unwrap_or("")
                .trim()
                .replace("'", "").replace("`", "").replace("\"", "");
            if !keyword.is_empty() && !payload.contains(&keyword) {
                violations.push(format!("Missing required: '{}'", keyword));
            }
        }

        // "shall not" / "must not" contain X
        if lower.contains("shall not") || lower.contains("must not") || lower.contains("no condition") {
            let forbidden = if lower.contains("contain") {
                assertion.split("contain").last().unwrap_or("")
            } else {
                assertion.split("word").last().unwrap_or("")
            };
            let forbidden = forbidden.trim()
                .replace("'", "").replace("`", "").replace("\"", "");
            if !forbidden.is_empty() && payload.contains(&forbidden) {
                violations.push(format!("Forbidden term found: '{}'", forbidden));
            }
        }

        // "must be" / "should be" X
        if lower.contains("must be ") || lower.contains("should be ") {
            // General pattern — check if the property is present
            let parts: Vec<&str> = assertion.splitn(2, "must be ").collect();
            if parts.len() == 2 {
                let expected = parts[1].trim().replace("'", "").replace("`", "");
                if !expected.is_empty() && !payload.contains(&expected) {
                    violations.push(format!("Expected property: '{}'", expected));
                }
            }
        }
    }

    let passed = violations.is_empty();
    let summary = if passed {
        "✅ All assertions verified.".to_string()
    } else {
        format!("❌ {} violation(s):\n{}", violations.len(), violations.join("\n"))
    };

    AssertionResult { passed, violations, summary }
}

// ============================================================
// Grid Bridge — tensor cell ↔ room mapping
// ============================================================

/// Maps spreadsheet cell coordinates to room paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridBridge {
    cells: HashMap<String, String>,
}

impl GridBridge {
    pub fn new() -> Self { Self { cells: HashMap::new() } }

    /// Map a cell coordinate to a room path.
    pub fn hydrate(&mut self, coord: &str, room_path: &str) {
        self.cells.insert(coord.to_string(), room_path.to_string());
    }

    /// Get room path for a cell.
    pub fn get_room(&self, coord: &str) -> Option<&str> {
        self.cells.get(coord).map(|s| s.as_str())
    }

    /// Serialize the entire grid topology to JSON.
    pub fn serialize(&self) -> String {
        serde_json::to_string_pretty(&self.cells).unwrap_or_default()
    }

    /// Get all cell coordinates.
    pub fn cells(&self) -> Vec<&str> {
        self.cells.keys().map(|s| s.as_str()).collect()
    }

    /// Hydrate from a directory tree representation.
    pub fn hydrate_tree(&mut self, tree: &HashMap<String, HashMap<String, String>>, prefix: &str) -> usize {
        let mut count = 0;
        for (name, children) in tree {
            if children.contains_key("ROOM.json") {
                let row = self.cells.len() + 1;
                self.hydrate(&format!("A{}", row), &format!("{}/{}", prefix, name));
                count += 1;
            }
        }
        count
    }
}

// ============================================================
// Tutor Loop — the compile-test-refine cycle
// ============================================================

/// The tutor loop executor — wraps execution in assertion-checked cycles.
pub struct TutorLoop {
    pub max_iterations: usize,
    pub iteration: usize,
}

impl TutorLoop {
    pub fn new(max_iterations: usize) -> Self { Self { max_iterations, iteration: 0 } }

    /// Execute one cycle: generate output, validate, return result.
    pub fn cycle(&mut self, output: &str, spec: &str) -> TutorCycleResult {
        self.iteration += 1;
        let assertions = extract_assertions(spec);
        let result = validate_payload(output, &assertions);

        TutorCycleResult {
            iteration: self.iteration,
            passed: result.passed,
            violations: result.violations.clone(),
            should_continue: !result.passed && self.iteration < self.max_iterations,
            summary: result.summary,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TutorCycleResult {
    pub iteration: usize,
    pub passed: bool,
    pub violations: Vec<String>,
    pub should_continue: bool,
    pub summary: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // RoomIdentity tests
    #[test] fn test_room_identity() { let ri = RoomIdentity { room_id: "/rooms/test".into(), tensor_hash: "abc".into(), grid_position: (0,0), depth: RoomDepth::Floor }; assert_eq!(ri.room_id, "/rooms/test"); }
    #[test] fn test_room_depth_floor() { let ri = RoomIdentity { room_id: "/".into(), tensor_hash: "".into(), grid_position: (0,0), depth: RoomDepth::Floor }; assert_eq!(ri.depth, RoomDepth::Floor); }

    // RoomContract tests
    #[test] fn test_contract_json_roundtrip() { let c = RoomContract { room_id: "/test".into(), identity: RoomIdentity { room_id: "/test".into(), tensor_hash: "h".into(), grid_position: (1,1), depth: RoomDepth::Code }, topology: RoomTopology { parent_room: None, adjacent_rooms: vec![], traversal_history: vec![] }, runtime_assets: RuntimeAssets { specification: "README.md".into(), reflex_bindings: HashMap::new() } }; let json = c.to_json(); let parsed = RoomContract::from_json(&json).unwrap(); assert_eq!(parsed.room_id, "/test"); }
    #[test] fn test_contract_adjacent() { let c = RoomContract { room_id: "/a".into(), identity: RoomIdentity { room_id: "/a".into(), tensor_hash: "".into(), grid_position: (0,0), depth: RoomDepth::Floor }, topology: RoomTopology { parent_room: None, adjacent_rooms: vec!["/b".into(), "/c".into()], traversal_history: vec![] }, runtime_assets: RuntimeAssets { specification: "README.md".into(), reflex_bindings: HashMap::new() } }; assert!(c.is_adjacent("/b")); assert!(!c.is_adjacent("/d")); }
    #[test] fn test_traversal_record() { let mut c = RoomContract { room_id: "/a".into(), identity: RoomIdentity { room_id: "/a".into(), tensor_hash: "".into(), grid_position: (0,0), depth: RoomDepth::Floor }, topology: RoomTopology { parent_room: None, adjacent_rooms: vec!["/b".into()], traversal_history: vec![] }, runtime_assets: RuntimeAssets { specification: "README.md".into(), reflex_bindings: HashMap::new() } }; c.record_traversal("/b", "baton_1", 1); assert_eq!(c.topology.traversal_history.len(), 1); assert_eq!(c.topology.traversal_history[0].weight, 0.1); }

    // Baton tests
    #[test] fn test_baton_new() { let b = Baton::new("b1", "/rooms/test"); assert_eq!(b.baton_id, "b1"); assert_eq!(b.tick, 0); }
    #[test] fn test_baton_advance() { let mut b = Baton::new("b1", "/a"); b.advance_to("/b"); assert_eq!(b.current_scope, "/b"); assert_eq!(b.tick, 1); }
    #[test] fn test_baton_data() { let mut b = Baton::new("b1", "/a"); b.set_data("key", "value"); assert_eq!(b.get_data("key"), Some("value")); }
    #[test] fn test_baton_timestamp() { let b = Baton::new("b1", "/a"); let ts = b.timestamp(); assert!(ts > 0); }
    #[test] fn test_baton_timestamp_f64() { let b = Baton::new("b1", "/a"); let ts = b.timestamp_f64(); assert!(ts > 0.0); }

    // Assertion extraction tests
    #[test] fn test_extract_assertions() { let md = "# Title\n## Behavioral Constraints\n* Must contain SUCCESS\n* Shall not contain ERROR"; let a = extract_assertions(md); assert_eq!(a.len(), 2); }
    #[test] fn test_extract_assertions_emoji() { let md = "# Title\n## 🛑 Constraints\n* Must contain OK\n* Must not contain FAIL"; let a = extract_assertions(md); assert_eq!(a.len(), 2); }
    #[test] fn test_extract_no_constraints() { let md = "# Title\nJust some text"; let a = extract_assertions(md); assert!(a.is_empty()); }
    #[test] fn test_extract_constraints_boundary() { let md = "## 🛑 Constraints\n* Rule 1\n## Next Section\n* Not a constraint"; let a = extract_assertions(md); assert_eq!(a.len(), 1); }

    // Validation tests
    #[test] fn test_validate_pass() { let assertions = vec!["output must contain OK".into()]; let r = validate_payload("Status: OK", &assertions); assert!(r.passed); }
    #[test] fn test_validate_fail_missing() { let assertions = vec!["output must contain SUCCESS".into()]; let r = validate_payload("Status: FAIL", &assertions); assert!(!r.passed); assert_eq!(r.violations.len(), 1); }
    #[test] fn test_validate_forbidden() { let assertions = vec!["shall not contain ERROR".into()]; let r = validate_payload("System ERROR occurred", &assertions); assert!(!r.passed); }
    #[test] fn test_validate_forbidden_pass() { let assertions = vec!["shall not contain CRITICAL".into()]; let r = validate_payload("All systems nominal", &assertions); assert!(r.passed); }
    #[test] fn test_validate_multiple() { let assertions = vec!["output must contain Resource ID".into(), "shall not contain CRITICAL".into()]; let r = validate_payload("Resource ID: abc-123\nStatus: OK", &assertions); assert!(r.passed); }
    #[test] fn test_validate_empty() { let r = validate_payload("anything", &[]); assert!(r.passed); }

    // GridBridge tests
    #[test] fn test_grid_new() { let g = GridBridge::new(); assert!(g.cells().is_empty()); }
    #[test] fn test_grid_hydrate() { let mut g = GridBridge::new(); g.hydrate("A1", "/rooms/test"); assert_eq!(g.get_room("A1"), Some("/rooms/test")); }
    #[test] fn test_grid_serialize() { let mut g = GridBridge::new(); g.hydrate("A1", "/a"); let json = g.serialize(); assert!(json.contains("A1")); }

    // TutorLoop tests
    #[test] fn test_tutor_pass() { let mut tl = TutorLoop::new(3); let spec = "## 🛑 Constraints\n* output must contain OK"; let r = tl.cycle("Status: OK", spec); assert!(r.passed); assert!(!r.should_continue); }
    #[test] fn test_tutor_fail_then_pass() { let mut tl = TutorLoop::new(3); let spec = "## 🛑 Constraints\n* output must contain OK"; let r1 = tl.cycle("Status: PENDING", spec); assert!(!r1.passed); assert!(r1.should_continue); let r2 = tl.cycle("Status: OK", spec); assert!(r2.passed); }
    #[test] fn test_tutor_max_iterations() { let mut tl = TutorLoop::new(2); let spec = "## 🛑 Constraints\n* Must contain OK"; tl.cycle("nope", spec); tl.cycle("still nope", spec); assert_eq!(tl.iteration, 2); }
}
