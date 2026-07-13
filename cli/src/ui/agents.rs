use super::state::TuiState;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, state: &TuiState) {
    let mut agents = state
        .agents
        .iter()
        .map(|(name, status)| ListItem::new(format!("{name} [{status:?}]")))
        .collect::<Vec<_>>();
    if agents.is_empty() {
        agents.push(ListItem::new("no agents discovered"));
    }
    let list = List::new(agents).block(Block::default().title("Agent pool").borders(Borders::ALL));
    f.render_widget(list, area);
}
