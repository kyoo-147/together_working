use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportClass {
    Compact,
    Medium,
    Wide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CockpitAreas {
    pub viewport: ViewportClass,
    pub header: Rect,
    pub left: Option<Rect>,
    pub center: Rect,
    pub right: Option<Rect>,
    pub command: Rect,
}

pub fn classify(width: u16) -> ViewportClass {
    if width < 84 {
        ViewportClass::Compact
    } else if width < 120 {
        ViewportClass::Medium
    } else {
        ViewportClass::Wide
    }
}

pub fn cockpit_areas(frame: Rect, drawer_open: bool) -> CockpitAreas {
    let viewport = classify(frame.width);
    let header_height = if frame.height < 22 { 2 } else { 3 };
    let command_height = if frame.height < 18 { 1 } else { 2 };
    let shell = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(6),
            Constraint::Length(command_height),
        ])
        .split(frame);

    let (left, center, right) = match viewport {
        ViewportClass::Wide => {
            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(26),
                    Constraint::Min(50),
                    Constraint::Length(36),
                ])
                .split(shell[1]);
            (Some(body[0]), body[1], Some(body[2]))
        }
        ViewportClass::Medium if drawer_open => {
            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(44), Constraint::Length(34)])
                .split(shell[1]);
            (None, body[0], Some(body[1]))
        }
        ViewportClass::Medium => {
            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(24), Constraint::Min(44)])
                .split(shell[1]);
            (Some(body[0]), body[1], None)
        }
        ViewportClass::Compact => (None, shell[1], None),
    };

    CockpitAreas {
        viewport,
        header: shell[0],
        left,
        center,
        right,
        command: shell[2],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_layout_keeps_three_columns() {
        let areas = cockpit_areas(Rect::new(0, 0, 140, 36), false);

        assert_eq!(areas.viewport, ViewportClass::Wide);
        assert!(areas.left.is_some());
        assert!(areas.right.is_some());
        assert!(areas.center.width >= 50);
    }

    #[test]
    fn medium_layout_prioritizes_left_and_center_when_not_editing() {
        let areas = cockpit_areas(Rect::new(0, 0, 100, 30), false);

        assert_eq!(areas.viewport, ViewportClass::Medium);
        assert!(areas.left.is_some());
        assert!(areas.right.is_none());
        assert!(areas.center.width >= 44);
    }

    #[test]
    fn medium_layout_prioritizes_drawer_when_editing() {
        let areas = cockpit_areas(Rect::new(0, 0, 100, 30), true);

        assert_eq!(areas.viewport, ViewportClass::Medium);
        assert!(areas.left.is_none());
        assert!(areas.right.is_some());
        assert_eq!(areas.right.unwrap().width, 34);
    }

    #[test]
    fn compact_layout_uses_single_main_column() {
        let areas = cockpit_areas(Rect::new(0, 0, 70, 20), true);

        assert_eq!(areas.viewport, ViewportClass::Compact);
        assert!(areas.left.is_none());
        assert!(areas.right.is_none());
        assert_eq!(areas.center.width, 70);
        assert_eq!(areas.header.height, 2);
        assert_eq!(areas.command.height, 2);
    }
}
