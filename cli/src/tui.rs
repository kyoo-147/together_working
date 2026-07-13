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
use crate::ui::format::{self, themed_base_style, themed_panel_block, themed_selected_style};
use crate::ui::layout::{cockpit_areas, ViewportClass};
use crate::ui::state::TuiState;
use crate::ui::theme::{presets, theme_from_settings, Theme};
use crate::ui::wizard::ContractWizard;
use crate::ui::{agents, pty, tasks};
use core::chat::ChatSource;
use core::ipc::{Command, Response};
use core::settings::UiSettings;

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
    ChatInput,
    ProposalPreview,
    TaskContract,
    Settings,
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

    let mut state = TuiState {
        settings: daemon::settings::load_settings(&std::env::current_dir()?),
        ..TuiState::default()
    };
    let mut mode = Mode::Navigate;
    let mut wizard = ContractWizard::new();
    let mut chat_input = String::new();
    let mut settings = SettingsPanel::from_settings(&state.settings);
    let mut status = "ready".to_string();

    loop {
        while let Ok(event) = rx.try_recv() {
            state.process_event(event);
            settings.sync_from_state(&state.settings);
        }

        terminal
            .draw(|f| draw_cockpit(f, mode, &state, &wizard, &chat_input, &settings, &status))?;

        if event::poll(Duration::from_millis(50))? {
            if let CEvent::Key(key) = event::read()? {
                if handle_key(
                    key,
                    &mut mode,
                    &mut wizard,
                    &mut chat_input,
                    &mut settings,
                    &mut state,
                    &mut status,
                )? {
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
    chat_input: &str,
    settings: &SettingsPanel,
    status: &str,
) {
    let frame = f.size();
    let theme = theme_from_settings(&settings.current_settings());
    f.render_widget(Block::default().style(themed_base_style(&theme)), frame);
    let areas = cockpit_areas(frame, matches!(mode, Mode::TaskContract | Mode::Settings));

    draw_header(f, areas.header, areas.viewport, mode, state, &theme);

    if let Some(left) = areas.left {
        tasks::draw(f, left, state, &theme);
    }

    if matches!(mode, Mode::TaskContract) && areas.right.is_none() {
        draw_contract_drawer(f, areas.center, wizard, &theme);
    } else if matches!(mode, Mode::Settings) && areas.right.is_none() {
        draw_settings_panel(f, areas.center, settings, &theme);
    } else {
        pty::draw(
            f,
            areas.center,
            state,
            mode == Mode::PtyFocus,
            mode == Mode::ChatInput,
            chat_input,
            &theme,
        );
    }

    if let Some(right) = areas.right {
        if mode == Mode::TaskContract {
            draw_contract_drawer(f, right, wizard, &theme);
        } else if mode == Mode::Settings {
            draw_settings_panel(f, right, settings, &theme);
        } else {
            draw_right_rail(f, right, state, &theme);
        }
    }

    draw_command_bar(f, areas.command, areas.viewport, mode, status, &theme);
}

fn draw_header(
    f: &mut Frame,
    area: Rect,
    viewport: ViewportClass,
    mode: Mode,
    state: &TuiState,
    theme: &Theme,
) {
    let task_label = state
        .selected_task_detail()
        .and_then(|detail| detail.title.as_deref())
        .unwrap_or("no active task");
    let mode_label = match mode {
        Mode::Navigate => "monitor",
        Mode::PtyFocus => "pty focus",
        Mode::ChatInput => "chat",
        Mode::ProposalPreview => "proposal",
        Mode::TaskContract => "task contract",
        Mode::Settings => "settings",
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
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Project", Style::default().fg(theme.text).bg(theme.panel)),
    ];
    if !branch.is_empty() {
        spans.push(Span::styled(
            branch,
            Style::default().fg(theme.muted).bg(theme.panel),
        ));
    }
    if viewport != ViewportClass::Compact {
        spans.push(Span::styled(
            " daemon local",
            Style::default().fg(theme.ready).bg(theme.panel),
        ));
    }
    spans.extend([
        Span::styled(
            format!(" mode {mode_label}"),
            Style::default().fg(theme.accent).bg(theme.panel),
        ),
        Span::styled(
            format!(" task {}", format::truncate(task_label, task_width)),
            Style::default().fg(theme.muted).bg(theme.panel),
        ),
    ]);
    let line = Line::from(spans);

    f.render_widget(
        Paragraph::new(line)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(theme.border).bg(theme.bg)),
            )
            .style(Style::default().bg(theme.panel)),
        area,
    );
}

fn draw_right_rail(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);
    agents::draw(f, chunks[0], state, theme);
    draw_needs_attention(f, chunks[1], state, theme);
}

