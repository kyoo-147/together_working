use super::format;
use super::state::{TaskStatus, TuiState};
use super::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme) {
    let mut items = vec![
        ListItem::new(Line::from(vec![Span::styled(
            "Project",
            Style::default().fg(theme.muted).bg(theme.panel),
        )])),
        ListItem::new(Line::from(vec![Span::styled(
            "  Codex app",
            Style::default()
                .fg(theme.text)
                .bg(theme.panel)
                .add_modifier(Modifier::BOLD),
        )])),
    ];
    if area.height >= 18 {
        items.extend([
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "Sources",
                Style::default().fg(theme.muted).bg(theme.panel),
            )])),
            ListItem::new(Line::from("  Codex app")),
            ListItem::new(Line::from("  Together chat")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "Departments",
                Style::default().fg(theme.muted).bg(theme.panel),
            )])),
            ListItem::new(Line::from("  engineering")),
            ListItem::new(Line::from("  integration")),
            ListItem::new(Line::from("  review")),
            ListItem::new(Line::from("")),
        ]);
    } else {
        items.push(ListItem::new(Line::from("")));
    }
    items.push(ListItem::new(Line::from(vec![Span::styled(
        "Tasks",
        Style::default().fg(theme.muted).bg(theme.panel),
    )])));

    let mut task_ids = state.tasks.keys().cloned().collect::<Vec<_>>();
    task_ids.sort();
    for id in task_ids {
        let selected = state.selected_task_id.as_deref() == Some(id.as_str());
        let status = state.tasks.get(&id).unwrap();
        let detail = state.task_detail(&id);
        let title = detail
            .and_then(|detail| detail.title.as_deref())
            .unwrap_or("untitled task");
        let row = format!(
            "{} {}",
            status_marker(status),
            format::truncate(title, area.width.saturating_sub(8) as usize)
        );
        let meta = format!(
            "  {} - {}",
            format::short_task_id(&id),
            status_label(status)
        );
        let style = if selected {
            format::themed_selected_style(theme)
        } else {
            Style::default().fg(theme.text).bg(theme.panel)
        };
        items.push(ListItem::new(Line::from(row)).style(style));
        items.push(
            ListItem::new(Line::from(meta)).style(Style::default().fg(theme.muted).bg(theme.panel)),
        );
    }
    if state.tasks.is_empty() {
        items.push(
            ListItem::new(Line::from("  n  new task")).style(
                Style::default()
                    .fg(theme.accent)
                    .bg(theme.panel)
                    .add_modifier(Modifier::BOLD),
            ),
        );
    }

    let list = List::new(items)
        .block(format::themed_panel_block("Project", theme))
        .style(Style::default().fg(theme.text).bg(theme.panel));
    f.render_widget(list, area);
}

fn status_marker(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Draft => "+",
        TaskStatus::Queued(_) => "?",
        TaskStatus::Routed(_) => ">",
        TaskStatus::Completed(true) => "x",
        TaskStatus::Completed(false) => "!",
    }
}

fn status_label(status: &TaskStatus) -> String {
    match status {
        TaskStatus::Draft => "draft".to_string(),
        TaskStatus::Queued(target) => format!("queued {target:?}"),
        TaskStatus::Routed(agent) => format!("running {agent}"),
        TaskStatus::Completed(true) => "done".to_string(),
        TaskStatus::Completed(false) => "failed".to_string(),
    }
}
