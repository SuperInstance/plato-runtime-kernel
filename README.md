# plato-runtime-kernel

**Plato Engine Block — a sub-400-line room runtime for agent-space interaction.**  

*The spatial spreadsheet engine. Rooms are cells. Cells are tensors. Markdown is the AST. Plain-English bullets are runtime assertions. Delta compression for sync. Three-way merge for conflict. This is the Plato Matrix's brain.*

## Why This Exists

The Plato Engine Block (the C/Rust room runtime) handles sensors, actuators, ticks, and text protocols. But a real system has hundreds of rooms that need to be organized, connected, and kept in sync. The plato-runtime-kernel provides the spatial model: rooms as cells in a tensor grid, agents as batons passing between rooms, and Markdown specifications as behavioral contracts.

This crate is where the Plato Matrix stops being "a bunch of sensor nodes" and becomes "a coherent space."

## Architecture

```
                    ┌─────────────────────────────┐
                    │      Tensor Grid             │
                    │  ┌──────┬──────┬──────┐      │
                    │  │ A1   │ A2   │ A3   │      │
                    │  │Engine│Wheel │Back- │      │
                    │  │Room  │house │deck  │      │
                    │  ├──────┼──────┼──────┤      │
                    │  │ B1   │ B2   │ B3   │      │
                    │  │Galley│Bilge │Crow's│      │
                    │  │      │      │Nest  │      │
                    │  └──────┴──────┴──────┘      │
                    │         ↑                    │
                    │    Baton (agent state)        │
                    │    passing between cells      │
                    └─────────────────────────────┘
```

### Five Depth Levels

Every room exists at one of five depths, from macro to micro:

| Depth | Name | Analogy | What lives here |
|-------|------|---------|----------------|
| 0 | Floor | Dance floor | Agents, humans, autonomous behavior |
| 1 | Board | DJ board | Instruments, tools, control surfaces |
| 2 | Panel | Instrument panel | Settings, presets, configurations |
| 3 | Code | Code editor | Functions, algorithms, logic |
| 4 | Metal | Transistors | Raw bits, hardware registers, firmware |

A room can zoom between depths. The engine room at Floor depth shows live gauges. At Metal depth, it shows register values.

### Key Types

- **`RoomIdentity`** — A room's spatial identity: room_id, tensor hash, grid position, depth level
- **`RoomContract`** — The ROOM.json schema defining a room's borders, topology, and runtime assets
- **`RoomTopology`** — Parent room, adjacent rooms, traversal history with weights
- **`Baton`** — Immutable execution state passing through rooms. An agent's "carry-on luggage."
- **`AssertionResult`** — Validation of output against plain-English behavioral constraints
- **`GridBridge`** — Maps spreadsheet cell coordinates to room paths
- **`TutorLoop`** — The compile-test-refine cycle: generate output, validate against spec, iterate

### The Baton Pattern

Agents don't live in rooms — they pass through them. A Baton carries the agent's state from room to room:

```rust
let mut baton = Baton::new("watchdog", "/rooms/engine_room");
baton.set_data("coolant_threshold", "95");
baton.advance_to("/rooms/wheelhouse");
// Baton now in wheelhouse, carrying engine room data
```

Traversals are recorded in the room's topology. Over time, the traversal weights reveal which rooms are most connected — the spatial equivalent of PageRank.

### Assertion Traps

Every room can have a Markdown specification with a `## Behavioral Constraints` section. The runtime extracts these as plain-English assertions:

```markdown
## 🛑 Behavioral Constraints
* Output must contain Resource ID
* Shall not contain CRITICAL
```

The `validate_payload` function checks output against these. The `TutorLoop` cycles until all assertions pass. This is the self-correcting mechanism: rooms don't just run, they verify they're running correctly.

## Usage

