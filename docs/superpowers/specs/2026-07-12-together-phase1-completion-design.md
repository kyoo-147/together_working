# Together Phase 1 Completion Design

## 1. Overview
This design covers the completion of Phase 1 of the Together orchestrator (Core Daemon + CLI). Building upon MVP 1 (workspace, basic discovery, basic TUI), this phase implements Local Socket IPC, SQLite Event Sourcing, Task Contract Parsing, and Task Routing.

## 2. Local Socket API (Daemon ↔ Client)
- **Protocol Model:** Bidirectional Event Stream.
- **Transport:** `interprocess` Local Sockets (Unix sockets on Linux/macOS, Named Pipes on Windows).
- **Format:** JSON lines over the socket.
- **Flow:**
  - Client connects to the Daemon socket.
  - Client sends command messages (e.g., `CreateTask { contract_path: "..." }`).
  - Daemon continuously emits event messages (e.g., `AgentDiscovered`, `TaskCreated`, `TaskRouted`).
  - TUI subscribes to this stream to update its UI state in real-time.

## 3. Persistence (SQLite + Event Sourcing)
- **Event-Driven:** All state mutations are persisted as an append-only event log.
- **SQLite Schema:**
  - Table: `event_log`
    - `id` (INTEGER PRIMARY KEY)
    - `timestamp` (DATETIME)
    - `event_type` (TEXT) - e.g., "TaskCreated", "TaskRouted"
    - `payload` (JSON) - the full event payload.
- **State Hydration:** On daemon startup, the current state of tasks and agents is rehydrated by replaying the `event_log`.

## 4. Task Contract Input & Validation
- **Input Strategy:** Daemon-side parsing.
- **Flow:**
  - The user runs a command via CLI/TUI providing a path to a YAML file (e.g., `together create task-contract.yaml`).
  - The CLI sends a `CreateTask { contract_path }` command via Local Socket.
  - The Daemon reads the file from the filesystem, parses the YAML into a `TaskContract` struct, validates its schema (scope, deliverables), and persists a `TaskCreated` event if valid.

## 5. Routing Strategy (Explicit + Fallback)
- **Mechanism:**
  - When a `TaskCreated` event is processed, the Routing engine evaluates the task.
  - **Explicit:** If the `TaskContract` specifies a specific agent (e.g., `agent: "codex"`), the daemon verifies the agent is `Ready`.
  - **Fallback:** If no agent is explicitly set, the daemon looks at the `department` field in the contract and randomly selects a `Ready` agent from that department.
- **Result:** The daemon emits a `TaskRouted { task_id, agent_name }` event. (Execution is deferred to Phase 2).

## 6. Project Boundaries
- The `core` crate will house the Event definitions, Task Contract schema, and state hydration logic.
- The `daemon` crate will house the SQLite integration (using `rusqlite`), the Local Socket server, and the Routing logic.
- The `cli` crate will house the Local Socket client and update the TUI to reflect real-time events.
