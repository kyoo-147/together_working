use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io::{self, stdout};
use std::sync::mpsc;
use std::time::Duration;

use crate::client;
use crate::ui::format::{
    self, base_style, panel_block, selected_style, ACCENT, BG, MUTED, PANEL, TEXT,
};
use crate::ui::layout::{cockpit_areas, ViewportClass};
use crate::ui::state::TuiState;
use crate::ui::wizard::ContractWizard;
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
    let mut status = "ready".to_string();

    loop {
        while let Ok(event) = rx.try_recv() {
            state.process_event(event);
        }

        terminal.draw(|f| draw_cockpit(f, mode, &state, &wizard, &status))?;

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

fn draw_cockpit(
    f: &mut Frame,
    mode: Mode,
    state: &TuiState,
    wizard: &ContractWizard,
    status: &str,
) {
    let frame = f.size();
    f.render_widget(Block::default().style(base_style()), frame);
    let areas = cockpit_areas(frame, mode == Mode::Wizard);

    draw_header(f, areas.header, areas.viewport, mode, state);

    if let Some(left) = areas.left {
        tasks::draw(f, left, state);
    }

    if mode == Mode::Wizard && areas.right.is_none() {
        draw_contract_drawer(f, areas.center, wizard);
    } else {
        pty::draw(f, areas.center, state, mode == Mode::PtyFocus);
    }

    if let Some(right) = areas.right {
        if mode == Mode::Wizard {
            draw_contract_drawer(f, right, wizard);
        } else {
            draw_right_rail(f, right, state);
        }
    }

    draw_command_bar(f, areas.command, areas.viewport, mode, status);
}

fn draw_header(f: &mut Frame, area: Rect, viewport: ViewportClass, mode: Mode, state: &TuiState) {
    let task_label = state
        .selected_task_detail()
        .and_then(|detail| detail.title.as_deref())
        .unwrap_or("no active task");
    let mode_label = match mode {
        Mode::Navigate => "navigate",
        Mode::PtyFocus => "pty focus",
        Mode::Wizard => "new contract",
    };
    let branch = match viewport {
        ViewportClass::Compact => "",
        ViewportClass::Medium => " branch feat/mvp1",
        ViewportClass::Wide => " branch feat/mvp1-daemon-discovery",
    };
    let task_width = area.width.saturating_sub(match viewport {
        ViewportClass::Compact => 34,
        ViewportClass::Medium => 55,
        ViewportClass::Wide => 76,
    }) as usize;
    let mut spans = vec![
        Span::styled(
            " together ",
            Style::default()
                .fg(Color::White)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Product", Style::default().fg(TEXT).bg(PANEL)),
    ];
    if !branch.is_empty() {
        spans.push(Span::styled(branch, Style::default().fg(MUTED).bg(PANEL)));
    }
    if viewport != ViewportClass::Compact {
        spans.push(Span::styled(
            " daemon local",
            Style::default().fg(format::READY).bg(PANEL),
        ));
    }
    spans.extend([
        Span::styled(
            format!(" mode {mode_label}"),
            Style::default().fg(ACCENT).bg(PANEL),
        ),
        Span::styled(
            format!(" task {}", format::truncate(task_label, task_width)),
            Style::default().fg(MUTED).bg(PANEL),
        ),
    ]);
    let line = Line::from(spans);

    f.render_widget(
        Paragraph::new(line)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(format::BORDER).bg(BG)),
            )
            .style(Style::default().bg(PANEL)),
        area,
    );
}

fn draw_right_rail(f: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);
    agents::draw(f, chunks[0], state);
    draw_review_queue(f, chunks[1], state);
}