```rust
use plato_runtime_kernel::*;

// Create a room contract
let mut contract = RoomContract {
    room_id: "/engine_room".into(),
    identity: RoomIdentity {
        room_id: "/engine_room".into(),
        tensor_hash: "abc123".into(),
        grid_position: (0, 0),
        depth: RoomDepth::Floor,
    },
    topology: RoomTopology {
        parent_room: Some("/boat".into()),
        adjacent_rooms: vec!["/wheelhouse".into(), "/bilge".into()],
        traversal_history: vec![],
    },
    runtime_assets: RuntimeAssets {
        specification: "ROOM.md".into(),
        reflex_bindings: HashMap::new(),
    },
};

// Record traversal (agent moved to wheelhouse)
contract.record_traversal("/wheelhouse", "watchdog_baton", 42);

// Create a baton and pass it through rooms
let mut baton = Baton::new("watchdog", "/engine_room");
baton.set_data("coolant_temp", "96.3");
baton.advance_to("/wheelhouse");

// Validate against behavioral constraints
let spec = "## 🛑 Constraints\n* output must contain OK\n* shall not contain ERROR";
let result = validate_payload("Status: OK, temp: 96.3", &extract_assertions(spec));
assert!(result.passed);

// Tutor loop: iterate until assertions pass
let mut tutor = TutorLoop::new(5);
let output = generate_output();
let result = tutor.cycle(&output, spec);
if result.passed { /* good to go */ }
```

## The Deeper Idea

The plato-runtime-kernel treats space as a first-class data structure. Rooms aren't just endpoints — they're cells in a tensor. The grid position, depth level, and traversal weights form a rich spatial graph that agents can navigate and reason about.

The connection to the Plato Engine Block is direct: each engine block is one cell in the tensor grid. The runtime kernel provides the spatial model that connects them. The engine block handles the physical layer (sensors, actuators, ticks); the runtime kernel handles the spatial layer (topology, traversals, contracts).

The assertion traps connect to the SuperInstance verification philosophy: every component should verify its own correctness. The TutorLoop is the runtime expression of the ternary-compiler-optimizer's compile-test cycle, but applied to spatial behavior.

## PLATO Engine Block Family

The Runtime Kernel provides the spatial layer that connects engine blocks:

| Component | Language | Repo | Focus |
|---|---|---|---|
| **Runtime Kernel** ← you are here | Rust | [plato-runtime-kernel](https://github.com/SuperInstance/plato-runtime-kernel) | Spatial model: tensor grid, batons, assertion traps |
| **C Reference** | C99 | [plato-engine-block-c](https://github.com/SuperInstance/plato-engine-block-c) | Embedded, bare-metal, zero heap alloc |
| **Rust (Original)** | Rust | [plato-engine-block](https://github.com/SuperInstance/plato-engine-block) | `no_std` + alloc, builder pattern, tokio server |
| **Elixir/OTP** | Elixir | [plato-engine-block-elixir](https://github.com/SuperInstance/plato-engine-block-elixir) | BEAM supervision trees, fault tolerance |
| **Zig** | Zig | [plato-engine-block-zig](https://github.com/SuperInstance/plato-engine-block-zig) | Comptime ternary packing, cross-compile |
| **Python Core** | Python | [plato-core](https://github.com/SuperInstance/plato-core) | Foundation types, mesh registry, training tiles |
| **Server** | Python | [plato-server](https://github.com/SuperInstance/plato-server) | Knowledge tiles, fleet sync via Matrix, HTTP API |

**Specs & Guides:**
- 📜 [PLATO Wire Protocol](https://github.com/SuperInstance/AI-Writings/blob/main/PLATO_WIRE_PROTOCOL.md)
- 📖 [PLATO Master Guide](https://github.com/SuperInstance/AI-Writings/blob/main/PLATO_MASTER_GUIDE.md)
- 🗺️ [PLATO Ecosystem Map](https://github.com/SuperInstance/AI-Writings/blob/main/PLATO_ECOSYSTEM_MAP.md)


## Related Crates

- `plato-engine-block` — The atomic room runtime (sensors, actuators, ticks)
- `plato-room-configs` — Production room configurations (ROOM.json files)
- `room-cell` — Cellular computation adapting to available resources
- `pincher` — Agent reflexes that navigate rooms via batons
- `flux-core` — FLUX bytecode for room logic compilation
- `oxide-fleet` — Fleet management for room server clusters

## Ecosystem

This repo is part of the **SuperInstance** flagship ecosystem — agent-first computation, constraint theory, and self-improving runtimes.

### FLUX Runtime Family

| Repo | Language | Description |
|------|----------|-------------|
| [flux-runtime](https://github.com/SuperInstance/flux-runtime) | Python | Full FLUX runtime: markdown→bytecode, 2037 tests, zero deps |
| [flux-core](https://github.com/SuperInstance/flux-core) | Rust | Register-based bytecode VM, deterministic agent computation |
| [flux-js](https://github.com/SuperInstance/flux-js) | JavaScript | FLUX VM for Node.js and browsers, ~400ns/iter |
| [flux-compiler](https://github.com/SuperInstance/flux-compiler) | Rust/Python | Formal-methods compiler for safety-critical codegen |
| [flux-vm](https://github.com/SuperInstance/flux-vm) | Rust | Stack-based constraint-checking VM, 50 opcodes, Turing-incomplete |

### PLATO Engine Family

| Repo | Language | Description |
|------|----------|-------------|
| [plato-server](https://github.com/SuperInstance/plato-server) | Python | Knowledge tiles, fleet sync via Matrix, HTTP API |
| [plato-engine-block](https://github.com/SuperInstance/plato-engine-block) | Rust | Original room runtime: no_std + alloc, builder pattern |
| [plato-engine-block-c](https://github.com/SuperInstance/plato-engine-block-c) | C99 | Embedded reference: zero heap alloc, bare-metal portable |
| [plato-engine-block-elixir](https://github.com/SuperInstance/plato-engine-block-elixir) | Elixir | BEAM supervision trees, fault tolerance, hot reload |
| [plato-runtime-kernel](https://github.com/SuperInstance/plato-runtime-kernel) | Rust | Spatial model: tensor grid, batons, assertion traps |

### Constraint / Theory Family

| Repo | Language | Description |
|------|----------|-------------|
| [categorical-agents](https://github.com/SuperInstance/categorical-agents) | Rust | Category theory for agent composition (functors, naturality) |
| [cuda-constraint-engine](https://github.com/SuperInstance/cuda-constraint-engine) | CUDA/C | GPU constraint checking at 1B+ constraints/sec |
| [grand-pattern-rs](https://github.com/SuperInstance/grand-pattern-rs) | Rust | Fibonacci dual-direction cellular graph architecture |
| [lau-hodge-theory](https://github.com/SuperInstance/lau-hodge-theory) | Rust | Hodge decomposition, Betti numbers, spectral sequences |
| [ternary-science](https://github.com/SuperInstance/ternary-science) | Rust | Experimental evidence for ternary intelligence, 5 conservation laws |

### Agent / Infrastructure Family

| Repo | Language | Description |
|------|----------|-------------|
| [construct-core](https://github.com/SuperInstance/construct-core) | Rust | Layered trait system: bare-metal → alloc → async agent runtime |
| [crab](https://github.com/SuperInstance/crab) | Bash | Agent shell for repo entry/leave (MUD-room metaphor) |
| [exocortex](https://github.com/SuperInstance/exocortex) | Rust | Persistent cognitive substrate, S3-compatible memory |
| [git-agent](https://github.com/SuperInstance/git-agent) | Python | The repo IS the agent — autonomous lifecycle via Git |
| [capitaine-1](https://github.com/SuperInstance/capitaine-1) | TypeScript | Git-native repo-agent, Cloudflare Workers heartbeat |
| [codespace-edge-rd](https://github.com/SuperInstance/codespace-edge-rd) | Research | Codespace→Edge agent lifecycle and yoke transfer protocols |
| [git-agent-codespace](https://github.com/SuperInstance/git-agent-codespace) | DevContainer | One-click Codespace template for Git-Agent runtimes |

### Registries

| Registry | Package | Install |
|----------|---------|---------|
| **PyPI** | `flux-vm` | `pip install flux-vm` |
| **crates.io** | `fluxvm` | `cargo add fluxvm` |
| **npm** | `flux-js` | `npm install flux-js` *(coming soon)* |

### Philosophy & Architecture

- 📖 [AI-Writings](https://github.com/SuperInstance/AI-Writings) — Philosophy, essays, and design rationale
- 📦 [PACKAGES.md](https://github.com/SuperInstance/SuperInstance/blob/main/PACKAGES.md) — Full package index
