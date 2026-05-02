# Merix PHASE 1 Development PLAN.md

*Last updated: May 01, 2026 — aligned with README.md v1.4*

4-Pillar Recursive Approach (CLI → Logging → Testing → Feature)

## Development Discipline

Follow **exactly** (never skip order):

### Three Pillars FIRST (strict sequence)

1. Pillar 1: CLI — Minimal working CLI foundation (clap, workspace, basic commands)
2. Pillar 2: Logging — Structured observability (tracing + tracing-subscriber)
3. Pillar 3: Testing — Comprehensive test framework (unit, integration, CLI E2E, resilience)

### Recursive Feature Loop (after Pillars are complete)

For every subsequent Phase 1 feature:

1. Implement the feature in the correct crate
2. Expose it via new CLI subcommand(s)
3. Add/update all tests
4. Run full test suite
5. Verify zero warnings, crash-safe, resumable
6. Mark checkbox **only after** tests pass
7. Repeat

## Test Commands (run after EVERY major step)

cargo check --workspace
cargo test --workspace
cargo run --bin merix -- --help

## Phase 1 Checklist

### Three Pillars

- [x] Pillar 1: CLI Foundation (workspace + crates/cli)
- [x] Pillar 2: Logging Infrastructure
- [x] Pillar 3: Testing Framework

### Feature Loop (one at a time, CLI + test before next)

- [x] Schemas crate (domain models)
- [x] MemoryLayer (Persistent SurrealDB + Ethereal Dashmap)
- [ ] Create resources crate — system resource management (CPU/GPU/VRAM/memory pressure)
- [ ] Create llama crate — llama-cpp-2 API, InferenceConfig, model loading, and GPU optimizations
- [ ] Core runtime
  - [ ] Task execution, session/task model
  - [ ] System resource management and optimizations
  - [ ] llama-cpp-2 implementation
- [ ] Project-wide code optimization pass
- [ ] Registry / MCP Tooling System
- [ ] Registry (unified registry for tools, skills, agents, capabilities)
- [ ] MCP Tooling System + registry
- [ ] Skills Registry & loading
- [ ] Planner stub
- [ ] Executor stub
- [ ] Agents crate (basic identity + permissions)
- [ ] Sandbox stub
- [ ] Reflection stub
- [ ] Full code review and benchmarks for additional optimization, security
- [ ] Full Phase 1 integration + end-to-end smoke test (single agent loop)
- [ ] Final stability verification

> We need to also implement proper GPU/VRAM scheduling and other optimization techniques during this phase
