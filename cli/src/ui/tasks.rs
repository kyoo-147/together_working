use super::format::{self, panel_block, selected_style, MUTED, PANEL, TEXT};
use super::state::{TaskStatus, TuiState};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, state: &TuiState) {
    let mut items = vec![
        ListItem::new(Line::from(vec![Span::styled(
            "spaces",
            Style::default().fg(MUTED).bg(PANEL),
        )])),
        ListItem::new(Line::from(vec![Span::styled(
            "  Product",
            Style::default()
                .fg(TEXT)
                .bg(PANEL)
                .add_modifier(Modifier::BOLD),
        )])),
    ];
    if area.height >= 18 {
        items.extend([
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "departments",
                Style::default().fg(MUTED).bg(PANEL),
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
        "tasks",
        Style::default().fg(MUTED).bg(PANEL),
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
            selected_style()
        } else {
            Style::default().fg(TEXT).bg(PANEL)
        };
        items.push(ListItem::new(Line::from(row)).style(style));
        items.push(ListItem::new(Line::from(meta)).style(Style::default().fg(MUTED).bg(PANEL)));
    }
    if state.tasks.is_empty() {
        items.push(
            ListItem::new(Line::from("  n  new contract")).style(
                Style::default()
                    .fg(format::ACCENT)
                    .bg(PANEL)
                    .add_modifier(Modifier::BOLD),
            ),
        );
    }

    let list = List::new(items)
        .block(panel_block("workspace"))
        .style(Style::default().fg(TEXT).bg(PANEL));
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
