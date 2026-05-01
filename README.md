# Merix

**Version 3.1**

**The rogue messenger that delivers forbidden intelligence.**

One clean desktop binary.  
Zero cloud. Zero daemons.  
Everything is data.
Pure local intelligence.

---

## The Problem

Today’s best local AI tools are powerful but fragmented:

- LM Studio provides usability and model management
- Agent frameworks provide orchestration and memory
- Coding tools provide execution capabilities

But none of them form a single coherent system.

Workflows break. Context is lost. Tools do not compose cleanly.

Merix exists to unify this.

---

## The Vision

Merix is a local-first AI runtime that evolves from:

- inference engine
- to workflow system
- to self-extending computational environment

It is designed as:

> **a machine that builds the machine that builds the work**

---

**Development Discipline (v3.1):**

- All work should follow a `PLAN.md` checklist that is designated for that Phase/Feature.
- One checkbox (feat) at a time, e.g.,: Full test (`cargo check --workspace` + CLI smoke tests + resilience checks), before moving forward.
- Never implement any later-phase feature until the previous phase is 100% complete and stable.

### Architecture Principles (Non-Negotiable)

- Everything is resumable
- Nothing runs unless required
- Tools are explicit and sandboxed
- Skills define behavior, not prompts
- The system must be capable of extending itself
- State is first-class
- Modular crate design (each subsystem in its own crate)
- MemoryLayer is separated: `PersistentMemory` (SurrealDB 3.0 + RocksDB) + `EtherealMemory` (Dashmap)
- MCP Tooling & Agent Skills must be importable, exportable, and able to be designed "in-app".

The overall capabilities of Merix should include the following:

```text
Merix/
├── adaptation/             # Real-time adjustments for dynamic strategy switching, fallback behaviors and performance tuning mid-task for agents.
├── agents/                 # Provides identity, permission scopes, capability ownership and audit trails
├── alignment/              # Ensures agents act within user goals, constraints, and system objectives.
├── app_connect/            # Connects cloud apps for AI use (Cloud AI, Google, APIs, etc...)
├── attention/              # Provides context prioritization, tool selection filtering, dynamic context compression
├── capabilities/           # Allows for skill generation via LLM codegen, validation loop and versioning
├── cli/                    # Minimal Control Interface
├── communication/          # A2A Protocol Layer - handles structured agent messaging, negotiation protocols, shared context exchange
├── compatibility/          # Handles model differences, tool schema evolution and backwards compatibility.
├── context/                # Handles context assembly, prioritization and truncation strategies. See note.
├── coordination/           # Similar to orchestration, but handles conflict resolution, resource arbitration, multi-agent synchronization (see note)
├── core/                   # Task execution + LLM runtime (llama.cpp)
├── deployment/             # Allows the packaging of agents, distributing skills, updating components and version rollbacks
├── economics/              # Handles model selection, tool cost budgeting, execution optimization
├── environment/            # Abstracts OS interaction, browser automation and file system control.
├── executor/               # Defines execution graphs (DAGs), step scheduling, retry/failure handling, determinism boundaries. See note.
├── experience/             # Stores trajectories (task -> actions -> outcomes) and enables reuse of successful patterns and powers learning systems.
├── evaluation/             # Scoring outputs, benchmarking tasks and regression detection for when agents modify their own behavior and skills evolve.
├── events/                 # Provides a pub/sub event bus for system-wide signals for observability, decoupling, real-time adaptation. See note for examples.
├── governance/             # Policy enforcement, risk thresholds, approval systems constraint validation
├── intent/                 # Parses user goals into constraints, success criteria, sub-goals to prevent vague execution and enables verification
├── interop/                # Normalizes external interfaces and prevents pollution of core logic when using Apps
├── interface/              # Allows for voice, real-time streams, multimodal input interactions with the agent(s)
├── knowledge/              # Structured knowledge reasoning using knowledge graphs, entity linking and semantic relationships.
├── memory/                 # Persistent (SurrealDB) + Ethereal (Dashmap) — (separated into 2 implementations)
├── observability/          # Provides traces, spans, decision logs, replay systems
├── orchestrator/           # Handles agent-to-agent protocols (A2A), role assignment, concensus / arbitration (see note)
├── persona/                # Handles "what the agent becomes" - personality/behavior profiles, long-term preferences, communication style, role specialization.
├── planner/                # Prompts LLM for planning, validates plan and provides cost/complexity estimation. See note.
├── rag/                    # File embedding provider (choose "data location" and it will create embeddings & add them to the DB) for ingestion.
├── recovery/               # Structured error handling, retry strategies, fallback planning to prevent hallucinations, tool errors, timeouts.
├── reflection/             # History analysis, what worked, what failed, performance per skill/tool. See note.
├── registry/               # Provides a registry for tools, skills, agents, and anything else added with versioning, capability metadata and search
├── relationships/          # Tracks agent-to-agent relationships, trust scores / reliability, delegation history.
├── resources/              # Provides CPU/GPU allocation, memory pressure and concurrency limits.
├── sandbox/                # Security + Execution Isolation. Isolates execution, resource limits and failure containment.
├── schemas/                # Database & In-Memory data structures (Session, Task, Checkpoint, Skill, etc.)
├── security/               # System-wide protection and handles secrets, auth, encryption, attack prevention for both incoming and outgoing
├── server/                 # Provides VPN-like and E2EE capabilities for various communication apps and direct communications to a `Merix-Server`.
├── skills/                 # Skills registry & loading
├── simulation/             # Allows agents to simulate outcomes, test plans, estimate risk within a sandbox for planning validation and "what-if" reasoning
├── state/                  # Provides the current system snapshot. Handles active tasks, agent states, execution context.
├── time/                   # Schedulers, delayed execution and recurring tasks. Allows agents to schedule tasks, revisit goals, maintain long-term objectives.
├── utilities/              # Utility implementations such as logging.
├── verification/           # The "Trust Layer" - gives output validation, plan verification, constraint checking
├── workflow/               # Provides long-running workflows (hours/days/weeks), checkpointing, resumability, distributed execution across agents
└── world_model/            # Enables planning accuracy, simulation quality, long-term reasoning. See note.
```