fn draw_needs_attention(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme) {
    let mut lines = Vec::new();
    if !state.needs_attention.is_empty() {
        for item in state
            .needs_attention
            .iter()
            .take(area.height.saturating_sub(2) as usize)
        {
            lines.push(Line::from(Span::styled(
                format!(
                    "- {}",
                    format::truncate(item, area.width.saturating_sub(4) as usize)
                ),
                Style::default().fg(theme.warn).bg(theme.panel),
            )));
        }
    } else if let Some(detail) = state.selected_task_detail() {
        let approval = if detail.approval_blocked {
            Span::styled(
                "blocked",
                Style::default()
                    .fg(theme.danger)
                    .bg(theme.panel)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                "pending",
                Style::default()
                    .fg(theme.warn)
                    .bg(theme.panel)
                    .add_modifier(Modifier::BOLD),
            )
        };
        lines.push(Line::from(vec![
            Span::styled(
                "approval ",
                Style::default().fg(theme.muted).bg(theme.panel),
            ),
            approval,
        ]));
        if let Some(verification) = &detail.verification {
            lines.push(Line::from(Span::styled(
                format::truncate(verification, area.width.saturating_sub(3) as usize),
                Style::default().fg(theme.text).bg(theme.panel),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "verification waits for task completion",
                Style::default().fg(theme.muted).bg(theme.panel),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "timeline",
            Style::default().fg(theme.muted).bg(theme.panel),
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
                Style::default().fg(theme.text).bg(theme.panel),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "No gate event yet.",
            Style::default().fg(theme.muted).bg(theme.panel),
        )));
        lines.push(Line::from(Span::styled(
            "Blocked approvals and worker prompts appear here.",
            Style::default().fg(theme.muted).bg(theme.panel),
        )));
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(themed_panel_block("Needs Attention", theme))
            .style(Style::default().bg(theme.panel)),
        area,
    );
}

fn draw_contract_drawer(f: &mut Frame, area: Rect, wizard: &ContractWizard, theme: &Theme) {
    let mut lines = vec![
        Line::from(Span::styled(
            "New scoped task",
            Style::default()
                .fg(theme.text)
                .bg(theme.panel)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            wizard.status(),
            Style::default().fg(theme.muted).bg(theme.panel),
        )),
        Line::from(""),
    ];

    for (index, field) in wizard.fields().iter().enumerate() {
        let focused = index == wizard.current();
        let required = if field.required { " *" } else { "" };
        let label_style = if focused {
            themed_selected_style(theme)
        } else {
            Style::default().fg(theme.muted).bg(theme.panel)
        };
        let value = if field.value.is_empty() {
            field.placeholder
        } else {
            field.value.as_str()
        };
        let value_style = if field.value.is_empty() {
            Style::default().fg(theme.muted).bg(theme.panel)
        } else if focused {
            Style::default()
                .fg(theme.accent)
                .bg(theme.panel)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text).bg(theme.panel)
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
                .fg(theme.ready)
                .bg(theme.panel)
                .add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "contract needs attention",
            Style::default()
                .fg(theme.warn)
                .bg(theme.panel)
                .add_modifier(Modifier::BOLD),
        )));
        for error in errors.iter().take(3) {
            lines.push(Line::from(Span::styled(
                format!("- {error}"),
                Style::default().fg(theme.warn).bg(theme.panel),
            )));
        }
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(themed_panel_block("Task Contract", theme))
            .style(Style::default().bg(theme.panel)),
        area,
    );
}

