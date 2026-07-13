use core::events::AgentStatus;
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};

pub const BG: Color = Color::Rgb(224, 229, 242);
pub const PANEL: Color = Color::Rgb(237, 241, 249);
pub const PANEL_ALT: Color = Color::Rgb(214, 221, 237);
pub const BORDER: Color = Color::Rgb(148, 163, 199);
pub const TEXT: Color = Color::Rgb(48, 57, 83);
pub const MUTED: Color = Color::Rgb(101, 113, 150);
pub const ACCENT: Color = Color::Rgb(54, 103, 214);
pub const READY: Color = Color::Rgb(55, 121, 82);
pub const WARN: Color = Color::Rgb(191, 105, 48);
pub const DANGER: Color = Color::Rgb(191, 62, 75);

pub fn base_style() -> Style {
    Style::default().fg(TEXT).bg(BG)
}

pub fn panel_style() -> Style {
    Style::default().fg(TEXT).bg(PANEL)
}

pub fn selected_style() -> Style {
    Style::default()
        .fg(Color::White)
        .bg(ACCENT)
        .add_modifier(Modifier::BOLD)
}

pub fn panel_block(title: &str) -> Block<'_> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER).bg(PANEL))
        .style(panel_style())
}

pub fn status_label(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::Ready => "ready",
        AgentStatus::Busy => "running",
        AgentStatus::Degraded { .. } => "degraded",
        AgentStatus::Offline => "offline",
        AgentStatus::Blocked => "blocked",
        AgentStatus::Cooldown { .. } => "cooldown",
        AgentStatus::Unknown => "unknown",
    }
}

pub fn status_style(status: &AgentStatus) -> Style {
    let color = match status {
        AgentStatus::Ready => READY,
        AgentStatus::Busy => ACCENT,
        AgentStatus::Degraded { .. } | AgentStatus::Cooldown { .. } => WARN,
        AgentStatus::Offline | AgentStatus::Blocked => DANGER,
        AgentStatus::Unknown => MUTED,
    };
    Style::default()
        .fg(color)
        .bg(PANEL)
        .add_modifier(Modifier::BOLD)
}

pub fn format_agent_row(name: &str, status: &AgentStatus, max_width: usize) -> String {
    let mut row = format!("{name} - {}", status_label(status));
    match status {
        AgentStatus::Degraded { reason } | AgentStatus::Cooldown { reason }
            if !reason.trim().is_empty() =>
        {
            row.push_str(" - ");
            row.push_str(reason.trim());
        }
        _ => {}
    }
    truncate(&row, max_width)
}

pub fn short_task_id(task_id: &str) -> String {
    if task_id.len() > 8 {
        task_id.chars().take(8).collect()
    } else {
        task_id.to_string()
    }
}

pub fn truncate(value: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let count = value.chars().count();
    if count <= max_width {
        return value.to_string();
    }
    if max_width <= 1 {
        return "...".chars().take(max_width).collect();
    }
    let mut truncated = value
        .chars()
        .take(max_width.saturating_sub(1))
        .collect::<String>();
    truncated.push('~');
    truncated
}

#[cfg(test)]
mod tests {
    use core::events::AgentStatus;

    #[test]
    fn formats_degraded_agent_without_debug_noise() {
        let row = super::format_agent_row(
            "codex",
            &AgentStatus::Degraded {
                reason: "Access is denied while launching codex.exe from WindowsApps".to_string(),
            },
            36,
        );

        assert!(row.contains("codex"));
        assert!(row.contains("degraded"));
        assert!(row.contains("Access is denied"));
        assert!(!row.contains("Degraded {"));
        assert!(row.len() <= 36);
    }

    #[test]
    fn short_task_id_preserves_recognition() {
        assert_eq!(super::short_task_id("12345678-90ab-cdef"), "12345678");
        assert_eq!(super::short_task_id("draft"), "draft");
    }
}
