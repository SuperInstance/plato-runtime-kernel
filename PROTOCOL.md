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

To connect a runtime kernel room to a wire protocol engine block:

```rust
use plato_runtime_kernel::*;

// A baton arriving at an engine room carries protocol context
let mut baton = Baton::new("watchdog", "/engine_room");
baton.set_data("protocol_version", "0.1");
baton.set_data("room_id", "engine_room");

// The baton's tick corresponds to the engine block's sequence number
baton.advance_to("/wheelhouse");
// Baton.tick == 1 == engine block seq after one tick
```

The runtime kernel does not need to implement `tick`, `history`, `actuator`, `alarm`,
`subscribe`, or `quit` commands because it delegates those to the engine block layer.

## Cross-Audit Result

| Feature | Status | Note |
|---------|--------|------|
| JSON tick response | N/A | Engine block responsibility |
| JSON history response | N/A | Engine block responsibility |
| Alarm system | N/A | Engine block responsibility |
| Subscribe/unsubscribe | N/A | Engine block responsibility |
| Spatial topology | ✅ | This repo's purpose |
| Baton passing | ✅ | This repo's purpose |
| Assertion traps | ✅ | This repo's purpose |

*Last reviewed: 2026-07-12*
