# Plug & Play — Plato Runtime Kernel

Copy-paste templates for the three most common patterns.

## Pattern 1: Create Rooms and Navigate Batons

```rust
use plato_runtime_kernel::*;
use std::collections::HashMap;

// Create a room
let mut room = RoomContract {
    room_id: "/engine".into(),
    identity: RoomIdentity {
        room_id: "/engine".into(),
        tensor_hash: "v1".into(),
        grid_position: (0, 0),
        depth: RoomDepth::Floor,
    },
    topology: RoomTopology {
        parent_room: None,
        adjacent_rooms: vec!["/bridge".into()],
        traversal_history: vec![],
    },
    runtime_assets: RuntimeAssets {
        specification: "ROOM.md".into(),
        reflex_bindings: HashMap::new(),
    },
};

// Create and advance a baton
let mut baton = Baton::new("watchdog", "/engine");
baton.set_data("temp", "72.5");
baton.advance_to("/bridge");
room.record_traversal("/bridge", &baton.baton_id, baton.tick);
```

## Pattern 2: Validate Output Against Spec

```rust
use plato_runtime_kernel::*;

let spec = "## 🛑 Constraints\n* output must contain OK\n* shall not contain ERROR";
let assertions = extract_assertions(spec);
let result = validate_payload("Status: OK, temp: 72°C", &assertions);

if result.passed {
    println!("✅ Valid");
} else {
    println!("❌ Violations: {:?}", result.violations);
}

// Tutor loop: iterate until valid
let mut tutor = TutorLoop::new(5);
let result = tutor.cycle("Status: OK", spec);
```

## Pattern 3: Delta Sync + Three-Way Merge

```rust
use plato_runtime_kernel::delta::*;
use plato_runtime_kernel::merge::*;

// Delta: compute and apply
let base = "temp: 70.0\nstatus: OK";
let current = "temp: 75.0\nstatus: OK";
let delta = compute_delta(base, current);
let restored = apply_delta(base, &delta);
assert_eq!(restored, current);

// Merge: resolve concurrent edits
let base = "temp: 70\nrpm: 3000";
let ours = "temp: 75\nrpm: 3000";      // we changed temp
let theirs = "temp: 70\nrpm: 2800";    // they changed rpm
let result = three_way_merge(base, ours, theirs);
if result.has_conflicts {
    println!("Conflicts: {}", result.conflict_count);
} else {
    let merged = render_merge(&result);
    println!("Auto-merged:\n{}", merged);
}
```

## Quick Reference

| What | Code |
|------|------|
| Create room | `RoomContract { room_id, identity, topology, runtime_assets }` |
| Room depth | `RoomDepth::Floor / Board / Panel / Code / Metal` |
| Check adjacent | `room.is_adjacent("/other_room")` |
| Record traversal | `room.record_traversal(target, baton_id, tick)` |
| New baton | `Baton::new("id", "/scope")` |
| Move baton | `baton.advance_to("/new_room")` |
| Carry data | `baton.set_data("key", "val")` / `baton.get_data("key")` |
| Extract assertions | `extract_assertions(markdown_spec)` |
| Validate output | `validate_payload(output, &assertions)` |
| Tutor loop | `TutorLoop::new(max_iter).cycle(output, spec)` |
| Compute delta | `compute_delta(base, current)` → `DeltaPatch` |
| Apply delta | `apply_delta(base, &patch)` → `String` |
| Three-way merge | `three_way_merge(base, ours, theirs)` → `MergeResult` |
| Render merge | `render_merge(&result)` → conflict-marked text |
| Grid map | `GridBridge::new(); grid.hydrate("A1", "/room")` |
