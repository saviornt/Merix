# Merix

**Version 2.0**

**The rogue messenger that delivers forbidden intelligence.**

One clean desktop binary.  
Zero cloud. Zero daemons.  
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

## PHASE 1 — PRIMARY MVP (Foundation Runtime)

This phase establishes the core execution environment.

### Core Objective

A stable local AI runtime that can:

- execute tasks
- persist state
- resume execution
- and safely invoke tools via MCP
- and evolve behavior via Skills

This is the **minimal self-contained intelligence loop**.

---

### 1. Local Model Runtime

- llama.cpp-based inference engine
- user-provided model loading (no bundling requirement)
- CPU + GPU execution support
- deterministic session initialization

---

### 2. Session + Task Execution System (Critical)

- structured execution model:
  - task → steps → outputs → checkpoints
- persistent state storage
- crash-safe resume system
- partial execution recovery

---

### 3. Memory Layer

- SurrealDB-backed persistence memory support (session-based and lifetime long-term memory)
- Dashmap-backed ethereal memory support (short-term memory)
- retrieval-based context reconstruction
- structured project memory support

---

### 4. MCP Tooling System (Core Interface Layer)

The standardized bridge between reasoning and action.

- MCP-compatible tool execution layer
- tool discovery + registry system
- permissioned execution sandbox
- structured input/output contracts
- safe tool invocation boundaries

This enables the system to interact with external capabilities.

---

### 5. Skills System (Behavior Layer)

Skills are modular capability definitions.

- installable and versioned skill units
- runtime skill loading/unloading
- skill chaining and composition
- skills can:
  - call MCP tools
  - access memory
  - extend workflows
  - define reusable behaviors

Skills are the foundation of extensibility.

---

### 6. Self-Extension Core (Machine Bootstrap Layer)

The MVP must support:

- generating new skills from tasks
- saving behaviors as reusable modules
- evolving workflows from repeated patterns
- building higher-level abstractions from execution history

This is the first step toward system recursion.

---

### 7. Minimal Control Interface

- lightweight UI or CLI
- session/task browser
- MCP tool execution log
- skill registry viewer
- resume last task capability

UI is secondary to execution reliability.

---

### 8. System and Application Logging

- lighweight logging system
- generates .json logs for review
- uses datetime (UTC format) severity levels (0 -> 7), tags, and descriptions
- custom severity level for application specific logging (8+) including tag and description/output
- logs can be written to *.logs file or separate database (logs)
- datetime can be shown in other formats (Local, EST, PST, etc..)

Logs are important for both developers, users and agents.

---

### 9. Vector Embeddings & Semantic Memory

- Local embedding generation using the same llama.cpp engine as the main inference runtime
- Support for user-provided GGUF embedding models (nomic-embed-text, bge-small-en-v1.5, snowflake-arctic-embed, etc.)
- Automatic vector embedding of Sessions, Tasks, Steps, Checkpoints, and Project Memory items on save
- Native vector fields and indexes in SurrealDB (cosine / euclidean / dot-product)
- Semantic retrieval API in MemoryLayer (`reconstruct_context` becomes hybrid vector + structured)
- Top-k similarity search with configurable thresholds
- Full integration with Skills System, MCP Tooling, and Self-Extension Core (skills/tools can query by semantic relevance)
- Zero-cloud, fully offline, deterministic, and crash-safe
- Backward-compatible with existing non-vector data

This feature completes the foundation for meaningful long-term memory and enables the self-extension loop to reason over semantic patterns rather than raw text dumps.

---

### 10. Local Model Runtime (LLM Inference Engine)

- llama.cpp-based inference engine via production-ready Rust bindings
- User-provided GGUF model loading (no bundling — user places models in `models/` or any path)
- Full CPU + GPU support (CUDA, Metal, ROCm) with automatic hardware detection
- Deterministic session initialization and reproducible sampling
- Streaming token generation with configurable context length, temperature, top-p, etc.
- Direct integration with TaskExecutor: each step can invoke the model for reasoning, planning, or output generation
- Shared inference context across tasks/steps for efficient memory use
- Zero-cloud, fully offline, crash-safe, and resumable inference sessions
- Performance-first design matching or exceeding llama.cpp native speed

This is the final foundational piece of PHASE 1. With this feature the entire intelligence loop (Task → Memory → MCP Tools → Skills → Self-Extension) becomes truly intelligent and no longer relies on stubbed execution.

---

## PHASE 1.5 — Visual Studio Code Plugin

**The developer’s bridge between raw CLI power and polished desktop UX.**

This phase brings Merix directly into the most important tool a developer uses every day — Visual Studio Code — creating a seamless, always-available agent that accelerates development of Merix itself and every subsequent phase.

### Phase 1.5 Objective

