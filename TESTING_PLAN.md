# Merix PHASE 1 — Comprehensive Testing Plan

**Version:** 1.0  
**Date:** April 29, 2026  
**Status:** PHASE 1 MVP (Foundation Runtime) — Local-first, resumable, self-extending AI runtime

This plan combines our internal component tests with industry best practices from AI agent frameworks (LangGraph, CrewAI, AutoGen, Letta, Ollama/llama.cpp ecosystems), persistent memory benchmarks, tool-calling evaluation, crash-recovery testing, and self-extension loops.

**Core Goals**

- Verify the **minimal self-contained intelligence loop** works end-to-end.
- Ensure everything is **resumable, crash-safe, and state-first-class**.
- Measure basic performance, memory usage, and tool/skill accuracy.
- Validate self-extension (history analysis → new skill generation).
- Catch regressions before any Phase 2 work begins.

## 1. Prerequisites

```powershell
# From project root
cargo clean
cargo check          # Must be zero errors / zero warnings
cargo build --bin merix-cli
```

Create a test data directory (ignored by .gitignore):

```powershell
mkdir -p data
```

## 2. Manual / CLI-Driven Tests (Core Smoke Tests)

Run these in order and record the output.

### 2.1 System Status

```powershell
cargo run --bin merix-cli -- status
```

### 2.2 Run a Full Task (TaskExecutor + MemoryLayer + Persistence)

```powershell
cargo run --bin merix-cli -- task "Test the full Merix PHASE 1 intelligence loop - create a task, persist it, and reconstruct context"
```

**Expected:** Session ID printed, “Task completed”, JSON file appears in `data/`.

### 2.3 Verify Persistence

```powershell
dir data\ -Recurse
dir data\memory\ -Recurse
```

Look for:

- `session_*.json` files
- `memory/merix.db` (SurrealDB RocksDB file)

### 2.4 Tool & Skill Discovery

```powershell
cargo run --bin merix-cli -- tool-list
cargo run --bin merix-cli -- skill-list
```

### 2.5 Self-Extension Loop

Copy the Session ID from step 2.2 and run:

```powershell
cargo run --bin merix-cli -- self-extend --session-id YOUR_SESSION_ID_HERE
```

**Expected:** “Analyzing history…”, “Generating new skill…”, “New skill registered”.

### 2.6 Resume Capability (basic)

```powershell
cargo run --bin merix-cli -- resume
```

### 2.7 Verbose Run (full trace)

```powershell
$env:RUST_LOG="info,merix=debug"
cargo run --bin merix-cli -- task "Second verbose test task"
```

## 3. Automated / Scripted Tests

Create these small test scripts in the root (or add to `tests/` later).

### 3.1 `test-resume.bat` (PowerShell)

```powershell
# test-resume.ps1
$env:RUST_LOG="info"
cargo run --bin merix-cli -- task "Create a resumable task that will be interrupted"
# Simulate crash by killing (manual for now)
cargo run --bin merix-cli -- resume
```

### 3.2 Memory Reconstruction Accuracy

Add a temporary test command later, but for now manually verify `reconstruct_context` returns meaningful output in logs.

## 4. Performance & Benchmarks (Inspired by Ollama/llama.cpp, SurrealDB crud-bench, Agent Memory Benchmarks)

Run these and record numbers.

```powershell
# Basic timing of a full task + self-extension
Measure-Command { cargo run --bin merix-cli -- task "Benchmark task" }

# Memory usage (Windows)
Get-Process -Name "merix-cli" | Select-Object Name, WorkingSet64
```

### Target Benchmarks (PHASE 1 expectations)

- Task round-trip (3 steps): < 2 seconds on typical hardware
- Self-extension loop: < 5 seconds
- MemoryLayer reconstruction: < 100 ms
- No memory leaks (monitor over 10+ runs)

## 5. Resilience & Edge-Case Testing

- **Crash Recovery:** Run a long task, kill the process mid-execution (`Ctrl+C` or Task Manager), then run `resume`.
- **Invalid Session ID:** Try `self-extend` with a fake UUID.
- **Empty Memory:** Run tools/skills before any tasks exist.
- **Concurrent CLI runs:** Open two terminals and run tasks simultaneously (Dashmap + SurrealDB should handle it).

## 6. Developer Usage Testing (Best Practices from Research)

- **Trajectory Evaluation:** Manually inspect logs for correct tool-call order and memory access.
- **LLM-as-Judge (future):** When a real LLM is wired in, add golden-set tests.
- **Golden Datasets:** Create a small set of expected outputs for the example tasks.
- **Observability:** All logs use `tracing` — enable `RUST_LOG=debug` for deep inspection.
- **Regression Suite:** Re-run the full sequence after any change to core/memory/mcp/skills.

## 7. Success Criteria (PHASE 1 Exit)

- All CLI commands succeed without panics.
- Persistent files are created and reloadable.
- Self-extension registers at least one new skill.
- `cargo check` and `cargo build` are clean (zero warnings).
- Basic resilience (crash + resume) works.
- No data loss or corruption.
