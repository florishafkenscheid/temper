# Context

This project is a Rust-first game engine experiment focused on learning serious software architecture while staying practical.

The engine is not intended to be a generic clone of existing engines. Its identity is simulation-first: deterministic, inspectable, benchmarkable, and friendly to long-running save files.

The current architectural direction is:

> A modular, plugin-driven ECS engine with data-oriented storage, scheduled systems, command-buffered mutation, event messaging, handle-based asset management, and simulation-first tooling.

The engine should be especially good for simulation-heavy games such as factory games, automation games, colony sims, city builders, strategy games, and sandbox simulations.

## Assumptions

- The engine should be generic enough to support different game types.
- The engine does not need to be equally optimized for every genre.
- Rust is the primary implementation language.
- Runtime performance and data layout matter.
- Deterministic fixed-tick simulation is important.
- Headless execution should be a first-class use case.
- Benchmarking and reproducibility should be built into the workflow.
- Long-term save/load support should influence architecture.
- Editor support is desirable, but not the first implementation target.
- Modularity matters more than feature count.
- The architecture should teach useful theory without becoming over-engineered.

## Known gaps

- Rendering backend has not been chosen.
- ECS implementation strategy has not been finalized.
- Asset pipeline format has not been finalized.
- Save format and migration strategy have not been finalized.
- Replay/snapshot implementation has not been finalized.
- Editor architecture is intentionally deferred.
- Scripting is not a core goal yet.
- Networking is out of scope for the initial architecture.