fn draw_settings_panel(f: &mut Frame, area: Rect, settings: &SettingsPanel, theme: &Theme) {
    let all = presets();
    let mut lines = vec![
        Line::from(Span::styled(
            "Theme Settings",
            Style::default()
                .fg(theme.text)
                .bg(theme.panel)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "j/k preset - b bg - m main - Enter applies - r resets",
            Style::default().fg(theme.muted).bg(theme.panel),
        )),
        Line::from(""),
    ];

    let visible = area.height.saturating_sub(8) as usize;
    let start = settings.selected.saturating_sub(visible.saturating_sub(1));
    for (index, preset) in all.iter().enumerate().skip(start).take(visible.max(1)) {
        let selected = index == settings.selected;
        let style = if selected {
            themed_selected_style(theme)
        } else {
            Style::default().fg(theme.text).bg(theme.panel)
        };
        lines.push(Line::from(Span::styled(
            format!(
                "{} {}  bg {}  main {}",
                if selected { ">" } else { " " },
                preset.name,
                preset.bg,
                preset.main
            ),
            style,
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "custom bg ",
            Style::default().fg(theme.muted).bg(theme.panel),
        ),
        Span::styled(
            settings.custom_display(CustomField::Bg),
            Style::default().fg(theme.text).bg(theme.panel),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "custom main ",
            Style::default().fg(theme.muted).bg(theme.panel),
        ),
        Span::styled(
            settings.custom_display(CustomField::Main),
            Style::default().fg(theme.text).bg(theme.panel),
        ),
    ]));

    f.render_widget(
        Paragraph::new(lines)
            .block(themed_panel_block("Settings", theme))
            .style(Style::default().bg(theme.panel)),
        area,
    );
}

fn draw_command_bar(
    f: &mut Frame,
    area: Rect,
    viewport: ViewportClass,
    mode: Mode,
    status: &str,
    theme: &Theme,
) {
    let help = match (viewport, mode) {
        (ViewportClass::Compact, Mode::Navigate) => "n new | Enter focus | q quit",
        (ViewportClass::Compact, Mode::PtyFocus) => "type input | Esc detach",
        (ViewportClass::Compact, Mode::TaskContract) => "Tab next | Ctrl+Enter send | Esc cancel",
        (ViewportClass::Compact, Mode::ChatInput) => "Enter send | Ctrl+Enter confirm | Esc",
        (ViewportClass::Compact, Mode::Settings) => "j/k theme | b/m color | Enter apply | Esc",
        (_, Mode::Navigate) => {
            "n new task | / ask | s settings | j/k select | Enter focus PTY | q quit"
        }
        (_, Mode::PtyFocus) => "typing sends input | Enter newline | Esc detach",
        (_, Mode::ChatInput) => "Enter send chat | Ctrl+Enter confirm proposal | Esc cancel",
        (_, Mode::ProposalPreview) => "Ctrl+Enter confirm proposal | Esc reject",
        (_, Mode::TaskContract) => {
            "Tab next | Shift+Tab previous | Ctrl+Enter dispatch | Esc cancel"
        }
        (_, Mode::Settings) => "j/k theme | b bg | m main | Enter apply | r reset | Esc close",
    };
    let status_width = area.width.saturating_sub(help.len() as u16 + 8) as usize;
    let line = Line::from(vec![
        Span::styled(" ", Style::default().bg(theme.panel)),
        Span::styled(help, Style::default().fg(theme.text).bg(theme.panel)),
        Span::styled("  |  ", Style::default().fg(theme.muted).bg(theme.panel)),
        Span::styled(
            format::truncate(status, status_width),
            Style::default().fg(theme.muted).bg(theme.panel),
        ),
    ]);
    f.render_widget(
        Paragraph::new(line)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(theme.border).bg(theme.panel)),
            )
            .style(Style::default().bg(theme.panel)),
        area,
    );
}

