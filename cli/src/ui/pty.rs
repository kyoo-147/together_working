use super::format;
use super::state::{TaskView, TuiState};
use super::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(
    f: &mut Frame,
    area: Rect,
    state: &TuiState,
    pty_focused: bool,
    chat_focused: bool,
    chat_input: &str,
    theme: &Theme,
) {
    let header_height = if area.height < 16 { 3 } else { 4 };
    let input_height = if area.height < 14 { 3 } else { 4 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(4),
            Constraint::Length(input_height),
        ])
        .split(area);

    let selected = state.selected_task_id.as_deref();

    draw_task_header(f, chunks[0], state, theme);
    draw_output(f, chunks[1], state, selected, theme);
    draw_chat_dock(
        f,
        chunks[2],
        state,
        chat_focused,
        chat_input,
        pty_focused,
        theme,
    );
}

fn draw_task_header(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme) {
    let mut lines = Vec::new();
    if let Some(detail) = state.selected_task_detail() {
        lines.push(Line::from(vec![
            Span::styled(
                detail.title.as_deref().unwrap_or("untitled task"),
                Style::default()
                    .fg(theme.text)
                    .bg(theme.panel)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", format::short_task_id(&detail.task_id)),
                Style::default().fg(theme.muted).bg(theme.panel),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("scope ", Style::default().fg(theme.muted).bg(theme.panel)),
            Span::styled(
                format::truncate(
                    &detail.scope_summary,
                    area.width.saturating_sub(10) as usize,
                ),
                Style::default().fg(theme.text).bg(theme.panel),
            ),
        ]));
        if area.height < 4 {
            lines.truncate(1);
        }
    } else {
        lines.push(Line::from(vec![Span::styled(
            "No task selected",
            Style::default()
                .fg(theme.text)
                .bg(theme.panel)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "Chat in Codex app or type below to create scoped work",
            Style::default().fg(theme.muted).bg(theme.panel),
        )]));
        if area.height < 4 {
            lines.truncate(1);
        }
    }
    f.render_widget(
        Paragraph::new(lines)
            .block(format::themed_panel_block("Task Monitor", theme))
            .style(Style::default().bg(theme.panel)),
        area,
    );
}

fn draw_output(f: &mut Frame, area: Rect, state: &TuiState, selected: Option<&str>, theme: &Theme) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let lines = if let Some(task_id) = selected {
        match state.current_view {
            TaskView::Monitor | TaskView::Logs => {
                task_log_lines(area, state, task_id, visible_height, theme)
            }
            TaskView::Diff => task_diff_lines(area, state, task_id, theme),
            TaskView::Review => task_review_lines(area, state, task_id, theme),
            TaskView::Verify => task_verify_lines(area, state, task_id, theme),
        }
    } else {
        empty_lines(visible_height, theme)
    };

    f.render_widget(
        Paragraph::new(lines)
            .block(format::themed_panel_block(
                state.current_view.title(),
                theme,
            ))
            .style(Style::default().bg(theme.panel)),
        area,
    );
}

fn task_log_lines(
    area: Rect,
    state: &TuiState,
    task_id: &str,
    visible_height: usize,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(route) = state.route_decisions.get(task_id) {
        lines.push(Line::from(vec![
            Span::styled("route ", Style::default().fg(theme.muted).bg(theme.panel)),
            Span::styled(
                format::truncate(route, area.width.saturating_sub(10) as usize),
                Style::default().fg(theme.accent).bg(theme.panel),
            ),
        ]));
    }
    if let Some(logs) = state.logs.get(task_id) {
        let start = logs
            .len()
            .saturating_sub(visible_height.saturating_sub(lines.len()));
        lines.extend(logs.iter().skip(start).map(|line| {
            if line.starts_with("> ") {
                Line::from(Span::styled(
                    format::truncate(line, area.width.saturating_sub(3) as usize),
                    Style::default().fg(theme.accent).bg(theme.panel),
                ))
            } else {
                Line::from(Span::styled(
                    format::truncate(line, area.width.saturating_sub(3) as usize),
                    Style::default().fg(theme.text).bg(theme.panel),
                ))
            }
        }));
    } else {
        lines.push(Line::from(Span::styled(
            "waiting for worker output...",
            Style::default().fg(theme.muted).bg(theme.panel),
        )));
    }
    lines
}