> The executor's responsibility is to convert plans -> executable graphs. Those graphs run independent of LLM.

> The planner allows us to swap planning strategies and add symbolic planners

> Examples of the events crate are `TaskStarted`, `ToolFailed`, `MemoryUpdated`, `SkillLearned`

> The context crate is essential for growing context windows and for multi-agent coordination

> The reflection crate also tracks outcomes, adjusts routing decisions and evolves behavior over time.

> The world_model builds internal representations (entities, systems, dependencies), tracks causal relationships, supports reasoning beyond text.

> Orchestration handles planning/control.

> Coordination handles real-time interactions.

## PHASE 1 — PRIMARY MVP (Foundation Runtime)

### Core Objective

A stable local AI runtime that can execute tasks, persist state, resume execution, safely invoke tools via MCP, and evolve behavior via Skills — forming the **minimal self-contained intelligence loop**.

### Directory Structure (MCP release - evolve over time)

To support the above capabilities, this is the current proposed directory structure:

```text
Merix/
├── crates/
│   ├── core/              # Task execution, LLM runtime, basic session/task model
│   ├── memory/            # Persistent (SurrealDB) + Ethereal (Dashmap) — (separated into 2 implementations)
│   ├── mcp/               # MCP Tooling System + registry
│   ├── skills/            # Skills loading, registry, self-generation
│   ├── cli/               # Minimal Control Interface
│   ├── agents/            # Agent identity, persona, permissions, relationships
│   ├── planner/           # Planning, intent parsing, plan validation
│   ├── executor/          # Execution graphs, scheduling, retry, recovery
│   ├── reflection/        # Evaluation, reflection, learning from trajectories
│   ├── sandbox/           # Security, isolation, resource limits
│   ├── schemas/           # Domain models for Database & In-Memory data structures (Session, Task, Checkpoint, Skill, etc.)
│   ├── registry/          # Unified registry for tools, skills, agents, capabilities
│   ├── observability/     # Events, tracing, logging, metrics
│   ├── knowledge/         # RAG, embeddings, knowledge graph, world_model
│   ├── governance/        # Alignment, verification, policy enforcement, economics
│   └── utilities/         # Shared helpers (keep small)
├── tests/
├── PLAN.md
├── README.md
├── Cargo.toml
├── LICENSE
└── (data/ logs/ generated)
```

> The above structure and phase 1 should allow us to use a single agent for further development.

### Libraries & Frameworks

The following frameworks and libraries will be used within Merix. This is the **exact minimal set** for **Phase 1** (Foundation Runtime) and will be declared in the root `Cargo.toml` under `[workspace.dependencies]`.

