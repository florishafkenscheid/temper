# AI Context

This document summarizes the intended game engine architecture for future AI assistance.

The project is a Rust-first game engine experiment focused on learning architecture while keeping the implementation practical.

The preferred architecture is:

> A modular, plugin-driven ECS engine with data-oriented storage, scheduled systems, command-buffered mutation, event messaging, handle-based asset management, and simulation-first tooling.

The engine identity is:

> A simulation-first, inspectable Rust engine for deterministic, benchmarkable, save-heavy games.

Core concepts:

- ECS is the world model.
- Data-Oriented Design guides runtime data layout and system design.
- Plugins are the main extension mechanism.
- The scheduler controls system execution.
- Commands are used for deferred structural world mutation.
- Events are used for transient decoupled communication.
- Resources hold shared app/world state.
- Assets are referenced through stable handles.
- Fixed-tick simulation should be first-class.
- Headless execution should be supported.
- Benchmarks should be reproducible.
- Runtime state should be inspectable.
- Persistent state should use stable IDs, not runtime entity IDs.
- Save/load and migration should be treated as serious architecture concerns.

The engine should stand out through:

- deterministic simulation support
- benchmarkable runs
- replay-friendly design
- time-travel-friendly debugging
- robust save/load and migration
- runtime introspection
- modularity auditing
- AI-readable generated context

Avoid suggesting an enterprise-style architecture as the foundation.

Do not make the engine primarily:

- DDD-based
- CQRS-based
- Clean Architecture-based
- Actor-model-based
- fully event-sourced
- inheritance-heavy OOP
- render-first

Those patterns may be useful locally, but they should not define the engine core.

Useful local applications:

- CQRS-like commands are useful for editor undo/redo and ECS command buffers.
- DDD can help with game-specific logic or editor/project concepts.
- Hexagonal architecture can help with backend/platform abstraction.
- FRP/MVVM can help with editor UI.
- Event sourcing can help with replay/debugging/undo, but should not be default runtime state.
- Behavior trees and blackboards belong in an AI plugin or game layer, not the engine core.

Current priority:

1. App and plugin system
2. ECS world model
3. Resources
4. Systems and scheduler
5. Fixed timestep support
6. Command buffers
7. Events
8. Headless execution path
9. Basic benchmark runner
10. Asset handles
11. Basic input/windowing
12. Basic rendering
13. Stable IDs for persisted state
14. Scene/save serialization later

Modularity rules:

- Keep core small.
- Put subsystems behind plugins.
- Dependencies should point inward toward core.
- Simulation must not depend on rendering.
- Deterministic state must be separated from presentation/editor state.
- Avoid hidden plugin coupling.
- Prefer composition over inheritance.
- Use events for notifications, not state ownership.
- Use asset handles instead of file paths in runtime components.
- Do not persist raw runtime entity IDs.
- Do not bake editor-only concerns into the runtime core too early.
- Abstract only where replacement or platform variation is likely.