> Turn VS Code into the primary interface for Merix, allowing developers to chat, invoke tools/skills, run tasks, and collaborate with the full local intelligence loop without ever leaving their editor.

---

### 1. VS Code Extension Core

- Official VS Code extension (TypeScript + Webview or native LSP + chat panel)
- Sidebar chat interface with full Merix conversational capabilities
- Local communication with the running Merix runtime (via Unix socket / named pipe / local HTTP)

### 2. Agent Capabilities Inside VS Code

- Chat with Merix directly in the editor
- Execute any registered Skill or MCP Tool from chat
- Context-aware operations: analyze current file, workspace, git diff, open tabs, etc.
- Agent mode for tasks like “refactor this function”, “add tests”, “explain this error”, “generate documentation”

### 3. Deep Integration with Merix Runtime

- Direct access to MemoryLayer (semantic + structured recall)
- Full ToolRegistry and SkillRegistry exposure
- Task creation, resume, and self-extension commands from within VS Code
- Real-time feedback from running tasks and skill execution

### 4. Developer Productivity Features

- Command palette integration (`Merix: Run Task…`, `Merix: Self-Extend`, etc.)
- Inline code suggestions and edits powered by the local LLM
- Session browser and history viewer inside VS Code
- One-click “Ask Merix about this code” on any selection

### 5. Self-Bootstrapping Value

This phase is intentionally placed in the middle of PHASE 1 because the VS Code plugin will be used to **develop and test Merix itself**. It becomes the primary development interface for all future phases, creating a powerful feedback loop where Merix helps build Merix.

---

**Status:** Planned as the final step of the Foundation Runtime before moving to full desktop UX (PHASE 2).

This phase transforms Merix from a command-line tool into a living development companion that lives inside the editor — the most natural place for a local AI agent to exist.

---

## PHASE 2 — USER EXPERIENCE (Product Layer)

This phase transforms the runtime into a usable product.

### Phase 2 Objective

Make the system feel like:
> a coherent AI workstation, not a developer experiment

---

### 1. Desktop Application Layer

- Tauri v2 native shell
- React + TypeScript frontend
- cross-platform support
- fast startup and low idle overhead

---

### 2. Workspace System

- project-based environments
- file explorer integration
- multi-file editing support
- git awareness and diff tracking

---

### 3. Agent UX Layer

- structured planning mode
- multi-step execution visualization
- tool invocation transparency
- session timelines and history

---

### 4. Vision + Input Extensions

- drag-and-drop images
- screenshot understanding
- log file ingestion
- multimodal context integration

---

### 5. Optional Voice Interface

- local TTS/STT integration
- conversational mode
- hands-free interaction layer

---

### 6. UX Polish Layer

- dark-first interface
- preset modes (coding, planning, analysis)
- smooth session navigation
- improved ergonomics and responsiveness

---

## PHASE 3 — EVOLVING SYSTEMS (Autonomous Capability Layer)

This phase introduces self-adaptive behavior and system evolution.

### Phase 3 Objective

Transform Merix from a tool into:

> a system that grows its own capabilities during use

---

### Core Concept

Agents no longer just use tools.

They:

- discover missing capabilities
- generate new skills
- acquire or create tools
- evolve workflows dynamically

---

### 1. Dynamic Skill Generation

- agents generate new skills during task execution
- skills are derived from repeated patterns
- skill abstraction becomes automatic
- reusable behaviors are continuously extracted

---

### 2. Autonomous Tool Acquisition (MCP Expansion Layer)

- runtime discovery of MCP tools
- automatic installation of required tools (sandboxed)
- capability negotiation layer (what is allowed vs required)
- tool lifecycle management

---

### 3. Secure Execution Sandbox (Containment Layer)

- isolated execution environment for all tools
- prompt injection / malicious payload detection layer
- capability-based permissions system
- signed skill and tool verification system

Prevents:

- unsafe tool execution
- malicious instruction injection
- uncontrolled system modification

---

### 4. Self-Improving Workflow System

- workflows evolve based on usage patterns
- agents optimize their own task decomposition
- redundant steps are eliminated automatically
- performance improves over time through reinforcement of effective paths

---

### 5. Recursive Capability Loop (Core Innovation)

System loop:

1. receive task  
2. identify missing capabilities  
3. generate or acquire skills/tools  
4. execute task  
5. evaluate gaps  
6. refine system  
7. persist improvements  

This creates:
> a continuously evolving execution environment

---

### ARCHITECTURE PRINCIPLES

- Everything is resumable
- Nothing runs unless required
- Tools are explicit and sandboxed
- Skills define behavior, not prompts
- The system must be capable of extending itself
- State is first-class, not incidental

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

Not an assistant.

A system that builds assistants as your assistant.
