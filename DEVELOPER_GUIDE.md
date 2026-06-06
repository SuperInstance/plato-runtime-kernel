# Developer Guide — Plato Runtime Kernel

## Architecture Overview

The runtime kernel is the spatial layer of the Plato Matrix. Where the Engine Block handles individual room physics (sensors, actuators, ticks), the runtime kernel handles the **spatial model**: rooms as cells in a tensor grid, agents as batons passing between rooms, Markdown specifications as behavioral contracts, delta compression for sync, and three-way merge for conflict resolution.

**Core principle:** Space is a first-class data structure. Rooms aren't just endpoints — they're cells in a tensor with position, depth, and topology.

### Crate Structure

```
src/
├── lib.rs     — Core types: RoomIdentity, RoomContract, RoomTopology, Baton, GridBridge, TutorLoop, assertions
├── delta.rs   — Delta compression: line-based diff with checksums
├── merge.rs   — Three-way merge: reconcile concurrent edits against a common base
```

### Key Concepts

#### Five Depth Levels

Every room exists at one of five depths, from macro to micro:

| Depth | Name | What lives here | Use case |
|-------|------|-----------------|----------|
| 0 | Floor | Agents, humans, autonomous behavior | Live dashboard |
| 1 | Board | Instruments, tools, control surfaces | Operator panel |
| 2 | Panel | Settings, presets, configurations | Tuning interface |
| 3 | Code | Functions, algorithms, logic | Debug view |
| 4 | Metal | Raw bits, hardware registers, firmware | Hardware debug |

A room at Floor depth shows live gauges and agent activity. At Metal depth, it shows register values. The depth is part of `RoomIdentity` and can change — rooms "zoom" between levels.

#### The Baton Pattern

Agents don't live in rooms — they pass through them. A `Baton` carries execution state:

```rust
Baton {
    baton_id: "watchdog",
    current_scope: "/engine_room",
    cognitive_state: "monitoring",
    operational_data: {"coolant_temp": "96.3", "threshold": "95"},
    tick: 42,
}
```

- `advance_to(room_id)` moves the baton to a new room, increments tick.
- `set_data(k, v)` / `get_data(k)` carry arbitrary key-value state.
- Traversals are recorded in the room's `RoomTopology.traversal_history`.

Over time, traversal weights reveal which rooms are most connected — the spatial equivalent of PageRank. Weight increases by 0.1 per traversal.

#### Assertion Traps

Rooms can have Markdown specifications with `## Behavioral Constraints` sections. The runtime extracts plain-English assertions and validates output against them:

```markdown
## 🛑 Behavioral Constraints
* Output must contain Resource ID
* Shall not contain CRITICAL
```

Three assertion patterns:
- **"must contain X"** — Output must include the string X.
- **"shall not contain X"** / **"must not contain X"** — Output must not include X.
- **"must be X"** — Output must include property X.

The `TutorLoop` cycles: generate output → validate → iterate until all assertions pass or max iterations reached.

### Module Walkthrough

#### `lib.rs` — Core Types

**`RoomIdentity`** — A room's spatial coordinates: room_id, tensor_hash, grid_position (row, col), depth level.

**`RoomContract`** — The full ROOM.json schema. Contains identity, topology, and runtime assets (specification path, reflex bindings). Key methods:
- `from_json()` / `to_json()` — Serialization.
- `is_adjacent(room_id)` — Check topology adjacency.
- `record_traversal(target, baton_id, tick)` — Record agent movement.

**`RoomTopology`** — Spatial relationships: parent room, adjacent rooms, traversal history with weights.

**`Baton`** — Immutable-ish execution state carrier. Methods: `new()`, `advance_to()`, `set_data()`, `get_data()`.

**`GridBridge`** — Maps spreadsheet cell coordinates (A1, B3) to room paths. Methods: `hydrate(coord, path)`, `get_room(coord)`, `hydrate_tree()`, `serialize()`.

**`extract_assertions(markdown)`** — Parses a Markdown spec and returns a list of assertion strings from the Behavioral Constraints section.

**`validate_payload(payload, assertions)`** — Checks output against assertions, returns `AssertionResult` with pass/fail and violation details.

**`TutorLoop`** — Iterative validation wrapper. `cycle(output, spec)` runs one validation pass.

#### `delta.rs` — Delta Compression

Line-based diff engine:

- **`text_hash(text)`** — DJB2 hash for content fingerprinting.
- **`compute_delta(base, current)`** — Produces `DeltaPatch` with base/current checksums and `DiffOp` list (ADD/REM per line).
- **`apply_delta(base, patch)`** — Applies a patch to reconstruct current from base.

Use case: when rooms sync state, send deltas instead of full content. The checksums let the receiver verify the base matches.

#### `merge.rs` — Three-Way Merge

Reconciles concurrent edits against a common base:

- **`three_way_merge(base, ours, theirs)`** — Produces `MergeResult` with clean lines and conflicts.
- **`render_merge(result)`** — Outputs text with `<<<<<<< OURS` / `=======` / `>>>>>>> THEIRS` conflict markers.

Merge logic per line:
- Both same → take either (clean).
- Only ours changed → take ours (clean).
- Only theirs changed → take theirs (clean).
- Both changed differently → conflict.

Use case: multiple agents editing the same room's state concurrently. The merge resolves non-conflicting changes automatically and flags conflicts for resolution.

### Extension Points

#### Custom Assertion Patterns

Extend `validate_payload()` with new patterns:

```rust
// Add to the match arms in validate_payload
if lower.contains("must exceed") {
    // Parse threshold, compare against payload numeric value
}
```

#### Custom Grid Topology

The `GridBridge` supports arbitrary coordinate schemes:

```rust
let mut grid = GridBridge::new();
grid.hydrate("A1", "/rooms/engine");
grid.hydrate("B2", "/rooms/bridge");
// Hydrate from a directory tree
grid.hydrate_tree(&dir_tree, "/rooms");
```

#### Delta Over Network

```rust
let base = room_contract.to_json();
// ... room state changes ...
let current = room_contract.to_json();
let delta = compute_delta(&base, &current);
// Send delta (much smaller than full state)
```

### Testing Strategy

```bash
cargo test  # 30+ tests covering all modules
```

Tests include:
- RoomIdentity and RoomContract serialization roundtrips
- Topology adjacency and traversal recording
- Baton state management
- Assertion extraction from Markdown (various formats)
- Payload validation (pass, fail, multiple assertions)
- GridBridge hydration and lookup
- TutorLoop pass/fail iterations
- Delta: no-change, line-change, add, delete, checksums, apply
- Merge: no-change, ours-only, theirs-only, same-change, conflict, mixed, rendering

### Contributing

1. `#![forbid(unsafe_code)]` — No unsafe, ever.
2. All types derive `Serialize`/`Deserialize` — the kernel is network-transparent.
3. Keep assertions simple — they're parsed from plain English, not regex.
4. Add tests for any new assertion patterns or merge logic.
5. Run `cargo test` before submitting.
