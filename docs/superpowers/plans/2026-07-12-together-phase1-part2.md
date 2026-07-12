# Together Phase 1 (Part 2) Implementation Plan

## Context
Following the completion of the basic IPC, EventStore, and TaskContract models, this plan wires them together and implements the Routing logic, completing Phase 1. 

## Design Decisions (from /grill-me)
- **Agent State:** Daemon will have an in-memory `AgentRegistry` to track agent states (`Ready`, `Busy`).
- **Routing/Queuing:** If an explicitly requested agent or an agent from a requested department is not `Ready`, the task is placed into a `TaskQueued` state (with its targeting requirements). When an agent becomes `Ready`, the queue is re-evaluated.
- **IPC Protocol:** 
  - `SUB` command: Client receives a continuous JSONL stream of all events (past and future).
  - `CREATE {yaml}` command: Daemon parses the contract, persists `TaskCreated`, routes it, and returns an immediate ACK containing the `task_id`.
- **CLI/TUI Experience:** `together run <task.yaml>` will submit and exit immediately. The TUI (`together`) will use the `SUB` command to display real-time progress.

## Global Constraints
- Version floors: Rust 2021 edition.
- Single binary executable named `together`.
- Fully tested (TDD). 
- Use cross-platform local sockets (`ToNsName`).

---

### Task 5: Core Events and Routing Models
Update the domain models to support the new routing states and implement the `AgentRegistry`.

**Files:**
- Modify: `core/src/events.rs`, `core/src/contracts.rs`
- Create: `daemon/src/registry.rs`
- Create: `daemon/src/router.rs`

- [ ] **Step 1: Update Events and Target Models**
Update `Event` in `core/src/events.rs`:
```rust
use crate::contracts::TaskContract;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoutingTarget {
    Agent(String),
    Department(String),
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Ready,
    Busy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    TaskCreated { task_id: String, contract: TaskContract },
    TaskQueued { task_id: String, target: RoutingTarget },
    TaskRouted { task_id: String, agent_name: String },
    AgentStatusChanged { agent_name: String, status: AgentStatus },
}
```
Update tests in `core` to match.

- [ ] **Step 2: Implement AgentRegistry (`daemon/src/registry.rs`)**
Create a thread-safe registry (`Arc<Mutex<...>>` or `RwLock`) that maps `agent_name -> AgentStatus` and tracks which department an agent belongs to. It should have a method to get an available agent for a given `RoutingTarget`.

- [ ] **Step 3: Implement Router (`daemon/src/router.rs`)**
Implement logic that evaluates a `TaskContract`. If `registry.get_available_agent(target)` returns `Some(agent_name)`, return `Event::TaskRouted`. If `None`, return `Event::TaskQueued`.

- [ ] **Step 4: Commit**
Commit changes as `feat(routing): implement agent registry and routing models`

---

### Task 6: IPC Protocol and Daemon Wiring
Implement the `SUB` and `CREATE` IPC commands.

**Files:**
- Create: `core/src/ipc.rs`
- Modify: `daemon/src/server.rs`

- [ ] **Step 1: Define IPC Commands**
In `core/src/ipc.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Sub,
    CreateTask { yaml: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Ack { task_id: String },
    Error { message: String },
}
```
Export this module in `core/src/lib.rs`.

- [ ] **Step 2: Wire Daemon Server**
Update `daemon/src/server.rs` to parse incoming commands via `serde_json`.
- If `Command::CreateTask { yaml }`: parse it using `TaskContract::from_yaml`. If invalid, return `Response::Error`. If valid, generate a UUID for `task_id`, create `Event::TaskCreated`, pass to Router for `TaskRouted/TaskQueued`. Persist events to `EventStore`. Write `Response::Ack` back to the socket and close it.
- If `Command::Sub`: Read all events from `EventStore` and stream them as JSONL. Then subscribe to an in-memory broadcast channel (e.g., using `std::sync::mpsc` or `tokio::sync::broadcast` if tokio is added, otherwise standard channels) to push new events to this socket indefinitely.

- [ ] **Step 3: Commit**
Commit changes as `feat(daemon): implement SUB and CREATE IPC protocol`

---

### Task 7: CLI Client and TUI Integration
Connect the CLI and TUI to the Daemon.

**Files:**
- Modify: `cli/src/client.rs`
- Modify: `cli/src/main.rs`
- Modify: `cli/src/tui.rs` (or equivalent UI file)

- [ ] **Step 1: Implement `together run <file>`**
In `cli/src/main.rs`, add a `run` subcommand that reads a YAML file and calls `client::send_create_task(yaml)`. Print the resulting Task ID or error.

- [ ] **Step 2: TUI Event Subscription**
In `cli/src/client.rs`, add a `subscribe()` function that connects to the socket, sends `Command::Sub`, and returns an iterator or channel receiver of `Event`.

- [ ] **Step 3: TUI Hookup**
In the TUI loop, ingest the event stream. Update local state models (e.g., Lists of active tasks, queued tasks, and agent statuses). Render these models to the Ratatui dashboard.

- [ ] **Step 4: Commit**
Commit changes as `feat(cli): wire TUI and CLI to real event stream`
