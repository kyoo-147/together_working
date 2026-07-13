# Together Phase 1 Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete Phase 1 by implementing SQLite event sourcing, YAML task contracts, routing, and a Local Socket IPC layer between the CLI and Daemon.

**Architecture:** Bidirectional Event Stream over Local Sockets (`interprocess`). The Daemon manages an SQLite `event_log` for persistence and reconstructs state. The TUI/CLI sends commands (like `CreateTask`) to the Daemon and subscribes to state change events (`TaskCreated`, `TaskRouted`).

**Tech Stack:** Rust 2021, `rusqlite`, `serde`, `serde_json`, `serde_yaml`, `interprocess`, `tokio` (if async, but currently synchronous or standard threads based on initial MVP. We'll use standard `std::thread` and blocking `interprocess` as per simple TUI design, or `tokio` if needed. Let's stick to standard library + blocking I/O for simplicity unless `tokio` is already in the workspace). *Note: We will use standard blocking I/O for sockets to keep it simple, as `interprocess` supports blocking.*

## Global Constraints

- Version floors: Rust 2021 edition.
- Single binary executable named `together`. (The CLI and Daemon are run via `together` and `together daemon`).
- Code must be fully tested (TDD).

---

### Task 1: Define Event Domain Models

**Files:**
- Create: `core/src/events.rs`
- Modify: `core/src/lib.rs`
- Modify: `core/Cargo.toml`
- Test: `core/src/events.rs` (inline tests)

**Interfaces:**
- Produces: `Event` enum containing `TaskCreated`, `TaskRouted`, etc.

- [ ] **Step 1: Add dependencies to `core/Cargo.toml`**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

- [ ] **Step 2: Write failing test in `core/src/events.rs`**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_event_serialization() {
        let event = Event::TaskCreated { task_id: "t-1".to_string(), contract_path: "path".to_string() };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("TaskCreated"));
    }
}
```

- [ ] **Step 3: Run test to verify it fails**
Run: `cargo test -p core`
Expected: FAIL (Event not found)

- [ ] **Step 4: Implement `Event` in `core/src/events.rs`**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    TaskCreated { task_id: String, contract_path: String },
    TaskRouted { task_id: String, agent_name: String },
}
```
Update `core/src/lib.rs`:
```rust
pub mod discovery;
pub mod models;
pub mod events;
```

- [ ] **Step 5: Run test to verify it passes**
Run: `cargo test -p core`
Expected: PASS

- [ ] **Step 6: Commit**
```bash
git add core/Cargo.toml core/src/lib.rs core/src/events.rs
git commit -m "feat(core): add Event domain models for Phase 1"
```

### Task 2: Define Task Contract Models

**Files:**
- Create: `core/src/contracts.rs`
- Modify: `core/src/lib.rs`
- Modify: `core/Cargo.toml`
- Test: `core/src/contracts.rs`

**Interfaces:**
- Produces: `TaskContract` struct and YAML parsing logic.

- [ ] **Step 1: Add `serde_yaml` to `core/Cargo.toml`**
```toml
serde_yaml = "0.9"
```

- [ ] **Step 2: Write failing test in `core/src/contracts.rs`**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_contract() {
        let yaml = "
task_id: test-1
department: engineering
agent: codex
";
        let contract = TaskContract::from_yaml(yaml).unwrap();
        assert_eq!(contract.task_id, "test-1");
        assert_eq!(contract.department, Some("engineering".to_string()));
        assert_eq!(contract.agent, Some("codex".to_string()));
    }
}
```

- [ ] **Step 3: Run test to verify it fails**
Run: `cargo test -p core`
Expected: FAIL (TaskContract not found)

- [ ] **Step 4: Implement `TaskContract`**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContract {
    pub task_id: String,
    pub department: Option<String>,
    pub agent: Option<String>,
}

impl TaskContract {
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}
```
Update `core/src/lib.rs`:
```rust
pub mod contracts;
```

- [ ] **Step 5: Run test to verify it passes**
Run: `cargo test -p core`
Expected: PASS

- [ ] **Step 6: Commit**
```bash
git add core/Cargo.toml core/src/lib.rs core/src/contracts.rs
git commit -m "feat(core): add TaskContract model and parsing"
```

### Task 3: Implement SQLite Event Store in Daemon

**Files:**
- Create: `daemon/src/store.rs`
- Modify: `daemon/src/lib.rs`
- Modify: `daemon/Cargo.toml`
- Test: `daemon/src/store.rs`

**Interfaces:**
- Consumes: `core::events::Event`
- Produces: `EventStore` struct with `append(event)` and `get_all()` methods.

- [ ] **Step 1: Add `rusqlite` to `daemon/Cargo.toml`**
```toml
[dependencies]
rusqlite = { version = "0.31.0", features = ["bundled"] }
serde_json = "1.0"
core = { path = "../core" }
```

- [ ] **Step 2: Write failing test in `daemon/src/store.rs`**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use core::events::Event;

    #[test]
    fn test_event_store() {
        let store = EventStore::in_memory().unwrap();
        let event = Event::TaskRouted { task_id: "1".to_string(), agent_name: "test".to_string() };
        store.append(&event).unwrap();
        
        let events = store.get_all().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }
}
```

- [ ] **Step 3: Run test to verify it fails**
Run: `cargo test -p daemon`
Expected: FAIL

- [ ] **Step 4: Implement `EventStore`**
```rust
use core::events::Event;
use rusqlite::{params, Connection, Result as SqlResult};

