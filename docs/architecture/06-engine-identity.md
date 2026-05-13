# Engine Identity

This engine is simulation-first.

The goal is not to compete with general-purpose engines by having more features. The goal is to make simulation-heavy games easier to build, test, benchmark, debug, and preserve over time.

The engine should still have a generic ECS-based core, but its distinguishing focus is:

- deterministic fixed-tick simulation
- headless execution
- built-in benchmarking
- replay and time-travel-friendly debugging
- robust save/load support
- long-term save migration
- runtime introspection

## Target game types

The engine should be especially suitable for:

- factory games
- automation games
- colony sims
- city builders
- strategy games
- sandbox simulations
- server-authoritative simulations
- games with long-running save files

It does not need to be the best engine for every genre.

The aim is:

> Generic enough to build many games, opinionated enough to be excellent for simulations.

## Design personality

The engine should favor:

- explicit data flow
- deterministic systems where practical
- inspectable runtime state
- reproducible bugs
- stable save data
- benchmarkable behavior
- clear modular boundaries

The engine should avoid:

- magic runtime behavior
- hidden global state
- editor-only assumptions in runtime code
- render-first architecture
- inheritance-heavy entity design
- overabstracted enterprise patterns

## Standout features

### 1. Simulation-first runtime

The runtime should treat fixed-tick simulation as a first-class concern.

Important capabilities:

- fixed timestep update loop
- deterministic random number usage
- seedable simulation runs
- headless execution
- world-state hashing
- replayable inputs
- simulation snapshots
- benchmark scenarios

The engine should be able to run without a renderer.

Example use case:

```text
Run a 10,000 tick simulation benchmark and output a report without opening a window.
```

### 2. Time-travel-friendly debugging

The engine should be designed so simulation bugs can be reproduced and inspected.

Potential capabilities:

- record input streams
- record commands/events
- snapshot world state every N ticks
- replay from a snapshot
- step simulation tick-by-tick
- inspect changed entities/components per tick
- compare world state between ticks
- detect the first tick where an invariant breaks

This does not require full event sourcing of the entire engine. The core state should still live in ECS components and resources.

Time travel should be built from:

- deterministic ticks
- replayable inputs
- command/event logs
- periodic snapshots
- world-state hashes

### 3. Serious save/load and migration

The engine should treat long-lived saves as an important design problem.

Useful capabilities:

- stable entity IDs for persisted data
- stable asset IDs
- component schema versions
- explicit save migrations
- save compatibility tests
- partial world serialization
- deterministic load validation
- save diffing for debugging

Runtime entity IDs should not be used as persistent saved IDs.

Saved data should use stable identifiers that survive reloads, editor sessions, and engine updates.

## Practical priority

The standout features should not replace the boring foundation.

The core remains:

- ECS
- Data-Oriented Design
- scheduler
- plugins
- commands
- events
- resources
- asset handles

The simulation-first identity sits on top of that foundation.
