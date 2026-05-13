# ECS Storage Direction

This document records the intended ECS storage direction for the first implementation.

The public ECS API should not expose the storage implementation. Game and engine code should work in terms of:

- `World`
- `Entity`
- `Component`
- `Query`
- `Commands`
- `Resources`
- `Systems`

Storage choices should remain internal so the engine can evolve without forcing game code to change.

## Entity identity

Runtime entities should use compact generational IDs from the first ECS implementation.

An `Entity` contains:

- index
- generation

The index addresses an entity slot. The generation prevents stale references from accidentally targeting a newly spawned entity after a slot is reused.

The entity allocator should track:

- entity slots
- alive/dead state
- generation values
- free list for recycled indices

Despawning an entity should:

1. validate that the entity is currently alive
2. mark the slot dead
3. increment the generation
4. recycle the index

Runtime entity IDs are transient. They must not be used as saved scene, save file, prefab, replay, or migration identity.

Saved state should later use a separate stable identity system, such as `PersistentEntityId`.

## Table storage

The first ECS storage implementation should prioritize chunked archetype/table storage for normal components.

Entities should be grouped by their table component set. Each unique table component set maps to an archetype/table.

Each archetype should be split into chunks. A chunk stores dense columns for each component type in that archetype.

This gives the engine a data-oriented default path for common simulation components:

```text
Archetype: Position + Velocity + Lifetime
  Chunk 0:
    Position[]
    Velocity[]
    Lifetime[]
  Chunk 1:
    Position[]
    Velocity[]
    Lifetime[]
```

Adding or removing a table component changes archetype membership. The entity should move from one archetype/table to another while preserving the components that remain valid.

## Sparse side components

Sparse components are intentionally deferred until the core table path works.

The design should remain open for sparse side storage later.

Sparse components should not affect archetype membership. They are intended for data such as:

- high-churn tags
- editor-only markers
- dirty flags
- rare optional metadata
- relationship-like data, if that proves useful

Mixed table+sparse queries can be added after table storage, entity movement, and table queries are working.

## Separate design axes

The engine should keep these design axes separate:

```text
storage:      table/chunked archetype vs sparse
persistence:  saved stable IDs vs runtime transient IDs
determinism:  simulation state vs presentation/editor/debug state
```

A component being table-stored does not automatically mean it is persisted.

A component being persisted does not automatically mean it participates in deterministic simulation hashing.

A component being sparse does not automatically mean it is editor-only or non-deterministic.

Each concern should be modeled explicitly instead of being inferred from another axis.

## Deterministic simulation state

World hashing, replay validation, save validation, and benchmark validation should include deterministic simulation state only.

Presentation, editor, debug, render, and visual-only state should be excluded from deterministic hashes.

Visual particles may use floats later as presentation-only state.

Simulation particles should use deterministic simulation coordinates, such as integer or fixed-point values, or a documented deterministic numeric wrapper with strict platform and math caveats.

The first particle demo should model simulation particles as deterministic ECS entities, not visual particles.
