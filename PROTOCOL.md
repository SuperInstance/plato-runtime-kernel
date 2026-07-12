# PLATO Wire Protocol Compliance

> **Status: Not Applicable — This is not an engine block.**

This repository (`plato-runtime-kernel`) is the **spatial layer** of the PLATO ecosystem.
It provides tensor grid topology, baton passing, and assertion traps — not sensor/actuator monitoring.

## Relationship to Wire Protocol

The PLATO Wire Protocol v0.1 governs communication between agents and **engine blocks**
(`plato-engine-block-c`, `plato-engine-block`, `plato-engine-block-elixir`, `plato-engine-block-zig`).

This runtime kernel sits **above** the wire protocol layer. It orchestrates which rooms
exist, how they're connected, and how agents move between them. Engine blocks handle
the physical layer (sensors, actuators, ticks over TCP).

```
Agent ←→ [Wire Protocol v0.1] ←→ Engine Block ←→ [Spatial Model] ←→ Runtime Kernel
```

## Bridge Pattern

The `wire` module provides bidirectional conversion between kernel types and wire protocol JSON:

```rust
use plato_runtime_kernel::*;
use plato_runtime_kernel::wire::*;

// 1. Parse an engine block welcome message into a RoomContract
let welcome = WireWelcome::from_json(
    r#"{"type":"welcome","room_id":"engine_room","tick_hz":0.2,"sensors":["coolant_temp_c","rpm"]}"#
).unwrap();
let contract = welcome.to_room_contract((0, 0));
// contract now has reflex_bindings for each sensor

// 2. A baton arriving from an engine room carries tick data
let mut baton = Baton::new("watchdog", "/engine_room");
baton.set_data("coolant_temp_c", "96.3");
baton.set_data("rpm", "1790");
baton.advance_to("/wheelhouse");

// 3. Serialize the baton as a wire protocol tick JSON for downstream agents
let tick_json = WireTick::from_baton(&baton).to_json();
// {"type":"tick","t":1749234437.0,"seq":1,"data":{"coolant_temp_c":96.3,"rpm":1790}}

// 4. Send commands to the engine block
let cmd = cmd_tick();       // "tick"
let cmd = cmd_history(20);  // "history 20"
let cmd = cmd_actuator("bilge_pump", 1.0); // "actuator bilge_pump 1"
```

The runtime kernel does not implement `tick`, `history`, `actuator`, `alarm`,
`subscribe`, or `quit` as protocol endpoints (it delegates those to engine blocks).
The `wire` module provides the **translation layer** so batons can carry
tick data through the spatial topology and the kernel can parse engine block welcome messages.

## Cross-Audit Result

| Feature | Status | Note |
|---------|--------|------|
| JSON tick response | ✅ Bridge | `WireTick::from_baton()` converts Baton to tick JSON |
| JSON history response | N/A | Engine block responsibility |
| JSON welcome parsing | ✅ Bridge | `WireWelcome::from_json()` + `to_room_contract()` |
| Alarm system | N/A | Engine block responsibility |
| Subscribe/unsubscribe | N/A | Engine block responsibility |
| Wire command builders | ✅ Bridge | `cmd_tick()`, `cmd_history()`, `cmd_actuator()`, etc. |
| Spatial topology | ✅ | This repo's purpose |
| Baton passing | ✅ | This repo's purpose |
| Assertion traps | ✅ | This repo's purpose |

*Last reviewed: 2026-07-12*