- `surrealdb = { version = "3.0", features = ["kv-rocksdb"] }` → Embedded multi-model database (persistent long-term memory + native vector indexing)
- `dashmap = { version = "6.1" }` → High-performance concurrent HashMap for ethereal/short-term memory
- `serde = { version = "1.0", features = ["derive"] }` → Serialization / deserialization for all domain models and checkpoints
- `serde_json = "1.0"` → High-performance JSON handling (MCP manifests, checkpoints, state snapshots)
- `anyhow = "1.0"` → Simple, ergonomic error handling across the runtime
- `async-trait = "0.1"` → Async trait support for Tool, Skill, and executor interfaces
- `chrono = { version = "0.4", features = ["serde"] }` → Date/time handling with full serialization support
- `clap = { version = "4.5", features = ["derive"] }` → Fully-featured command-line argument parser for the CLI
- `tokio = { version = "1", features = ["full"] }` → Event-driven, non-blocking I/O and async runtime
- `tracing = "0.1"` → Structured, event-based diagnostic information (observability foundation)
- `tracing-subscriber = "0.3"` → Utilities for implementing and composing tracing subscribers (file + console logging)
- `uuid = { version = "1.10", features = ["v4", "serde"] }` → Unique identifiers for sessions, tasks, checkpoints, and skills
- `llama-cpp-2 = "0.1"` → Production-ready Rust bindings for llama.cpp (LLM inference engine — user-provided GGUF models)

> SurrealDB + serde_json::Value conflict heaviliy due to serialization friction. All code should use the explicit `serde_json::to_value(&struct)?` pattern before calling `.content()` / `.upsert()` on SurrealDB. No `SurrealValue` derive is used. However, this needs to be validated as stable and crash-safe.

**Notes for Phase 1:**

- No additional heavy dependencies will be added until Phase 1 is 100% complete and tested.
- All crates use the latest stable versions as of April 2026.
- The workspace will remain minimal and fast to compile.
- `llama-cpp-2` is the actively maintained binding that matches the exact requirements in the README (user-provided models, CPU+GPU, deterministic sessions).

> Once phase 1 is complete, use the `Qwen3 / Qwen3.5-14B-Coder` model to assist with phase 2 and above.

---

## PHASE 2 — Visual Studio Code Plugin

**The developer’s bridge between raw CLI power and polished desktop UX.**

### Objective

Turn VS Code into the **primary interface** for Merix. Developers should be able to chat with Merix, run tasks, invoke skills/tools, view memory, self-extend the system, and perform context-aware operations directly inside their editor — without ever leaving VS Code or relying on external services.

This phase transforms Merix from a powerful but CLI-only runtime into a seamless, always-available development companion that lives where developers already work.

### Core Capabilities

1. **Official VS Code Extension**  
   - Built with the official VS Code Extension API (TypeScript + Node.js).  
   - Supports both Webview-based chat panel **and** native LSP integration for maximum performance and responsiveness.  
   - Zero external dependencies — fully local and offline.

2. **Sidebar Chat Interface**  
   - Persistent Merix chat panel (similar to GitHub Copilot Chat or Continue.dev).  
   - Full conversational capabilities with streaming responses.  
   - Supports Markdown rendering, code blocks with one-click insert/apply, and threaded conversations.

3. **Local Communication with Merix Runtime**  
   - Connects to the running Merix core via:  
     - Named pipe (Windows) / Unix socket (macOS/Linux) for lowest latency, or  
     - Local HTTP server (as fallback).  
   - Secure, permission-checked communication using the same MCP protocol used by the CLI.

4. **Agent Capabilities Inside VS Code**  
   - Execute any registered Skill or MCP Tool directly from chat.  
   - Context-aware operations:  
     - “Ask Merix about this code” on any selection or file.  
     - Analyze current workspace, open tabs, git diff, or specific folders.  
     - Refactor function, add tests, explain error, generate documentation, etc.  
   - One-click commands for common developer tasks.

5. **Deep Integration with Merix Runtime**  
   - Real-time access to MemoryLayer (semantic + structured recall).  
   - Full visibility into ToolRegistry, SkillRegistry, and Self-Extension Core.  
   - Task creation, resume, and self-extend commands directly from the editor.  
   - Session browser and history viewer inside VS Code.

