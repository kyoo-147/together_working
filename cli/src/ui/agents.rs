use super::format;
use super::state::TuiState;
use super::theme::Theme;
use core::events::AgentStatus;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme) {
    let mut agents = state.agents.iter().collect::<Vec<_>>();
    agents.sort_by_key(|(name, status)| (agent_sort_key(status), preferred_rank(name)));
    let is_empty = agents.is_empty();

    let mut items = agents
        .into_iter()
        .map(|(name, status)| {
            let max_width = area.width.saturating_sub(4) as usize;
            let row = format::format_agent_row(name, status, max_width);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{}] ", format::status_label(status)),
                    format::themed_status_style(status, theme),
                ),
                Span::styled(
                    row.replace(
                        &format!("{} - {} - ", name, format::status_label(status)),
                        "",
                    ),
                    Style::default().fg(theme.text).bg(theme.panel),
                ),
            ]))
            .style(Style::default().bg(theme.panel))
        })
        .collect::<Vec<_>>();
    if is_empty {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "no agents discovered",
            Style::default().fg(theme.muted).bg(theme.panel),
        )])));
    }
    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(vec![Span::styled(
        "routing priority: codex, cmdc, agy, claude",
        Style::default()
            .fg(theme.muted)
            .bg(theme.panel)
            .add_modifier(Modifier::ITALIC),
    )])));

    let list = List::new(items)
        .block(format::themed_panel_block("Agents", theme))
        .style(Style::default().fg(theme.text).bg(theme.panel));
    f.render_widget(list, area);
}

fn agent_sort_key(status: &AgentStatus) -> u8 {
    match status {
        AgentStatus::Busy => 0,
        AgentStatus::Ready => 1,
        AgentStatus::Degraded { .. } => 2,
        AgentStatus::Cooldown { .. } => 3,
        AgentStatus::Blocked => 4,
        AgentStatus::Offline => 5,
        AgentStatus::Unknown => 6,
    }
}

fn preferred_rank(name: &str) -> u8 {
    match name {
        "codex" => 0,
        "cmdc" => 1,
        "agy" => 2,
        "claude" => 3,
        _ => 9,
    }
}
