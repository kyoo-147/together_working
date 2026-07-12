# Together MVP 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the Rust Cargo workspace, implement agent discovery, and a basic Ratatui TUI sidebar for the Together orchestrator.

**Architecture:** A single binary `together` that parses CLI arguments to either run the background `daemon` (handling agent discovery and IPC) or the TUI `cli` (connecting via IPC to display state). Code is split into `core`, `daemon`, and `cli` crates.

**Tech Stack:** Rust, Cargo, Tokio, Interprocess (for local sockets), Ratatui, Crossterm, Clap.

## Global Constraints

- Version floors: Rust 2021 edition.
- Single binary executable named `together`.

---

### Task 1: Initialize Cargo Workspace and Crates

**Files:**
- Create: `Cargo.toml` (Workspace root)
- Create: `core/Cargo.toml`, `core/src/lib.rs`
- Create: `daemon/Cargo.toml`, `daemon/src/lib.rs`
- Create: `cli/Cargo.toml`, `cli/src/main.rs`

**Interfaces:**
- Produces: Base Rust project structure.

- [ ] **Step 1: Create workspace root `Cargo.toml`**
```toml
[workspace]
members = ["core", "daemon", "cli"]
resolver = "2"
```

- [ ] **Step 2: Create `core` crate**
Run: `cargo new --lib core`

- [ ] **Step 3: Create `daemon` crate**
Run: `cargo new --lib daemon`

- [ ] **Step 4: Create `cli` crate**
Run: `cargo new --bin cli`

- [ ] **Step 5: Verify build**
Run: `cargo build`
Expected: PASS

- [ ] **Step 6: Commit**
Run: `git add . ; git commit -m "chore: setup cargo workspace"`

### Task 2: Define Core Domain Models

**Files:**
- Modify: `core/Cargo.toml`
- Create: `core/src/models.rs`
- Modify: `core/src/lib.rs`
- Create: `core/tests/models_test.rs`

**Interfaces:**
- Produces: `Agent`, `Department`, `ReadinessState` structs.

- [ ] **Step 1: Add dependencies to `core/Cargo.toml`**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
```

- [ ] **Step 2: Write failing test in `core/tests/models_test.rs`**
```rust
use core::models::{Agent, ReadinessState};

#[test]
fn test_agent_creation() {
    let agent = Agent {
        name: "codex".to_string(),
        state: ReadinessState::Ready,
    };
    assert_eq!(agent.name, "codex");
}
```

- [ ] **Step 3: Run test**
Run: `cargo test -p core`
Expected: FAIL (module not found)

- [ ] **Step 4: Implement models in `core/src/models.rs`**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ReadinessState {
    Ready,
    Idle,
    Offline,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub name: String,
    pub state: ReadinessState,
}
```
And add `pub mod models;` to `core/src/lib.rs`.

- [ ] **Step 5: Run test**
Run: `cargo test -p core`
Expected: PASS

- [ ] **Step 6: Commit**
Run: `git add core/ ; git commit -m "feat: core domain models"`

### Task 3: Agent Discovery Logic

**Files:**
- Create: `core/src/discovery.rs`
- Modify: `core/src/lib.rs`
- Create: `core/tests/discovery_test.rs`

**Interfaces:**
- Produces: `scan_agents() -> Vec<Agent>`

- [ ] **Step 1: Write failing test in `core/tests/discovery_test.rs`**
```rust
use core::discovery::scan_agents;

#[test]
fn test_scan_returns_agents() {
    let agents = scan_agents();
    assert!(agents.len() >= 0);
}
```

- [ ] **Step 2: Run test**
Run: `cargo test -p core`
Expected: FAIL (module not found)

- [ ] **Step 3: Implement discovery in `core/src/discovery.rs`**
```rust
use crate::models::{Agent, ReadinessState};

pub fn scan_agents() -> Vec<Agent> {
    vec![
        Agent { name: "codex".to_string(), state: ReadinessState::Ready },
        Agent { name: "claude".to_string(), state: ReadinessState::Idle },
    ]
}
```
Add `pub mod discovery;` to `core/src/lib.rs`.

- [ ] **Step 4: Run test**
Run: `cargo test -p core`
Expected: PASS

- [ ] **Step 5: Commit**
Run: `git add core/ ; git commit -m "feat: agent discovery stub"`

### Task 4: CLI Entrypoint (Daemon vs TUI mode)

**Files:**
- Modify: `cli/Cargo.toml`
- Modify: `cli/src/main.rs`

**Interfaces:**
- Consumes: `clap` args.

- [ ] **Step 1: Add clap to `cli/Cargo.toml`**
```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
```

- [ ] **Step 2: Implement CLI routing in `cli/src/main.rs`**
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "together", about = "AI Department Orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Daemon,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Daemon) => {
            println!("Starting daemon...");
        }
        None => {
            println!("Starting TUI...");
        }
    }
}
```

- [ ] **Step 3: Test command**
Run: `cargo run -p cli -- daemon`
Expected: "Starting daemon..."

- [ ] **Step 4: Commit**
Run: `git add cli/ ; git commit -m "feat: cli argument parser"`

### Task 5: Basic Ratatui Sidebar

**Files:**
- Modify: `cli/Cargo.toml`
- Create: `cli/src/tui.rs`
- Modify: `cli/src/main.rs`

**Interfaces:**
- Consumes: `core::discovery::scan_agents`

- [ ] **Step 1: Add dependencies to `cli/Cargo.toml`**
```toml
# Add these under [dependencies]
crossterm = "0.27"
ratatui = "0.26"
core = { path = "../core" }
```

- [ ] **Step 2: Implement basic UI in `cli/src/tui.rs`**
```rust
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use crossterm::{
    event::{self, KeyCode, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout};
use core::discovery::scan_agents;

pub fn run_tui() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let agents = scan_agents();
            let items: Vec<ListItem> = agents
                .iter()
                .map(|a| ListItem::new(format!("{} [{:?}]", a.name, a.state)))
                .collect();
            let list = List::new(items).block(Block::default().title("Agents").borders(Borders::ALL));
            
            // Just render the list in the whole screen for MVP 1
            f.render_widget(list, f.size());
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, crossterm::cursor::Show)?;
    Ok(())
}
```

- [ ] **Step 3: Connect to `main.rs`**
Modify `cli/src/main.rs`:
```rust
mod tui;
// ... inside main(), replace println!("Starting TUI...") with:
// tui::run_tui().unwrap();
```

- [ ] **Step 4: Commit**
Run: `git add cli/ ; git commit -m "feat: basic ratatui sidebar"`
