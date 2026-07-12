# Together Phase 3: TUI MVP Implementation Plan

## Context
Phase 1 (Daemon, IPC, Routing) and Phase 2 (Execution Engine, PTY) are complete. Phase 3 focuses on delivering a robust, responsive Terminal User Interface (TUI) for Together using `ratatui`. 

## Design Decisions (from /grill-me)
- **Layout:** A responsive 3-column layout. 
  - Left column: Agents / Departments overview.
  - Middle column: Tasks & status (Queued, Routed, Completed).
  - Right column: PTY log view of the currently selected task.
- **Architecture:** Maintain a centralized `TuiState` that is mutated by the background IPC event stream. The rendering logic will be broken down into individual components inside `cli/src/ui/` for maintainability.

## Global Constraints
- Version floors: Rust 2021 edition.
- Single binary executable named `together`.
- Code must be tested where feasible (TUI view logic is notoriously hard to test, so focus tests on state transformation).

---

### Task 1: TUI State & Component Architecture
Set up the `ui` module, state container, and separate the basic drawing modules.

**Files:**
- Create: `cli/src/ui/mod.rs`
- Create: `cli/src/ui/state.rs`
- Create: `cli/src/ui/agents.rs`
- Create: `cli/src/ui/tasks.rs`
- Create: `cli/src/ui/pty.rs`
- Modify: `cli/src/tui.rs`

- [ ] **Step 1: Define `TuiState`**
In `cli/src/ui/state.rs`, create a `TuiState` struct containing:
- `agents`: List/Map of discovered agents and their statuses.
- `tasks`: List/Map of tasks and their statuses (Queued, Routed).
- `logs`: Map of `task_id` -> `Vec<String>` representing PTY output chunks.
- `selected_task_id`: `Option<String>` to track which task's logs to show.
Implement a `process_event(&mut self, event: core::events::Event)` method to update this state.

- [ ] **Step 2: Component Skeleton**
In the component files (`agents.rs`, `tasks.rs`, `pty.rs`), define a generic `pub fn draw(f: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &TuiState)` function. For now, they can just render simple placeholder blocks (e.g., `Block::default().title("Agents").borders(Borders::ALL)`).

- [ ] **Step 3: State Tests**
In `cli/src/ui/state.rs`, add unit tests validating that `process_event` correctly updates lists when it receives `TaskQueued`, `TaskRouted`, `PtyOutput`, and `TaskCompleted`.

- [ ] **Step 4: Commit**
Commit changes as `feat(cli): establish TUI state and component architecture`

---

### Task 2: 3-Column Layout & Event Integration
Wire the state into the main `run_tui` loop and split the screen real-estate.

**Files:**
- Modify: `cli/src/tui.rs`

- [ ] **Step 1: Update Layout**
In `cli/src/tui.rs` inside the `terminal.draw` closure, use `Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(20), Constraint::Percentage(30), Constraint::Percentage(50)])` to slice the frame into three `Rect`s.

- [ ] **Step 2: Delegate Drawing**
Call `ui::agents::draw(f, chunks[0], &state)`, `ui::tasks::draw(f, chunks[1], &state)`, and `ui::pty::draw(f, chunks[2], &state)`.

- [ ] **Step 3: Event Loop Mutability**
Instantiate `TuiState` before the loop. Inside the event loop, after `try_recv()` pulls an event from the IPC background thread, call `state.process_event(event)`.

- [ ] **Step 4: Commit**
Commit changes as `feat(cli): wire 3-column layout and real-time state updates`

---

### Task 3: Render Implementations & Interactions
Make the components actually show the data and add keyboard navigation to select tasks.

**Files:**
- Modify: `cli/src/ui/tasks.rs`
- Modify: `cli/src/ui/pty.rs`
- Modify: `cli/src/tui.rs`

- [ ] **Step 1: Render Task & Agent Data**
Update `agents.rs` and `tasks.rs` to iterate over `state.agents` and `state.tasks`, mapping them to `ListItem`s to show inside a `List` widget.

- [ ] **Step 2: Render PTY Logs**
Update `pty.rs`. If `state.selected_task_id` is `Some`, fetch the corresponding log vector from `state.logs`. Render it as a `Paragraph` or `List`. (Keep it simple, just string concatenation of the chunks for now, Ratatui's `Paragraph` will handle basic wrapping).

- [ ] **Step 3: Interactive Selection**
In `tui.rs`, update the keyboard input handling (`crossterm::event::Event::Key`). Add `Up` and `Down` arrow keys to change the `selected_task_id` in `TuiState`. (You may need a `ListState` from Ratatui in `TuiState` to manage list selection indices properly).

- [ ] **Step 4: Commit**
Commit changes as `feat(cli): implement interactive task selection and data rendering`
