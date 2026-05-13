# Core Concepts

## App

The `App` is the composition root of the engine.

It owns:

- world state
- resources
- schedules
- plugins
- event registries
- asset systems
- runtime loop

Plugins configure the app.

## Plugin

A plugin is the main unit of engine extension.

A plugin may register:

- components
- resources
- systems
- events
- asset loaders
- render passes
- editor tooling, later

Plugins should not secretly depend on unrelated plugins unless that dependency is explicit.

## ECS

The ECS is the main world model.

- Entity: identity
- Component: data
- System: behavior over matching data
- World: storage for entities, components, and resources

Game objects should generally be composed from components instead of inheritance trees.

Example:

```text
Player = Transform + MeshRenderer + Collider + PlayerController
Camera = Transform + Camera
Light = Transform + Light
```

## Data-Oriented Design

Runtime systems should be designed around data access.

Prefer:

```
Transform + Velocity -> MovementSystem -> Transform
```

over object-style update chains.

The engine should make it clear which systems read and write which data. This allows better scheduling, parallelism, and reasoning.

## Scheduler

The scheduler controls when systems run.

Likely stages:

```
Startup
PreUpdate
FixedUpdate
Update
PostUpdate
RenderExtract
Render
Cleanup
```

The scheduler should eventually be able to reason about system ordering and conflicts.

## Commands

Commands are deferred world mutations.

They are used for operations such as:

- spawn entity
- despawn entity
- add component
- remove component
- modify hierarchy

Systems should generally not structurally mutate the world while queries are active. They should enqueue commands and let the engine apply them at safe points.

## Events

Events are transient messages used to decouple systems.

Good event examples:

- CollisionEvent
- DamageEvent
- WindowResized
- ButtonClicked
- AssetLoaded

Events should not replace normal ECS data flow. Most state should live in components or resources, not in event streams.

## Resources

Resources represent shared app/world state.

Examples:

- Time
- Input
- AssetServer
- WindowState
- RendererState
- GameSettings

Use resources for one logical instance of something. Use components for many instances.

## Assets

Assets should be referenced through handles, not direct file paths or raw loaded objects.

Core concepts:

- AssetId
- Handle<T>
- AssetServer
- AssetLoader
- Asset dependency graph, later

Game/runtime code should generally hold handles like:

```rust
Handle<Mesh>
Handle<Texture>
Handle<Material>
```

## Fixed-Tick Simulation

The engine should support fixed-tick simulation as a first-class mode.

Fixed ticks are important for:

- deterministic simulation
- physics
- replay
- benchmarking
- save/load validation
- server-authoritative simulation

The render loop should not be the owner of simulation time.

A renderer may run at variable framerate, but simulation should be able to run independently.

The engine should support:

```text
interactive mode:
  window + input + simulation + rendering

headless mode:
  simulation only

benchmark mode:
  deterministic simulation for N ticks with metrics output

replay mode:
  deterministic simulation from recorded input/command data
```

## Runtime Introspection

The engine should be able to explain what it is doing.

Useful introspection targets:

- registered plugins
- plugin dependencies
- registered systems
- system order
- system read/write access
- registered components
- registered resources
- registered events
- entity/component counts
- per-system timings
- command/event counts

This supports debugging, benchmarking, documentation, and AI-assisted development.

## World Hashing

The engine should eventually support hashing deterministic world state.

World hashes are useful for:

- determinism checks
- replay validation
- save/load validation
- multiplayer debugging
- regression tests

The hash should include deterministic simulation state, not renderer/editor-only state.

## Snapshots and Replays

The engine should be designed so replay and time-travel debugging can be added without replacing the ECS model.

The intended model is:

```text
ECS world = source of truth
input log = external simulation inputs
command/event log = useful debugging metadata
snapshots = periodic saved world states
world hashes = validation points
```

The engine should not require full event sourcing as the default state model.

## Persistent Identity

Runtime entity IDs are temporary.

Persistent data should use stable IDs.

Examples:

```text
Entity       = runtime identity
PersistentId = saved scene/save identity
AssetId      = stable asset identity
Handle<T>    = runtime reference to an asset
```

This is important for:

- save files
- scene files
- prefab references
- migrations
- replay validation
- editor support
