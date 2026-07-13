use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io::{self, stdout};
use std::sync::mpsc;
use std::time::Duration;

use crate::client;
use crate::ui::state::TuiState;
use crate::ui::{agents, pty, tasks};
use core::ipc::{Command, Response};

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Navigate,
    PtyFocus,
    Wizard,
}

#[derive(Debug, Clone)]
struct ContractWizard {
    fields: Vec<WizardField>,
    current: usize,
    status: String,
}

#[derive(Debug, Clone)]
struct WizardField {
    label: &'static str,
    value: String,
}

impl ContractWizard {
    fn new() -> Self {
        Self {
            fields: vec![
                WizardField {
                    label: "title",
                    value: "Fix scoped task".to_string(),
                },
                WizardField {
                    label: "department",
                    value: "engineering".to_string(),
                },
                WizardField {
                    label: "scope",
                    value: "src/**".to_string(),
                },
                WizardField {
                    label: "allowed_files",
                    value: "src/**".to_string(),
                },
                WizardField {
                    label: "denied_files",
                    value: ".env,**/secrets/*".to_string(),
                },
                WizardField {
                    label: "success_criteria",
                    value: "task completes".to_string(),
                },
            ],
            current: 0,
            status: "Enter: next field | Ctrl+Enter: dispatch | Esc: cancel".to_string(),
        }
    }

    fn push_char(&mut self, ch: char) {
        self.fields[self.current].value.push(ch);
    }

    fn backspace(&mut self) {
        self.fields[self.current].value.pop();
    }

    fn next(&mut self) {
        self.current = (self.current + 1).min(self.fields.len() - 1);
    }

    fn previous(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }

    fn yaml(&self) -> String {
        let title = self.value("title");
        let department = self.value("department");
        format!(
            "task_id: draft\ntitle: {}\ndepartment: {}\nscope:\n{}allowed_files:\n{}denied_files:\n{}success_criteria:\n{}reviewer_required: true\nverification_required: true\nmerge_authority: codex\nenforcement_mode: strict\nunknown_files_policy: needs_review\n",
            yaml_scalar(title),
            yaml_scalar(department),
            yaml_list(self.value("scope")),
            yaml_list(self.value("allowed_files")),
            yaml_list(self.value("denied_files")),
            yaml_list(self.value("success_criteria")),
        )
    }

    fn value(&self, label: &str) -> &str {
        self.fields
            .iter()
            .find(|field| field.label == label)
            .map(|field| field.value.as_str())
            .unwrap_or("")
    }

    fn render_text(&self) -> String {
        let mut lines = vec![
            "NEW TASK CONTRACT".to_string(),
            self.status.clone(),
            String::new(),
        ];
        for (index, field) in self.fields.iter().enumerate() {
            let marker = if index == self.current { ">" } else { " " };
            lines.push(format!("{marker} {}: {}", field.label, field.value));
        }
        lines.join("\n")
    }
}

pub fn run_tui() -> Result<(), io::Error> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel();
    let stream = client::subscribe(client::DEFAULT_SOCKET_NAME).map_err(|e| {
        io::Error::new(
            io::ErrorKind::ConnectionRefused,
            format!("Failed to connect to daemon: {e}"),
        )
    })?;

    std::thread::spawn(move || {
        for event_res in stream {
            if let Ok(event) = event_res {
                if tx.send(event).is_err() {
                    break;
                }
            } else {
                break;
            }
        }
    });

    let mut state = TuiState::default();
    let mut mode = Mode::Navigate;
    let mut wizard = ContractWizard::new();
    let mut status = "n: new task | Enter: PTY focus | q: quit".to_string();

    loop {
        while let Ok(event) = rx.try_recv() {
            state.process_event(event);
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                ])
                .split(f.size());

            tasks::draw(f, chunks[0], &state);
            pty::draw(f, chunks[1], &state, mode == Mode::PtyFocus);
            agents::draw(f, chunks[2], &state);

            if mode == Mode::Wizard {
                let area = centered_rect(70, 70, f.size());
                let block = Paragraph::new(wizard.render_text()).block(
                    Block::default()
                        .title("Guided contract wizard")
                        .borders(Borders::ALL),
                );
                f.render_widget(block, area);
            } else {
                let status_area = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(1), Constraint::Length(1)])
                    .split(f.size())[1];
                f.render_widget(Paragraph::new(status.as_str()), status_area);
            }
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let CEvent::Key(key) = event::read()? {
                if handle_key(key, &mut mode, &mut wizard, &mut state, &mut status)? {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn handle_key(
    key: KeyEvent,
    mode: &mut Mode,
    wizard: &mut ContractWizard,
    state: &mut TuiState,
    status: &mut String,
) -> Result<bool, io::Error> {
    match *mode {
        Mode::Navigate => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('n') => {
                *wizard = ContractWizard::new();
                *mode = Mode::Wizard;
            }
            KeyCode::Down | KeyCode::Char('j') => state.select_next_task(),
            KeyCode::Up | KeyCode::Char('k') => state.select_previous_task(),
            KeyCode::Enter if state.selected_task_id.is_some() => {
                *mode = Mode::PtyFocus;
                *status = "PTY focus: Esc to detach input".to_string();
            }
            _ => {}
        },
        Mode::PtyFocus => match key.code {
            KeyCode::Esc => {
                *mode = Mode::Navigate;
                *status = "n: new task | Enter: PTY focus | q: quit".to_string();
            }
            KeyCode::Enter => send_selected_input(state, "\r\n")?,
            KeyCode::Backspace => send_selected_input(state, "\u{8}")?,
            KeyCode::Char(ch) => send_selected_input(state, &ch.to_string())?,
            _ => {}
        },
        Mode::Wizard => match key.code {
            KeyCode::Esc => *mode = Mode::Navigate,
            KeyCode::Tab | KeyCode::Down => wizard.next(),
            KeyCode::BackTab | KeyCode::Up => wizard.previous(),
            KeyCode::Backspace => wizard.backspace(),
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                dispatch_contract(wizard, status)?;
                *mode = Mode::Navigate;
            }
            KeyCode::Enter => wizard.next(),
            KeyCode::Char(ch) => wizard.push_char(ch),
            _ => {}
        },
    }

    Ok(false)
}

fn send_selected_input(state: &TuiState, input: &str) -> Result<(), io::Error> {
    if let Some(task_id) = state.selected_task_id.as_deref() {
        let _ = client::send_input(client::DEFAULT_SOCKET_NAME, task_id, input)?;
    }
    Ok(())
}

fn dispatch_contract(wizard: &ContractWizard, status: &mut String) -> Result<(), io::Error> {
    let cmd = Command::CreateTask {
        yaml: wizard.yaml(),
    };
    let cmd_str = format!("{}\n", serde_json::to_string(&cmd).unwrap());
    let resp = client::send_command(client::DEFAULT_SOCKET_NAME, &cmd_str)?;
    match serde_json::from_str::<Response>(&resp) {
        Ok(Response::Ack { task_id }) => {
            *status = format!("dispatched task {task_id}");
        }
        Ok(Response::Error { message }) => {
            *status = format!("dispatch failed: {message}");
        }
        Err(_) => {
            *status = format!("daemon response: {resp}");
        }
    }
    Ok(())
}

fn yaml_scalar(value: &str) -> String {
    format!("{:?}", value)
}

fn yaml_list(value: &str) -> String {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| format!("  - {:?}\n", item))
        .collect::<String>()
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
