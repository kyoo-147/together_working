use super::format::{self, panel_block, ACCENT, MUTED, PANEL, PANEL_ALT, TEXT};
use super::state::TuiState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, state: &TuiState, focused: bool) {
    let header_height = if area.height < 16 { 3 } else { 4 };
    let input_height = if area.height < 14 { 2 } else { 3 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(4),
            Constraint::Length(input_height),
        ])
        .split(area);

    let selected = state.selected_task_id.as_deref();

    draw_task_header(f, chunks[0], state);
    draw_output(f, chunks[1], state, selected);
    draw_input_strip(f, chunks[2], focused, selected.is_some());
}

fn draw_task_header(f: &mut Frame, area: Rect, state: &TuiState) {
    let mut lines = Vec::new();
    if let Some(detail) = state.selected_task_detail() {
        lines.push(Line::from(vec![
            Span::styled(
                detail.title.as_deref().unwrap_or("untitled task"),
                Style::default()
                    .fg(TEXT)
                    .bg(PANEL)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", format::short_task_id(&detail.task_id)),
                Style::default().fg(MUTED).bg(PANEL),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("scope ", Style::default().fg(MUTED).bg(PANEL)),
            Span::styled(
                format::truncate(
                    &detail.scope_summary,
                    area.width.saturating_sub(10) as usize,
                ),
                Style::default().fg(TEXT).bg(PANEL),
            ),
        ]));
        if area.height < 4 {
            lines.truncate(1);
        }
    } else {
        lines.push(Line::from(vec![Span::styled(
            "No task selected",
            Style::default()
                .fg(TEXT)
                .bg(PANEL)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "Press n to create a scoped contract",
            Style::default().fg(MUTED).bg(PANEL),
        )]));
        if area.height < 4 {
            lines.truncate(1);
        }
    }
    f.render_widget(
        Paragraph::new(lines)
            .block(panel_block("task execution"))
            .style(Style::default().bg(PANEL)),
        area,
    );
}

fn draw_output(f: &mut Frame, area: Rect, state: &TuiState, selected: Option<&str>) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let lines = if let Some(task_id) = selected {
        let mut lines = Vec::new();
        if let Some(route) = state.route_decisions.get(task_id) {
            lines.push(Line::from(vec![
                Span::styled("route ", Style::default().fg(MUTED).bg(PANEL)),
                Span::styled(
                    format::truncate(route, area.width.saturating_sub(10) as usize),
                    Style::default().fg(ACCENT).bg(PANEL),
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
                        Style::default().fg(ACCENT).bg(PANEL),
                    ))
                } else {
                    Line::from(Span::styled(
                        format::truncate(line, area.width.saturating_sub(3) as usize),
                        Style::default().fg(TEXT).bg(PANEL),
                    ))
                }
            }));
        } else {
            lines.push(Line::from(Span::styled(
                "waiting for worker output...",
                Style::default().fg(MUTED).bg(PANEL),
            )));
        }
        lines
    } else {
        let mut empty = vec![
            Line::from(Span::styled(
                "Create a task to start a real PTY worker.",
                Style::default()
                    .fg(TEXT)
                    .bg(PANEL)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "Required contract: title, department, scope, allowed files, success criteria.",
                Style::default().fg(MUTED).bg(PANEL),
            )),
            Line::from(Span::styled(
                "Codex is preferred; degraded agents fall back to the next healthy worker.",
                Style::default().fg(MUTED).bg(PANEL),
            )),
        ];
        empty.truncate(visible_height.max(1));
        empty
    };

    f.render_widget(
        Paragraph::new(lines)
            .block(panel_block("pty stream"))
            .style(Style::default().bg(PANEL)),
        area,
    );
}

fn draw_input_strip(f: &mut Frame, area: Rect, focused: bool, has_task: bool) {
    let status = if focused {
        "PTY focus active - type to send input - Esc detaches"
    } else if has_task {
        "Enter focuses PTY input - j/k selects tasks - n creates a task"
    } else {
        "n creates a task - q quits"
    };
    let status = format::truncate(status, area.width.saturating_sub(2) as usize);
    let style = if focused {
        Style::default().fg(Color::White).bg(ACCENT)
    } else {
        Style::default().fg(TEXT).bg(PANEL_ALT)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .style(style);
    f.render_widget(Paragraph::new(status).style(style).block(block), area);
}
