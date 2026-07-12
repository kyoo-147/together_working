# Together MVP 1: Daemon & Agent Discovery Design

## 1. Overview
Together MVP 1 establishes the foundational local-first architecture for the AI Department Orchestrator. It implements a decoupled client-server architecture with a background runtime (daemon) and a Terminal UI (TUI) client, focusing primarily on agent discovery and the basic communication protocol.

## 2. Architecture & Binary Structure
- **Single Binary (`together`)**: The application will be distributed as a single Rust executable.
- **Commands**:
  - `together daemon`: Starts the background server (the runtime).
  - `together`: Starts the TUI client which connects to the running daemon.

## 3. Cargo Workspace Organization
The Rust project will be organized as a Cargo workspace to ensure separation of concerns and maintainability:
- **`core`**: Contains domain models (`Task`, `Agent`, `Department`, `Contract`) and shared logic.
- **`daemon`**: Handles the IPC server, background tasks, state management, and agent orchestration.
- **`cli`**: Contains the command parser (`clap`) and the TUI interface (`ratatui` + `crossterm`).

## 4. IPC Communication
- **Protocol**: JSON payloads over Local Sockets.
- **Transport**: Unix Domain Sockets on Linux/macOS, Named Pipes on Windows.
- **Library**: `interprocess` crate for robust cross-platform local socket support.
- **Rationale**: Provides native, high-speed, secure communication suitable for a terminal-first workflow.

## 5. Agent Discovery & Readiness
- **Hybrid Approach**:
  1. **Dynamic PATH Scanning**: Automatically scans the system `PATH` for known CLI agents (e.g., `claude`, `codex`, `amp`, `gemini`).
  2. **Static Configuration**: Reads from a user configuration file (`~/.together/config.toml` or Windows equivalent `%USERPROFILE%\.together\config.toml`) to allow users to register custom agents, override paths, or disable specific detected agents.
- **Readiness Check**: Validates if the detected executables exist and can be invoked.

## 6. TUI Layout (MVP 1 Scope)
- **Framework**: `ratatui` and `crossterm`.
- **Interface**: A basic screen containing a Sidebar.
- **Sidebar**: Displays the list of discovered agents, their respective departments, and their current static readiness state (e.g., `ready`, `offline`).
- **Exclusions**: MVP 1 will *not* include the actual PTY execution engine or task routing logic. These are reserved for subsequent MVPs.

## 7. Next Steps
- Initialize the Rust Cargo workspace.
- Implement the `interprocess` IPC layer.
- Implement the Agent Discovery scanner.
- Build the initial Ratatui Sidebar.
