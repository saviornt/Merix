# Merix

**Version 1.0**

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
