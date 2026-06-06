# plato-runtime-kernel

PLATO spatial spreadsheet runtime — room-as-cell tensor bridge, plain-English assertion traps, delta compression, three-way merge

## Overview

PLATO Runtime Kernel — the spatial spreadsheet engine.

Rooms are cells. Cells are tensors. Markdown is the AST.

## Architecture

This crate sits within the **five-layer Oxide Stack**:

| Layer | Crate | Role |
|-------|-------|------|
| 1 | open-parallel | Async runtime (tokio fork) |
| 2 | pincher | "Vector DB as runtime, LLM as compiler" |
| 3 | flux-core | Bytecode VM + A2A agent protocol |
| 4 | cuda-oxide | Flux→MIR→Pliron→NVVM→PTX compiler |
| 5 | cudaclaw | Persistent GPU kernels, warp consensus, SmartCRDT |

The key insight: **ternary values {-1, 0, +1} map directly to GPU compute**. They pack 16× denser than FP32, enable XNOR+popcount matmul, and conservation laws become compile-time checks.

## Stats

| Metric | Value |
|--------|-------|
| Tests | 24 |
| Lines of Code | 363 |
| Public API Surface | 29 items |
| License | MIT |

## Installation

```toml
[dependencies]
plato-runtime-kernel = "0.1.0"
```

## Usage

```rust
use plato_runtime_kernel::*;
// See src/lib.rs tests for complete working examples
```

### Key Types

```
- pub struct RoomIdentity {
- pub enum RoomDepth {
- pub struct RoomContract {
- pub struct RoomTopology {
- pub struct TraversalRecord {
- pub struct RuntimeAssets {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
    pub fn to_json(&self) -> String {
    pub fn is_adjacent(&self, room_id: &str) -> bool {
    pub fn record_traversal(&mut self, target: &str, baton_id: &str, tick: u64) {
```

## Design Philosophy

This crate uses **ternary algebra** (Z₃) where every value is {-1, 0, +1}:

- **+1** → positive signal (healthy, allocated, converged, ready)
- **0** → neutral (pending, balanced, monitoring, degraded)
- **-1** → negative signal (failed, free, diverged, overloaded)

This isn't arbitrary — ternary is the natural encoding for:
1. **BitNet b1.58** (Microsoft) — ternary neural networks at 60% less power
2. **GPU warp voting** — hardware ballot instructions return ternary consensus
3. **Conservation laws** — {-1, 0, +1} preserves quantity (what goes in must come out)

## Testing

```bash
git clone https://github.com/SuperInstance/plato-runtime-kernel.git
cd plato-runtime-kernel
cargo test
```

## License

MIT
