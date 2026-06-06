# Tutorial — Building a Spatial Agent System

This tutorial walks you through creating rooms, connecting them in a spatial grid, passing batons between rooms, and validating behavior with assertion traps.

**Prerequisites:** Rust toolchain, this crate in your project.

## Step 1: Create Your Project

```bash
cargo new spatial_demo && cd spatial_demo
```

Add to `Cargo.toml`:

```toml
[dependencies]
plato-runtime-kernel = { path = "/path/to/plato-runtime-kernel" }
serde_json = "1"
```

## Step 2: Define Room Contracts

Create `src/main.rs`:

```rust
use plato_runtime_kernel::*;
use std::collections::HashMap;

fn main() {
    // Create an engine room contract
    let mut engine_room = RoomContract {
        room_id: "/engine_room".into(),
        identity: RoomIdentity {
            room_id: "/engine_room".into(),
            tensor_hash: "engine_v1".into(),
            grid_position: (0, 0),  // Row 0, Column 0
            depth: RoomDepth::Floor,
        },
        topology: RoomTopology {
            parent_room: Some("/boat".into()),
            adjacent_rooms: vec!["/wheelhouse".into(), "/bilge".into()],
            traversal_history: vec![],
        },
        runtime_assets: RuntimeAssets {
            specification: "specs/engine_room.md".into(),
            reflex_bindings: HashMap::new(),
        },
    };

    // Create a wheelhouse contract
    let wheelhouse = RoomContract {
        room_id: "/wheelhouse".into(),
        identity: RoomIdentity {
            room_id: "/wheelhouse".into(),
            tensor_hash: "wheel_v1".into(),
            grid_position: (0, 1),
            depth: RoomDepth::Floor,
        },
        topology: RoomTopology {
            parent_room: Some("/boat".into()),
            adjacent_rooms: vec!["/engine_room".into()],
            traversal_history: vec![],
        },
        runtime_assets: RuntimeAssets {
            specification: "specs/wheelhouse.md".into(),
            reflex_bindings: HashMap::new(),
        },
    };

    println!("Engine room adjacent to wheelhouse: {}", engine_room.is_adjacent("/wheelhouse"));
    println!("Engine room adjacent to galley: {}", engine_room.is_adjacent("/galley"));
}
```

## Step 3: Create and Pass a Baton

Agents carry state between rooms via batons:

```rust
// Create a watchdog baton starting in the engine room
let mut baton = Baton::new("watchdog", "/engine_room");
baton.set_data("coolant_temp", "96.3");
baton.set_data("threshold", "95.0");
baton.set_data("status", "monitoring");

println!("Baton {} in {}", baton.baton_id, baton.current_scope);
println!("Coolant: {}°C", baton.get_data("coolant_temp").unwrap_or("N/A"));

// Move baton to wheelhouse
baton.advance_to("/wheelhouse");
engine_room.record_traversal("/wheelhouse", &baton.baton_id, baton.tick);

println!("Baton now in {} (tick {})", baton.current_scope, baton.tick);

// Check traversal history
for record in &engine_room.topology.traversal_history {
    println!("Traversal: → {} (weight: {:.1}, baton: {})",
             record.target_room, record.weight, record.last_baton);
}
```

## Step 4: Validate Behavior with Assertions

Define behavioral constraints in Markdown and validate output:

```rust
let spec = r#"
# Engine Room Specification

## 🛑 Behavioral Constraints
* Output must contain OK
* Output must contain Engine ID
* Shall not contain CRITICAL
* Shall not contain ERROR
"#;

// Extract assertions from the spec
let assertions = extract_assertions(spec);
println!("Extracted {} assertions:", assertions.len());
for a in &assertions {
    println!("  • {}", a);
}

// Test valid output
let good_output = "Engine ID: ENG-001 | Status: OK | Temp: 72.3°C";
let result = validate_payload(good_output, &assertions);
println!("\nValid output: {}", result.summary);

// Test invalid output
let bad_output = "CRITICAL: Engine failure detected";
let result = validate_payload(bad_output, &assertions);
println!("Invalid output: {}", result.summary);
for v in &result.violations {
    println!("  • {}", v);
}
```

## Step 5: Use the Tutor Loop

The TutorLoop iterates until all assertions pass:

```rust
let spec = "## 🛑 Constraints\n* output must contain OK\n* shall not contain ERROR";

let mut tutor = TutorLoop::new(5);  // Max 5 attempts
let mut outputs = vec![
    "Status: PENDING",
    "Status: PROCESSING",
    "Status: OK",
];

for output in &outputs {
    let result = tutor.cycle(output, spec);
    println!("Attempt {}: {} — {}", result.iteration,
             if result.passed { "PASS" } else { "FAIL" },
             result.summary);
    if result.passed {
        println!("✅ Output validated!");
        break;
    }
    if !result.should_continue {
        println!("❌ Max iterations reached");
        break;
    }
}
```

## Step 6: Build a Grid Map

Map rooms to spreadsheet coordinates:

```rust
let mut grid = GridBridge::new();

grid.hydrate("A1", "/engine_room");
grid.hydrate("A2", "/wheelhouse");
grid.hydrate("B1", "/bilge");
grid.hydrate("B2", "/galley");

println!("Grid map:");
for coord in grid.cells() {
    println!("  {} → {}", coord, grid.get_room(coord).unwrap());
}

// Serialize the grid topology
let json = grid.serialize();
println!("\nGrid JSON: {}", json);
```

## Step 7: Delta Sync Between Rooms

When rooms need to sync state efficiently:

```rust
use plato_runtime_kernel::delta::*;

let base_state = "engine_room: OK\ntemp: 72.3\npressure: 1013.2";
let current_state = "engine_room: OK\ntemp: 75.1\npressure: 1013.2";

let delta = compute_delta(base_state, current_state);
println!("Delta: {} ops", delta.ops.len());
println!("Base checksum: {}", delta.base_checksum);
println!("Current checksum: {}", delta.current_checksum);

for op in &delta.ops {
    println!("  [line {}] {}: {}", op.line, op.op, op.text);
}

// Apply the delta to reconstruct current state
let reconstructed = apply_delta(base_state, &delta);
assert_eq!(reconstructed, current_state);
println!("✅ Delta applied successfully");
```

## Step 8: Three-Way Merge

When two agents edit the same room concurrently:

```rust
use plato_runtime_kernel::merge::*;

let base = "temp: 70.0\npressure: 1013.2\nstatus: OK";

// Agent A changed temp
let ours = "temp: 72.5\npressure: 1013.2\nstatus: OK";

// Agent B changed status
let theirs = "temp: 70.0\npressure: 1013.2\nstatus: WARNING";

let result = three_way_merge(base, ours, theirs);
println!("Merge: {} clean, {} conflicts", result.clean_count, result.conflict_count);

let merged = render_merge(&result);
println!("{}", merged);
// Output: temp: 72.5\npressure: 1013.2\nstatus: WARNING
```

## What You Built

- ✅ Spatial room contracts with topology and adjacency
- ✅ Baton-based agent navigation with state carrying
- ✅ Markdown behavioral constraints with assertion validation
- ✅ Tutor loop for iterative self-correction
- ✅ Grid bridge for coordinate-based room mapping
- ✅ Delta compression for efficient state sync
- ✅ Three-way merge for concurrent edit reconciliation

## Next Steps

- Build a multi-room fleet with `GridBridge` and traversal tracking
- Implement custom assertion patterns for domain-specific validation
- Use delta compression over a network transport
- Read the `plato-engine-block` docs to see how rooms connect to real sensors