fn draw_review_queue(f: &mut Frame, area: Rect, state: &TuiState) {
    let mut lines = Vec::new();
    if let Some(detail) = state.selected_task_detail() {
        let approval = if detail.approval_blocked {
            Span::styled(
                "blocked",
                Style::default()
                    .fg(format::DANGER)
                    .bg(PANEL)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                "pending",
                Style::default()
                    .fg(format::WARN)
                    .bg(PANEL)
                    .add_modifier(Modifier::BOLD),
            )
        };
        lines.push(Line::from(vec![
            Span::styled("approval ", Style::default().fg(MUTED).bg(PANEL)),
            approval,
        ]));
        if let Some(verification) = &detail.verification {
            lines.push(Line::from(Span::styled(
                format::truncate(verification, area.width.saturating_sub(3) as usize),
                Style::default().fg(TEXT).bg(PANEL),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "verification waits for task completion",
                Style::default().fg(MUTED).bg(PANEL),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "timeline",
            Style::default().fg(MUTED).bg(PANEL),
        )));
        for entry in state
            .timeline_for(&detail.task_id)
            .iter()
            .rev()
            .take(5)
            .rev()
        {
            lines.push(Line::from(Span::styled(
                format!(
                    "- {}",
                    format::truncate(entry, area.width.saturating_sub(4) as usize)
                ),
                Style::default().fg(TEXT).bg(PANEL),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "No review item yet.",
            Style::default().fg(MUTED).bg(PANEL),
        )));
        lines.push(Line::from(Span::styled(
            "Create and run a task to see verification.",
            Style::default().fg(MUTED).bg(PANEL),
        )));
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(panel_block("review queue"))
            .style(Style::default().bg(PANEL)),
        area,
    );
}

fn draw_contract_drawer(f: &mut Frame, area: Rect, wizard: &ContractWizard) {
    let mut lines = vec![
        Line::from(Span::styled(
            "New scoped task",
            Style::default()
                .fg(TEXT)
                .bg(PANEL)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            wizard.status(),
            Style::default().fg(MUTED).bg(PANEL),
        )),
        Line::from(""),
    ];

    for (index, field) in wizard.fields().iter().enumerate() {
        let focused = index == wizard.current();
        let required = if field.required { " *" } else { "" };
        let label_style = if focused {
            selected_style()
        } else {
            Style::default().fg(MUTED).bg(PANEL)
        };
        let value = if field.value.is_empty() {
            field.placeholder
        } else {
            field.value.as_str()
        };
        let value_style = if field.value.is_empty() {
            Style::default().fg(MUTED).bg(PANEL)
        } else if focused {
            Style::default()
                .fg(ACCENT)
                .bg(PANEL)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT).bg(PANEL)
        };

        lines.push(Line::from(Span::styled(
            format!("{}{}", field.title, required),
            label_style,
        )));
        lines.push(Line::from(Span::styled(
            format!(
                "  {}",
                format::truncate(value, area.width.saturating_sub(5) as usize)
            ),
            value_style,
        )));
    }

    lines.push(Line::from(""));
    let errors = wizard.validation_errors();
    if errors.is_empty() {
        lines.push(Line::from(Span::styled(
            "ready to dispatch",
            Style::default()
                .fg(format::READY)
                .bg(PANEL)
                .add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "contract needs attention",
            Style::default()
                .fg(format::WARN)
                .bg(PANEL)
                .add_modifier(Modifier::BOLD),
        )));
        for error in errors.iter().take(3) {
            lines.push(Line::from(Span::styled(
                format!("- {error}"),
                Style::default().fg(format::WARN).bg(PANEL),
            )));
        }
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(panel_block("contract drawer"))
            .style(Style::default().bg(PANEL)),
        area,
    );
}

fn draw_command_bar(f: &mut Frame, area: Rect, viewport: ViewportClass, mode: Mode, status: &str) {
    let help = match (viewport, mode) {
        (ViewportClass::Compact, Mode::Navigate) => "n new | Enter focus | q quit",
        (ViewportClass::Compact, Mode::PtyFocus) => "type input | Esc detach",
        (ViewportClass::Compact, Mode::Wizard) => "Tab next | Ctrl+Enter send | Esc cancel",
        (_, Mode::Navigate) => "n new task | j/k select | Enter focus PTY | q quit",
        (_, Mode::PtyFocus) => "typing sends input | Enter newline | Esc detach",
        (_, Mode::Wizard) => "Tab next | Shift+Tab previous | Ctrl+Enter dispatch | Esc cancel",
    };
    let status_width = area.width.saturating_sub(help.len() as u16 + 8) as usize;
    let line = Line::from(vec![
        Span::styled(" ", Style::default().bg(PANEL)),
        Span::styled(help, Style::default().fg(TEXT).bg(PANEL)),
        Span::styled("  |  ", Style::default().fg(MUTED).bg(PANEL)),
        Span::styled(
            format::truncate(status, status_width),
            Style::default().fg(MUTED).bg(PANEL),
        ),
    ]);
    f.render_widget(
        Paragraph::new(line)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(format::BORDER).bg(PANEL)),
            )
            .style(Style::default().bg(PANEL)),
        area,
    );
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
                *status = "editing task contract".to_string();
            }
            KeyCode::Down | KeyCode::Char('j') => state.select_next_task(),
            KeyCode::Up | KeyCode::Char('k') => state.select_previous_task(),
            KeyCode::Enter if state.selected_task_id.is_some() => {
                *mode = Mode::PtyFocus;
                *status = "PTY focus active".to_string();
            }
            _ => {}
        },
        Mode::PtyFocus => match key.code {
            KeyCode::Esc => {
                *mode = Mode::Navigate;
                *status = "detached from PTY input".to_string();
            }
            KeyCode::Enter => send_selected_input(state, "\r\n")?,
            KeyCode::Backspace => send_selected_input(state, "\u{8}")?,
            KeyCode::Char(ch) => send_selected_input(state, &ch.to_string())?,
            _ => {}
        },
        Mode::Wizard => match key.code {
            KeyCode::Esc => {
                *mode = Mode::Navigate;
                *status = "contract cancelled".to_string();
            }
            KeyCode::Tab | KeyCode::Down | KeyCode::Enter
                if !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                wizard.next()
            }
            KeyCode::BackTab | KeyCode::Up => wizard.previous(),
            KeyCode::Backspace => wizard.backspace(),
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if wizard.is_valid() {
                    dispatch_contract(wizard, status)?;
                    *mode = Mode::Navigate;
                } else {
                    *status = wizard.validation_errors().join("; ");
                }
            }
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
