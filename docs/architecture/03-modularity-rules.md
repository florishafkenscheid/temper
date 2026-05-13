# Modularity Rules

This file defines rules for keeping the architecture modular.

## 1. Core must stay small

The core should contain only concepts required by nearly every subsystem.

Allowed in core:

- App
- Plugin
- World
- Entity
- Component
- Resource
- System
- Schedule
- Commands
- Events
- Time/logging basics

Avoid putting rendering, physics, audio, editor, or game-specific logic in core.

## 2. Subsystems should be plugins

Rendering, input, assets, physics, audio, UI, and editor functionality should be added through plugins.

Example:

```rust
App::new()
    .add_plugin(InputPlugin)
    .add_plugin(AssetPlugin)
    .add_plugin(RenderPlugin)
    .add_plugin(GamePlugin)
    .run();
```

## 3. Dependencies must point inward

Subsystems may depend on core.

Core should not depend on subsystems.

Good:

```text
render -> core
physics -> core
game -> core
editor -> core + selected subsystems
```

Bad:

```text
core -> render
core -> physics
core -> editor
```

## 4. Avoid hidden coupling

A system should not assume another plugin exists unless that dependency is declared.

Bad:

```text
PhysicsPlugin silently requires TransformPlugin
```

Better:

```text
PhysicsPlugin declares dependency on TransformPlugin
```

or the required transform functionality lives in core if it is truly universal.

## 5. Prefer data boundaries over object boundaries

Do not design the engine around deep inheritance trees.

Avoid:

```text
GameObject
  Actor
    Character
      Enemy
```

Prefer:

```text
Entity + components
```

## 6. Use events for notifications, not ownership

Events are good for “something happened”.

They are bad as the primary storage of truth.

Good:

```text
CollisionEvent happened this frame
```

Bad:

```text
PositionChangedEvent is the only way to know where entities are
```

State belongs in components/resources. Events describe transitions or notifications.

## 7. Asset references should be handle-based

Runtime code should not depend directly on file paths.

Good:

```rust
MeshRenderer {
    mesh: Handle<Mesh>,
    material: Handle<Material>,
}
```

Bad:

```rust
MeshRenderer {
    mesh_path: String,
    texture_path: String,
}
```

## 8. Editor functionality should not pollute runtime core

The editor may need reflection, serialization, undo/redo, selection, gizmos, and inspectors.

These should be layered on top of the runtime architecture, not baked into every runtime type by default.

## 9. Abstract only where replacement is likely

Useful abstraction seams:

- renderer backend
- filesystem
- window/input backend
- audio backend
- asset storage

Avoid abstracting everything preemptively.

## 10. Every module must have a clear reason to exist

A module should either provide:

- a core engine primitive
- a subsystem plugin
- a backend implementation
- a game-facing API
- editor/tooling support

If a module only exists because an architecture pattern suggested it, remove or merge it.

## 11. Simulation must not depend on rendering

Simulation systems should be able to run without a renderer.

Good:

```text
movement system updates Transform
render extraction reads Transform later if rendering is enabled
```

Bad:

```text
movement system directly depends on renderer state
```

Headless execution should not require stubbing half the engine.

## 12. Deterministic state must be separated from presentation state

Simulation-relevant state should be clearly separated from renderer/editor-only state.

Examples of deterministic state:

- transforms used by gameplay
- health
- inventory
- simulation RNG state
- physics state
- world resources

Examples of non-deterministic or presentation state:

- GPU buffers
- editor selection
- debug UI open/closed state
- frame interpolation cache
- temporary render resources

Only deterministic state should participate in world hashing and replay validation.

## 13. Runtime entity IDs must not be persisted

Runtime entity IDs are allowed to be fast, compact, and temporary.

Saved data must use stable IDs.

Do not serialize raw runtime entity IDs as long-term references.

Use persistent IDs for:

- saves
- scenes
- prefabs
- replay references
- migrations

## 14. Benchmarks must be reproducible

Benchmark scenarios should define:

- seed
- tick count
- initial world/scenario
- enabled plugins
- simulation settings
- expected validation behavior, if any

Benchmark output should include enough metadata to reproduce the run.

## 15. Save migrations must be explicit

Save compatibility should not rely on accidental deserialization behavior.

When saved component schemas change, migrations should be explicit and testable.

A migration should state:

- source version
- target version
- affected type/component/resource
- transformation logic
- fallback/default behavior