fn handle_key(
    key: KeyEvent,
    mode: &mut Mode,
    wizard: &mut ContractWizard,
    chat_input: &mut String,
    settings: &mut SettingsPanel,
    state: &mut TuiState,
    status: &mut String,
) -> Result<bool, io::Error> {
    match *mode {
        Mode::Navigate => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('n') => {
                *wizard = ContractWizard::new();
                *mode = Mode::TaskContract;
                *status = "editing task contract".to_string();
            }
            KeyCode::Char('/') | KeyCode::Char('i') => {
                *mode = Mode::ChatInput;
                *status = "chat dock active".to_string();
            }
            KeyCode::Char('s') => {
                *mode = Mode::Settings;
                *status = "settings open".to_string();
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
        Mode::ChatInput => match key.code {
            KeyCode::Esc => {
                chat_input.clear();
                *mode = Mode::Navigate;
                *status = "chat cancelled".to_string();
            }
            KeyCode::Backspace => {
                chat_input.pop();
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                confirm_latest_proposal(state, status)?;
                *mode = Mode::Navigate;
            }
            KeyCode::Enter => {
                if !chat_input.trim().is_empty() {
                    submit_chat(chat_input, status)?;
                    chat_input.clear();
                    *mode = Mode::ProposalPreview;
                }
            }
            KeyCode::Char(ch) => chat_input.push(ch),
            _ => {}
        },
        Mode::ProposalPreview => match key.code {
            KeyCode::Esc => {
                reject_latest_proposal(state, status)?;
                *mode = Mode::Navigate;
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                confirm_latest_proposal(state, status)?;
                *mode = Mode::Navigate;
            }
            KeyCode::Char('/') | KeyCode::Char('i') => {
                *mode = Mode::ChatInput;
                *status = "chat dock active".to_string();
            }
            _ => {}
        },
        Mode::TaskContract => match key.code {
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
        Mode::Settings => match key.code {
            KeyCode::Esc => {
                settings.cancel_edit();
                *mode = Mode::Navigate;
                *status = "settings closed".to_string();
            }
            KeyCode::Down | KeyCode::Char('j') if !settings.is_editing() => settings.next(),
            KeyCode::Up | KeyCode::Char('k') if !settings.is_editing() => settings.previous(),
            KeyCode::Char('b') if !settings.is_editing() => {
                settings.start_edit(CustomField::Bg);
                *status = "editing custom bg hex".to_string();
            }
            KeyCode::Char('m') if !settings.is_editing() => {
                settings.start_edit(CustomField::Main);
                *status = "editing custom main hex".to_string();
            }
            KeyCode::Char('r') => {
                *settings = SettingsPanel::from_settings(&UiSettings::default());
                update_settings(&settings.current_settings(), status)?;
            }
            KeyCode::Backspace if settings.is_editing() => settings.backspace(),
            KeyCode::Enter => {
                if settings.is_editing() {
                    settings.commit_edit();
                }
                update_settings(&settings.current_settings(), status)?;
            }
            KeyCode::Char(ch) if settings.is_editing() => settings.push_char(ch),
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
        Ok(Response::Proposal { proposal_id }) => {
            *status = format!("proposal {proposal_id}");
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

fn submit_chat(chat_input: &str, status: &mut String) -> Result<(), io::Error> {
    let cmd = Command::SubmitChat {
        source: ChatSource::TogetherChat,
        text: chat_input.trim().to_string(),
    };
    let response = send_command(cmd)?;
    match response {
        Response::Proposal { proposal_id } => {
            *status = format!("proposal ready {proposal_id}");
        }
        Response::Error { message } => {
            *status = format!("chat failed: {message}");
        }
        Response::Ack { task_id } => {
            *status = format!("chat accepted {task_id}");
        }
    }
    Ok(())
}

fn confirm_latest_proposal(state: &TuiState, status: &mut String) -> Result<(), io::Error> {
    let Some(proposal) = state.latest_pending_proposal() else {
        *status = "no pending proposal".to_string();
        return Ok(());
    };
    let response = send_command(Command::ConfirmProposal {
        proposal_id: proposal.proposal_id.clone(),
    })?;
    match response {
        Response::Ack { task_id } => {
            *status = format!("proposal confirmed: {task_id}");
        }
        Response::Proposal { proposal_id } => {
            *status = format!("proposal pending: {proposal_id}");
        }
        Response::Error { message } => {
            *status = format!("confirm failed: {message}");
        }
    }
    Ok(())
}

fn reject_latest_proposal(state: &TuiState, status: &mut String) -> Result<(), io::Error> {
    let Some(proposal) = state.latest_pending_proposal() else {
        *status = "no pending proposal".to_string();
        return Ok(());
    };
    let response = send_command(Command::RejectProposal {
        proposal_id: proposal.proposal_id.clone(),
    })?;
    match response {
        Response::Ack { .. } => {
            *status = "proposal rejected".to_string();
        }
        Response::Proposal { proposal_id } => {
            *status = format!("proposal pending: {proposal_id}");
        }
        Response::Error { message } => {
            *status = format!("reject failed: {message}");
        }
    }
    Ok(())
}

fn update_settings(settings: &UiSettings, status: &mut String) -> Result<(), io::Error> {
    let response = send_command(Command::UpdateSettings {
        settings: settings.clone(),
    })?;
    match response {
        Response::Ack { .. } => {
            *status = format!("theme set to {}", settings.theme_preset);
        }
        Response::Proposal { proposal_id } => {
            *status = format!("proposal pending: {proposal_id}");
        }
        Response::Error { message } => {
            *status = format!("settings failed: {message}");
        }
    }
    Ok(())
}

fn send_command(command: Command) -> Result<Response, io::Error> {
    let cmd_str = format!("{}\n", serde_json::to_string(&command).unwrap());
    let resp = client::send_command(client::DEFAULT_SOCKET_NAME, &cmd_str)?;
    serde_json::from_str::<Response>(&resp)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[derive(Debug, Clone)]
struct SettingsPanel {
    selected: usize,
    custom_bg: Option<String>,
    custom_main: Option<String>,
    editing: Option<CustomField>,
    edit_buffer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CustomField {
    Bg,
    Main,
}

impl SettingsPanel {
    fn from_settings(settings: &UiSettings) -> Self {
        let selected = presets()
            .iter()
            .position(|preset| preset.name == settings.theme_preset)
            .unwrap_or(0);
        Self {
            selected,
            custom_bg: settings.custom_bg.clone(),
            custom_main: settings.custom_main.clone(),
            editing: None,
            edit_buffer: String::new(),
        }
    }

    fn sync_from_state(&mut self, settings: &UiSettings) {
        if self.current_settings() != *settings {
            *self = Self::from_settings(settings);
        }
    }

    fn next(&mut self) {
        let len = presets().len().max(1);
        self.selected = (self.selected + 1) % len;
    }

    fn previous(&mut self) {
        let len = presets().len().max(1);
        self.selected = if self.selected == 0 {
            len - 1
        } else {
            self.selected - 1
        };
    }

    fn current_settings(&self) -> UiSettings {
        let all = presets();
        let preset = all
            .get(self.selected)
            .map(|preset| preset.name)
            .unwrap_or("Together Classic");
        UiSettings {
            theme_preset: preset.to_string(),
            custom_bg: self.custom_bg.clone(),
            custom_main: self.custom_main.clone(),
        }
    }

    fn is_editing(&self) -> bool {
        self.editing.is_some()
    }

    fn start_edit(&mut self, field: CustomField) {
        self.editing = Some(field);
        self.edit_buffer = match field {
            CustomField::Bg => self.custom_bg.clone(),
            CustomField::Main => self.custom_main.clone(),
        }
        .unwrap_or_else(|| "#".to_string());
    }

    fn cancel_edit(&mut self) {
        self.editing = None;
        self.edit_buffer.clear();
    }

    fn commit_edit(&mut self) {
        let value = self.edit_buffer.trim().to_string();
        let value = if value == "#" || value.is_empty() {
            None
        } else {
            Some(value)
        };
        match self.editing {
            Some(CustomField::Bg) => self.custom_bg = value,
            Some(CustomField::Main) => self.custom_main = value,
            None => {}
        }
        self.cancel_edit();
    }

    fn backspace(&mut self) {
        self.edit_buffer.pop();
    }

    fn push_char(&mut self, ch: char) {
        let can_push_hash = ch == '#' && self.edit_buffer.is_empty();
        let can_push_hex = ch.is_ascii_hexdigit() && self.edit_buffer.len() < 7;
        if can_push_hash || can_push_hex {
            self.edit_buffer.push(ch);
        }
    }

    fn custom_display(&self, field: CustomField) -> &str {
        if self.editing == Some(field) {
            self.edit_buffer.as_str()
        } else {
            match field {
                CustomField::Bg => self.custom_bg.as_deref().unwrap_or("none"),
                CustomField::Main => self.custom_main.as_deref().unwrap_or("none"),
            }
        }
    }
}
