# Architecture Thesis

The engine should be built around a small number of load-bearing concepts:

- ECS for the world model
- Data-Oriented Design for runtime data flow
- Scheduler for system execution
- Plugins for modular extension
- Command buffers for deferred world mutation
- Events for decoupled frame-local communication
- Resources for shared world/app state
- Asset handles for content references
- Stable IDs for persisted state
- Headless simulation as a first-class runtime mode

The architecture should avoid making optional patterns part of the foundation.

## Core thesis

> The engine is a modular ECS runtime where plugins register components, resources, systems, events, asset loaders, simulation tooling, and rendering functionality into an application schedule.

## Engine identity

The engine is simulation-first.

It should make simulation-heavy games easier to build, test, benchmark, debug, replay, and preserve over time.

The distinguishing features are:

- deterministic fixed-tick simulation
- benchmarkable runs
- headless execution
- replay-friendly input/command/event recording
- time-travel-friendly debugging
- robust save/load design
- explicit save migration support
- runtime introspection

## What this architecture is not

This is not primarily:

- DDD architecture
- CQRS architecture
- Clean Architecture
- Actor-model architecture
- Event-sourced architecture
- inheritance-heavy OOP architecture
- render-first engine architecture

Those patterns may still be useful locally, but they should not define the engine core.

## Practical design rule

A concept belongs in the core only if almost every game built with the engine benefits from it, or if it directly supports the simulation-first identity.

Core examples:

- ECS
- scheduling
- plugins
- assets
- commands
- events
- fixed timestep
- headless execution
- stable saved identifiers

Local or optional examples:

- behavior trees
- blackboards
- full event sourcing
- MVVM
- scripting
- networking
- editor undo/redo