pub struct EventStore {
    conn: Connection,
}

impl EventStore {
    pub fn in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init_db(&conn)?;
        Ok(Self { conn })
    }

    pub fn new(path: &str) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        Self::init_db(&conn)?;
        Ok(Self { conn })
    }

    fn init_db(conn: &Connection) -> SqlResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS event_log (
                id INTEGER PRIMARY KEY,
                event_type TEXT NOT NULL,
                payload TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn append(&self, event: &Event) -> SqlResult<()> {
        let payload = serde_json::to_string(event).unwrap();
        let event_type = match event {
            Event::TaskCreated { .. } => "TaskCreated",
            Event::TaskRouted { .. } => "TaskRouted",
        };
        self.conn.execute(
            "INSERT INTO event_log (event_type, payload) VALUES (?1, ?2)",
            params![event_type, payload],
        )?;
        Ok(())
    }

    pub fn get_all(&self) -> SqlResult<Vec<Event>> {
        let mut stmt = self.conn.prepare("SELECT payload FROM event_log ORDER BY id ASC")?;
        let event_iter = stmt.query_map([], |row| {
            let payload: String = row.get(0)?;
            let event: Event = serde_json::from_str(&payload).unwrap();
            Ok(event)
        })?;

        let mut events = Vec::new();
        for event in event_iter {
            events.push(event?);
        }
        Ok(events)
    }
}
```
Update `daemon/src/lib.rs` (create if missing):
```rust
pub mod store;
```

- [ ] **Step 5: Run test to verify it passes**
Run: `cargo test -p daemon`
Expected: PASS

- [ ] **Step 6: Commit**
```bash
git add daemon/Cargo.toml daemon/src/lib.rs daemon/src/store.rs
git commit -m "feat(daemon): implement SQLite EventStore"
```

### Task 4: Local Socket IPC API (Daemon and CLI)

**Files:**
- Modify: `cli/Cargo.toml`, `daemon/Cargo.toml`
- Create: `daemon/src/server.rs`
- Create: `cli/src/client.rs`

**Interfaces:**
- Produces: IPC communication via `interprocess`.

- [ ] **Step 1: Add `interprocess` dependency**
Add to `daemon/Cargo.toml` and `cli/Cargo.toml`:
```toml
interprocess = "2.2.1"
```

- [ ] **Step 2: Implement Daemon IPC Server (`daemon/src/server.rs`)**
```rust
use interprocess::local_socket::{prelude::*, GenericFilePath, ListenerOptions, Stream};
use std::io::{BufRead, BufReader, Write};
use core::events::Event;

pub fn start_server() {
    let name = "/tmp/together.sock".to_fs_name::<GenericFilePath>().unwrap();
    // Ignore error if file doesn't exist
    let _ = std::fs::remove_file("/tmp/together.sock");
    
    let opts = ListenerOptions::new().name(name);
    let listener = opts.create_sync().unwrap();
    
    std::thread::spawn(move || {
        for mut conn in listener.incoming().filter_map(Result::ok) {
            let mut conn_clone = conn.try_clone().unwrap();
            std::thread::spawn(move || {
                let mut reader = BufReader::new(&mut conn_clone);
                let mut buffer = String::new();
                while let Ok(bytes) = reader.read_line(&mut buffer) {
                    if bytes == 0 { break; }
                    println!("Daemon received: {}", buffer);
                    // Echo back as TaskCreated event for now
                    let event = Event::TaskCreated { task_id: "1".to_string(), contract_path: buffer.trim().to_string() };
                    let response = serde_json::to_string(&event).unwrap() + "\n";
                    let _ = conn.write_all(response.as_bytes());
                    buffer.clear();
                }
            });
        }
    });
}
```
Update `daemon/src/lib.rs`:
```rust
pub mod store;
pub mod server;
```

- [ ] **Step 3: Implement CLI IPC Client (`cli/src/client.rs`)**
```rust
use interprocess::local_socket::{prelude::*, GenericFilePath, Stream};
use std::io::{BufRead, BufReader, Write};

pub fn send_command(cmd: &str) -> Result<String, std::io::Error> {
    let name = "/tmp/together.sock".to_fs_name::<GenericFilePath>().unwrap();
    let mut conn = Stream::connect(name)?;
    
    let msg = format!("{}\n", cmd);
    conn.write_all(msg.as_bytes())?;
    
    let mut reader = BufReader::new(&mut conn);
    let mut response = String::new();
    reader.read_line(&mut response)?;
    Ok(response)
}
```
Update `cli/src/main.rs`:
```rust
mod client;
// rest of the file...
```

- [ ] **Step 4: Commit**
```bash
git add daemon/Cargo.toml cli/Cargo.toml daemon/src/server.rs cli/src/client.rs daemon/src/lib.rs cli/src/main.rs
git commit -m "feat(ipc): implement Local Socket Server and Client"
```