6. **Advanced Productivity Features**  
   - Command palette integration (`Merix: Run Task…`, `Merix: Self-Extend`, `Merix: List Skills`, etc.).  
   - Inline code suggestions and edits powered by the local LLM.  
   - Session timeline view showing tasks, checkpoints, and skill generations.  
   - “Ask Merix about this error” on terminal output or problems panel.  
   - Git-aware context (automatically includes diff when relevant).

### Technical Approach & Development Guide

**Tech Stack**

- TypeScript + VS Code Extension API (v1.XX+)
- Webview + React (for rich UI) or pure Webview + Lit for lighter footprint
- LSP (Language Server Protocol) for advanced code intelligence
- Communication layer: `@vscode/vscode-lsp` + custom JSON-RPC over named pipe/socket
- Reuse of existing Merix Rust crates via the CLI binary or direct library calls

**Development Milestones**

1. **Phase 2.1** — Extension skeleton + basic chat panel + connection to running Merix CLI
2. **Phase 2.2** — Full MCP/Skill/Tool execution from chat
3. **Phase 2.3** — Context-aware commands (current file, selection, workspace, git)
4. **Phase 2.4** — Deep integration with MemoryLayer and Self-Extension
5. **Phase 2.5** — Inline suggestions, command palette, session browser, and polish
6. **Phase 2.6** — Packaging, distribution, and documentation

**Dependencies**

- Requires a fully working Phase 1 runtime (MemoryLayer, MCP, Skills, Self-Extension, CLI).
- The extension will start the Merix runtime automatically if it is not already running (via `merix-cli` binary).

**Success Criteria**

- All Phase 1 CLI commands are available and work identically from within VS Code.
- Zero cloud or external service dependency.
- Extension loads in < 500 ms and maintains low idle memory usage.
- Developers can complete a full “create skill → self-extend → use new skill” loop entirely inside VS Code.
- The extension itself becomes a first-class Merix use-case (Merix helps develop Merix).

**Future Extensibility**

- Designed to allow third-party extensions to register custom Merix commands.
- Easy migration path to full Tauri desktop app (Phase 3) by reusing the same communication layer.

---

## PHASE 3 — Desktop Application (User Experience Layer)

**Objective:** Transform Merix from a powerful runtime + VS Code companion into a **coherent, polished AI workstation** that feels like a native desktop application. The user should have a beautiful, responsive, always-available interface that makes the full intelligence loop (tasks, memory, tools, skills, self-extension) feel effortless and delightful.

This phase turns Merix into the primary AI environment for daily work — no more switching between terminal, VS Code, and scattered tools.

### Core Capabilities

1. **Desktop Application Layer**  
   - Built with **Tauri v2** (Rust backend + lightweight native shell) + React + TypeScript frontend.  
   - Single executable installer for Windows, macOS, and Linux.  
   - Extremely low idle resource usage and fast startup (< 800 ms).

2. **Workspace System**  
   - Project-based environments with automatic discovery of existing folders.  
   - Built-in file explorer with git integration (status, diff preview, commit assistance).  
   - Multi-file editing with live Merix context (Merix understands the entire workspace).

3. **Agent UX Layer**  
   - Structured planning mode with visual step-by-step execution graphs.  
   - Real-time multi-step visualization (task → steps → checkpoints → outputs).  
   - Full transparency: tool invocation logs, permission prompts, and skill execution history.  
   - Session timelines with searchable history, resume buttons, and one-click self-extend.

4. **Vision + Input Extensions**  
   - Drag-and-drop support for images, PDFs, logs, and code files.  
   - Screenshot understanding (capture any part of the screen and ask Merix to analyze it).  
   - Log file ingestion and automatic error diagnosis.  
   - Multimodal context: combine text, images, code, and memory in a single prompt.

5. **Optional Voice Interface**  
   - Local TTS/STT integration (using Whisper.cpp + Piper or equivalent).  
   - Hands-free conversational mode.  
   - Voice-activated task creation and status queries.

6. **UX Polish Layer**  
   - Dark-first, modern interface with smooth animations and responsive design.  
   - Preset modes (Coding, Planning, Analysis, Research, Self-Extension).  
   - Keyboard shortcuts, quick commands, and customizable layouts.  
   - Seamless integration with the VS Code plugin (same backend, different frontends).

### UX Technical Approach & Development Guide

**Tech Stack**

