use ratatui::{Frame, layout::Rect, widgets::{Block, Borders}};
use super::state::TuiState;

pub fn draw(f: &mut Frame, area: Rect, _state: &TuiState) {
    let block = Block::default().title("Agents").borders(Borders::ALL);
    f.render_widget(block, area);
}
