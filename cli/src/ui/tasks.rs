use super::state::{TaskStatus, TuiState};
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, state: &TuiState) {
    let mut items = vec![
        ListItem::new("PROJECT"),
        ListItem::new("Product"),
        ListItem::new(""),
        ListItem::new("DEPARTMENTS"),
        ListItem::new("engineering"),
        ListItem::new("integration"),
        ListItem::new("review"),
        ListItem::new(""),
        ListItem::new("TASKS"),
    ];

    let mut task_ids = state.tasks.keys().cloned().collect::<Vec<_>>();
    task_ids.sort();
    for id in task_ids {
        let marker = if state.selected_task_id.as_deref() == Some(id.as_str()) {
            ">"
        } else {
            " "
        };
        let status = match state.tasks.get(&id).unwrap() {
            TaskStatus::Queued(target) => format!("queued -> {target:?}"),
            TaskStatus::Routed(agent) => format!("running -> {agent}"),
            TaskStatus::Completed(success) => format!("completed success={success}"),
        };
        items.push(ListItem::new(format!("{marker} {id} {status}")));
    }
    if state.tasks.is_empty() {
        items.push(ListItem::new("n: new contract"));
    }

    let list = List::new(items).block(
        Block::default()
            .title("Workspace / Departments")
            .borders(Borders::ALL),
    );
    f.render_widget(list, area);
}