- Tauri v2 (Rust core + secure WebView)
- React + TypeScript + TanStack Query + Zustand (state management)
- TailwindCSS + shadcn/ui for beautiful, consistent components
- Reuse the exact same Rust crates from Phase 1 (core, memory, mcp, skills, self-extension) via Tauri commands
- Communication layer: Tauri IPC (zero-overhead, type-safe calls between frontend and Rust backend)

**Development Milestones**

1. **Phase 3.1** — Tauri v2 skeleton + basic chat window + connection to Merix runtime
2. **Phase 3.2** — Workspace system + file explorer + git awareness
3. **Phase 3.3** — Agent UX Layer (planning view, execution visualization, timelines)
4. **Phase 3.4** — Vision + Input Extensions (drag-and-drop, screenshot, multimodal)
5. **Phase 3.5** — Voice interface + UX polish + preset modes
6. **Phase 3.6** — Packaging, installer, cross-platform testing, and documentation

**Dependencies**

- Requires fully completed and stable Phase 1 + Phase 2.
- The desktop app can run standalone or alongside the VS Code extension (both share the same local runtime).

**Success Criteria**

- The desktop app feels faster and more integrated than any cloud AI tool.
- Full parity with all Phase 1 CLI commands and Phase 2 VS Code features.
- Zero cloud dependency — everything runs 100% locally.
- Resource usage stays under 300 MB idle and under 2 GB during heavy tasks.
- A new user can complete an end-to-end “task → self-extend → use new skill” loop entirely in the desktop UI.
- The app itself becomes a showcase of Merix’s self-extension capabilities (e.g., Merix can help improve its own UI).

**Future Extensibility**

- Plugin system for third-party panels and custom views.
- Easy path to mobile or web versions by reusing the same Tauri/React codebase.
- Designed to support multi-window workflows and advanced collaboration features in later phases.

---

## PHASE 4 — Evolving Systems (Autonomous Capability Layer)

**Objective:** Transform Merix from a powerful tool into a **self-improving, autonomous intelligence system** that grows its own capabilities during use. Instead of just executing tasks, Merix will discover missing abilities, generate new skills and tools, optimize its own workflows, and continuously evolve into a more capable version of itself — all while remaining 100% local and fully under user control.

This is the phase where Merix becomes the “machine that builds the machine that builds the work.”

### Core Capabilities

1. **Dynamic Skill Generation**  
   Agents automatically analyze task history and repeated patterns, then generate, validate, version, and register new skills directly into the Skill Registry.

2. **Autonomous Tool Acquisition (MCP Expansion Layer)**  
   Runtime discovery of new MCP tools, automatic sandboxed installation, and negotiation of permissions. Merix can acquire or create the exact tools it needs to complete a task.

3. **Secure Execution Sandbox**  
   Full isolation of all tool and skill execution. Includes prompt-injection detection, capability-based permissions, resource limits, signed verification of generated skills/tools, and safe rollback.

4. **Self-Improving Workflow System**  
   Workflows evolve automatically based on usage patterns. Redundant steps are eliminated, successful paths are reinforced, and the system optimizes its own planning and execution over time.

5. **Recursive Capability Loop**  
   The core innovation of Merix:  
   **Receive task → Identify missing capabilities → Generate or acquire skills/tools → Execute → Evaluate gaps → Persist improvements → Repeat.**  
   This creates continuous, measurable self-improvement without any cloud dependency.

### Technical Approach & Development Guide

**Prerequisites**  

Phase 4 can only begin after **Phase 1 (Foundation Runtime), Phase 2 (VS Code Plugin), and Phase 3 (Desktop Application)** are complete and stable.

**Key Architectural Principles (Non-Negotiable)**

- Everything remains resumable and crash-safe
- Tools are explicit and sandboxed
- Skills define behavior, not prompts
- State is first-class and fully persisted
- The system must be capable of extending itself
- Modular crate design with clear separation of concerns

**Target Long-Term Architecture (Post-Phase 1)**  

The final Merix system will be organized into focused, composable crates that implement the full autonomous capability layer:

