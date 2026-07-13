# Together Phase 2: PTY Execution Engine Implementation Plan

## Context
Phase 1 implemented the Core Daemon, Local Socket IPC, and Routing logic. Phase 2 introduces the **Execution Engine** using `portable-pty`. The Daemon will take routed tasks and spawn actual agent processes (Adapters) inside a pseudo-terminal (PTY), capturing their output stream in real-time, persisting it, and broadcasting it over IPC.

## Design Decisions (from /grill-me)
- **AgentAdapters**: We will define an `AgentAdapter` trait. Implementations like `CodexAdapter` or `FakeAdapter` will wrap the actual execution commands.
- **PTY Management**: We will use the `portable-pty` crate to spawn the adapter processes. This ensures ANSI colors and full terminal behaviors are captured properly.
- **Scrollback / Persistence**: PTY output will be captured into SQLite (e.g. a `pty_logs` table) for scrollback.
- **CLI Attach**: The CLI will add a new subcommand `together attach <task_id>` which uses the `Sub` IPC connection to stream the real-time `PtyOutput` events for that task directly to the terminal.

## Global Constraints
- Version floors: Rust 2021 edition.
- Single binary executable named `together`.
- Code must be fully tested (TDD). 

---

### Task 1: Agent Adapter Interface
Create the execution interface that translates a TaskContract into a spawnable Command.

**Files:**
- Create: `daemon/src/adapters/mod.rs`
- Create: `daemon/src/adapters/fake.rs`

- [ ] **Step 1: Define `AgentAdapter` Trait**
In `daemon/src/adapters/mod.rs`, define:
```rust
use core::contracts::TaskContract;
use std::process::Command;

pub trait AgentAdapter {
    fn build_command(&self, contract: &TaskContract) -> Command;
}
```

- [ ] **Step 2: Implement `FakeAdapter`**
In `daemon/src/adapters/fake.rs`, implement `AgentAdapter` for a `FakeAdapter` that simply returns a `Command` executing a basic shell echo or sleep sequence to simulate work. (e.g. on Windows `cmd /c echo "Doing work..."`, on Linux `sh -c "echo 'Doing work...'"`).

- [ ] **Step 3: Commit**
Commit changes as `feat(daemon): define AgentAdapter interface and FakeAdapter`

---

### Task 2: PTY Manager
Integrate `portable-pty` to spawn and read from processes.

**Files:**
- Modify: `daemon/Cargo.toml` (add `portable-pty` dependency)
- Create: `daemon/src/pty.rs`

- [ ] **Step 1: PTY Spawner**
Create `PtyManager::spawn(command: Command) -> Result<Box<dyn Read + Send>>`. This function should use `portable_pty::native_pty_system` to allocate a PTY, spawn the child process into it, and return the reader for the master side of the PTY.

- [ ] **Step 2: Output Reading & Tests**
Implement a loop that reads chunks from the PTY reader into a buffer. Add unit tests that spawn the `FakeAdapter` via `PtyManager` and assert the output string can be successfully captured.

- [ ] **Step 3: Commit**
Commit changes as `feat(daemon): implement PtyManager using portable-pty`

---

### Task 3: Execution Supervisor and Event Flow
Wire the Router to the PTY Manager and broadcast the output.

**Files:**
- Modify: `core/src/events.rs`
- Modify: `daemon/src/supervisor.rs` (Create)
- Modify: `daemon/src/server.rs`

- [ ] **Step 1: Update Events**
Add to `core::events::Event`:
```rust
    PtyOutput { task_id: String, chunk: String },
    TaskCompleted { task_id: String, success: bool },
```

- [ ] **Step 2: Implement Supervisor**
Create `daemon/src/supervisor.rs`. The supervisor listens for `Event::TaskRouted`. When caught, it:
1. Looks up the appropriate `AgentAdapter` (use `FakeAdapter` for now).
2. Calls `PtyManager::spawn`.
3. In a separate thread, continuously reads chunks from the PTY.
4. For each chunk, it emits `Event::PtyOutput { task_id, chunk }` back into the Daemon's event broadcast channel.
5. When the process exits, it emits `Event::TaskCompleted`.

- [ ] **Step 3: Wire into Server**
Update `daemon/src/server.rs`. Whenever an event is processed or routed, feed it to the Supervisor so it can trigger executions. The Supervisor's emitted `PtyOutput` events must flow into the `EventStore` and out to the active `Sub` IPC clients.

- [ ] **Step 4: Commit**
Commit changes as `feat(daemon): implement execution supervisor for PTY output stream`

---

### Task 4: CLI Attach Command
Enable the user to watch the PTY output from their terminal.

**Files:**
- Modify: `cli/src/main.rs`

- [ ] **Step 1: Parse `attach <task_id>`**
Update CLI arguments to support `together attach <task_id>`.

- [ ] **Step 2: Stream Output**
When `attach` is called, the CLI connects using `client::subscribe()`. It filters the incoming stream:
- If `Event::PtyOutput { task_id, chunk }` matches the requested ID, print the chunk to `stdout` directly (use `print!("{chunk}"); io::stdout().flush()`).
- If `Event::TaskCompleted { task_id, .. }` matches, print a completion message and exit the CLI process.

- [ ] **Step 3: Commit**
Commit changes as `feat(cli): add attach command to view real-time PTY stream`
