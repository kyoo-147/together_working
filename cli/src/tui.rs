use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::collections::HashMap;
use std::io::{self, stdout};
use std::sync::mpsc;
use std::time::Duration;
use core::events::{AgentStatus, Event, RoutingTarget};
use crate::client;

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
    }
}

#[derive(Default)]
struct TuiState {
    agents: HashMap<String, AgentStatus>,
    queued_tasks: Vec<(String, RoutingTarget)>,
    routed_tasks: HashMap<String, String>,
}

pub fn run_tui() -> Result<(), io::Error> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel();

    let stream = match client::subscribe(client::DEFAULT_SOCKET_NAME) {
        Ok(s) => s,
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::ConnectionRefused, format!("Failed to connect to daemon (is it running?): {}", e)));
        }
    };

    std::thread::spawn(move || {
        for event_res in stream {
            if let Ok(event) = event_res {
                if tx.send(event).is_err() {
                    break;
                }
            } else {
                break;
            }
        }
    });

    let mut state = TuiState::default();

    loop {
        while let Ok(event) = rx.try_recv() {
            match event {
                Event::AgentStatusChanged { agent_name, status } => {
                    state.agents.insert(agent_name, status);
                }
                Event::TaskQueued { task_id, target } => {
                    state.queued_tasks.push((task_id, target));
                }
                Event::TaskRouted { task_id, agent_name } => {
                    state.queued_tasks.retain(|(id, _)| id != &task_id);
                    state.routed_tasks.insert(task_id, agent_name);
                }
                Event::TaskCreated { .. } => {}
            }
        }

        let agent_items: Vec<ListItem> = state.agents
            .iter()
            .map(|(name, status)| ListItem::new(format!("{} [{:?}]", name, status)))
            .collect();
            
        let mut task_items = Vec::new();
        for (id, target) in &state.queued_tasks {
            task_items.push(ListItem::new(format!("{} [Queued -> {:?}]", id, target)));
        }
        for (id, agent) in &state.routed_tasks {
            task_items.push(ListItem::new(format!("{} [Routed -> {}]", id, agent)));
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.size());

            let agent_list = List::new(agent_items).block(Block::default().title("Agents").borders(Borders::ALL));
            let task_list = List::new(task_items).block(Block::default().title("Tasks").borders(Borders::ALL));
            
            f.render_widget(agent_list, chunks[0]);
            f.render_widget(task_list, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(())
}