```text
Merix/
├── adaptation/             # Real-time strategy switching, fallback behaviors, performance tuning
├── agents/                 # Agent identity, permission scopes, capability ownership, audit trails
├── alignment/              # Ensures agents stay within user goals, constraints, and objectives
├── app_connect/            # Cloud app integrations (Google, APIs, etc.) – optional and permissioned
├── attention/              # Context prioritization, tool selection, dynamic context compression
├── capabilities/           # Skill generation via LLM codegen, validation loop, and versioning
├── communication/          # A2A Protocol Layer – structured messaging and shared context
├── coordination/           # Multi-agent synchronization, conflict resolution, resource arbitration
├── core/                   # Task execution + LLM runtime (llama.cpp)
├── deployment/             # Packaging, distributing skills, version rollbacks
├── economics/              # Model selection, cost budgeting, execution optimization
├── environment/            # OS interaction, browser automation, file system control
├── executor/               # Execution graphs (DAGs), scheduling, retry, determinism
├── experience/             # Trajectory storage and reuse of successful patterns
├── evaluation/             # Scoring, benchmarking, regression detection
├── events/                 # Pub/sub event bus (TaskStarted, ToolFailed, SkillLearned, etc.)
├── governance/             # Policy enforcement, risk thresholds, approval systems
├── intent/                 # Goal parsing, constraints, success criteria
├── knowledge/              # Knowledge graphs, entity linking, semantic relationships
├── memory/                 # Persistent (SurrealDB) + Ethereal (Dashmap)
├── observability/          # Traces, spans, decision logs, replay systems
├── orchestrator/           # Planning/control and agent-to-agent protocols
├── persona/                # Personality, long-term preferences, role specialization
├── planner/                # Planning strategies (swappable, including symbolic planners)
├── rag/                    # File embedding and ingestion pipeline
├── recovery/               # Error handling, retry, fallback planning
├── reflection/             # History analysis, outcome tracking, behavior evolution
├── registry/               # Unified registry for tools, skills, agents, capabilities
├── sandbox/                # Security isolation and execution containment
├── security/               # System-wide protection, secrets, encryption
├── skills/                 # Skills registry & loading
├── simulation/             # “What-if” simulation and risk estimation
├── verification/           # Trust layer – output validation, plan verification
├── workflow/               # Long-running workflows with checkpointing and resumability
├── world_model/            # Internal entity/system modeling and causal reasoning
└── utilities/              # Shared helpers (logging, etc.)
```

**Development Milestones**

1. **Phase 4.1** — Dynamic Skill Generation + Self-Extension Core enhancements
2. **Phase 4.2** — Autonomous MCP Tool Acquisition + Sandbox hardening
3. **Phase 4.3** — Secure Execution Sandbox + permission/verification system
4. **Phase 4.4** — Self-Improving Workflow System + reflection/evaluation loop
5. **Phase 4.5** — Full Recursive Capability Loop + world_model integration
6. **Phase 4.6** — Testing, observability, and long-term stability

**Success Criteria**

- Merix can autonomously generate and deploy a new working skill that solves a task it previously could not handle.
- The system demonstrates measurable improvement over time on the same task family.
- All generated code and tools remain fully auditable, versioned, and reversible by the user.
- The recursive loop runs safely with zero cloud dependency and full user oversight.

This phase completes the original vision: Merix is no longer just an assistant — it is a system that builds assistants as your assistant.

---

## WHY THIS EXISTS

Because AI systems are currently:

- stateless in practice
- fragmented in tooling
- dependent on external services
- incapable of evolving their own capabilities

Merix aims to change that by building:

> a local-first, self-extending AI execution runtime

---

## FINAL FORM VISION

Merix ultimately becomes:

- a runtime for AI agents
- a tool execution OS layer
- a skill evolution engine
- and a self-improving development environment

**Not an assistant.**

**A system that builds assistants as your assistant.**

---

**License:** Apache 2.0  
**Repository:** https://github.com/saviornt/Merix  

*Last updated: April 30, 2026 — Version 3.1*  
This README is now **fully self-contained** for easy copy-paste. It incorporates the fresh-start organization I would use if we deleted the repo and started from zero, enforces the `PLAN.md` test-driven discipline, and restores every detail from the original vision (including Phase 1.5/2/3) so you always know exactly what comes next — but only after PHASE 1 is complete and tested.

---

**Next Action (per PLAN.md discipline):**  
Replace your current `README.md` with the content above, then continue with the next unchecked item in your `PLAN.md`.  

When you’re ready for the next file (e.g. `crates/core/src/lib.rs` after MemoryLayer is confirmed working), just reply with the checkbox you just completed and the test output.  

The system remains 100% local-first, resumable, crash-safe, and aligned with the original vision. Ready for your next command.