fn task_diff_lines(
    area: Rect,
    state: &TuiState,
    task_id: &str,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(Span::styled(
        "Changed-file diff summary appears after verification.",
        Style::default().fg(theme.muted).bg(theme.panel),
    ))];
    if let Some(detail) = state.task_detail(task_id) {
        lines.push(Line::from(Span::styled(
            format::truncate(&detail.scope_summary, area.width.saturating_sub(3) as usize),
            Style::default().fg(theme.text).bg(theme.panel),
        )));
    }
    lines
}

fn task_review_lines(
    _area: Rect,
    state: &TuiState,
    task_id: &str,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let state = state
        .task_detail(task_id)
        .and_then(|detail| detail.review_state.as_deref())
        .unwrap_or("pending_review");
    vec![
        Line::from(Span::styled(
            format!("review state: {state}"),
            Style::default().fg(theme.text).bg(theme.panel),
        )),
        Line::from(Span::styled(
            "Use approve / reject / request-changes commands for gate decisions.",
            Style::default().fg(theme.muted).bg(theme.panel),
        )),
    ]
}

fn task_verify_lines(
    area: Rect,
    state: &TuiState,
    task_id: &str,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let summary = state
        .task_detail(task_id)
        .and_then(|detail| detail.verification.as_deref())
        .unwrap_or("verification has not completed");
    vec![Line::from(Span::styled(
        format::truncate(summary, area.width.saturating_sub(3) as usize),
        Style::default().fg(theme.text).bg(theme.panel),
    ))]
}

fn empty_lines(visible_height: usize, theme: &Theme) -> Vec<Line<'static>> {
    let mut empty = vec![
        Line::from(Span::styled(
            "Together shows what Codex is coordinating behind the scenes.",
            Style::default()
                .fg(theme.text)
                .bg(theme.panel)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Use Codex chat for normal work, or use the dock below for direct requests.",
            Style::default().fg(theme.muted).bg(theme.panel),
        )),
        Line::from(Span::styled(
            "Requests become reviewable proposals before Together dispatches real tasks.",
            Style::default().fg(theme.muted).bg(theme.panel),
        )),
    ];
    empty.truncate(visible_height.max(1));
    empty
}

fn draw_chat_dock(
    f: &mut Frame,
    area: Rect,
    state: &TuiState,
    chat_focused: bool,
    chat_input: &str,
    pty_focused: bool,
    theme: &Theme,
) {
    let prefix = if chat_focused { "chat > " } else { "/ ask > " };
    let input = if chat_input.is_empty() && !chat_focused {
        "ask Codex to create or inspect work"
    } else {
        chat_input
    };
    let mut lines = vec![Line::from(vec![
        Span::styled(
            prefix,
            Style::default().fg(theme.accent).bg(theme.panel_alt),
        ),
        Span::styled(
            format::truncate(
                input,
                area.width.saturating_sub(prefix.len() as u16 + 3) as usize,
            ),
            Style::default()
                .fg(if chat_input.is_empty() {
                    theme.muted
                } else {
                    theme.text
                })
                .bg(theme.panel_alt),
        ),
    ])];
    if let Some(proposal) = state.latest_pending_proposal() {
        lines.push(Line::from(vec![
            Span::styled(
                "proposal ",
                Style::default().fg(theme.warn).bg(theme.panel_alt),
            ),
            Span::styled(
                format::truncate(&proposal.title, area.width.saturating_sub(13) as usize),
                Style::default().fg(theme.text).bg(theme.panel_alt),
            ),
        ]));
    } else if pty_focused {
        lines.push(Line::from(Span::styled(
            "PTY focus active - typing sends input, Esc detaches",
            Style::default().fg(theme.muted).bg(theme.panel_alt),
        )));
    }
    let style = if chat_focused {
        Style::default()
            .fg(if theme.dark {
                Color::Black
            } else {
                Color::White
            })
            .bg(theme.accent)
    } else {
        Style::default().fg(theme.text).bg(theme.panel_alt)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .style(style);
    f.render_widget(Paragraph::new(lines).style(style).block(block), area);
}
