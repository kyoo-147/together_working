use super::state::TuiState;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, state: &TuiState, focused: bool) {
    let title = if focused {
        "Task execution / PTY [FOCUS]"
    } else {
        "Task execution / PTY"
    };
    let selected = state.selected_task_id.as_deref();
    let mut lines = Vec::new();
    if let Some(task_id) = selected {
        lines.push(format!("TASK {task_id}"));
        if let Some(route) = state.route_decisions.get(task_id) {
            lines.push(format!("route: {route}"));
        }
        lines.push(String::new());
        if let Some(logs) = state.logs.get(task_id) {
            lines.extend(logs.iter().cloned());
        } else {
            lines.push("waiting for PTY output...".to_string());
        }
    } else {
        lines.push("No task selected. Press n to create a contract.".to_string());
    }

    let paragraph =
        Paragraph::new(lines.join("")).block(Block::default().title(title).borders(Borders::ALL));
    f.render_widget(paragraph, area);
}
