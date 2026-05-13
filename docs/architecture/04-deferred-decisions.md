# Deferred Decisions

This file tracks architectural decisions that are intentionally not part of the first implementation.

## Reflection / Type Registry

Useful for:

- editor inspectors
- generic serialization
- scripting
- debug UI
- prefab editing

Deferred because the first engine version can work with manually registered types.

## Serialization, Saves, and Migration

Useful for:

- scenes
- prefabs
- materials
- project files
- editor state
- long-running save files
- replay snapshots
- benchmark scenarios

Initial serialization may be simple and explicit. Reflection-driven serialization can come later.

Important future concerns:

- runtime entity IDs should not be used as persistent saved IDs
- assets need stable IDs
- scenes/prefabs need stable references
- component schemas need versions
- save migrations should be explicit
- save/load should be testable through deterministic world hashes

Possible future capabilities:

- component-level schema versions
- migration registry
- save compatibility tests
- save diffing
- partial world serialization
- snapshot serialization for replay/debugging

## Replay and Time-Travel Debugging

Useful for:

- reproducing simulation bugs
- deterministic testing
- regression checks
- multiplayer debugging, later
- benchmark validation
- AI-assisted bug reports

Deferred because it depends on:

- fixed-tick simulation
- stable input representation
- command/event logging
- deterministic world state
- snapshot serialization
- world hashing

The engine should not use full event sourcing as the default runtime model.

Preferred future model:

```text
periodic snapshots
+ recorded inputs
+ command/event metadata
+ world hashes
= replay/time-travel-friendly debugging
```

## Benchmarking and Headless Execution

Useful for:

- simulation-heavy games
- performance regression testing
- CI
- deterministic validation
- stress testing
- comparing architectural changes

Deferred as a polished feature, but the engine should be designed for it from the start.

Important design implication:

> Simulation must be able to run without rendering, windowing, or editor state.

## Render Graph

Useful for:

- shadow passes
- post-processing
- deferred rendering
- resource lifetime tracking
- GPU synchronization
- advanced renderer extensibility

Deferred until the renderer has enough complexity to justify it.

## Job System

Useful for:

- parallel ECS systems
- asset loading
- background imports
- render preparation
- physics work

Deferred until single-threaded scheduling becomes limiting.

The current design should still avoid APIs that make parallelism impossible later.

## Editor

Useful for:

- scene editing
- asset inspection
- entity hierarchy
- gizmos
- play mode
- undo/redo

Deferred because editor architecture depends on ECS, assets, serialization, and reflection.

## Scripting

Potential options:

- native Rust systems
- Lua
- Rhai
- WASM
- C#

Deferred because scripting requires stable engine APIs, reflection, world access rules, and asset bindings.

## Networking

Out of scope for the initial architecture.

Networking may require:

- deterministic simulation
- replication
- rollback
- prediction
- snapshots
- authority model

These decisions should not shape the first core unless networking becomes a primary goal.
