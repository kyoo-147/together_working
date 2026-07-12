use core::events::Event;
use rusqlite::{params, Connection, Result as SqlResult};

pub struct EventStore {
    conn: Connection,
}

impl EventStore {
    pub fn in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init_db(&conn)?;
        Ok(Self { conn })
    }

    pub fn new(path: &str) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        Self::init_db(&conn)?;
        Ok(Self { conn })
    }

    fn init_db(conn: &Connection) -> SqlResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS event_log (
                id INTEGER PRIMARY KEY,
                event_type TEXT NOT NULL,
                payload TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn append(&self, event: &Event) -> SqlResult<()> {
        let payload = serde_json::to_string(event).unwrap();
        let event_type = match event {
            Event::TaskCreated { .. } => "TaskCreated",
            Event::TaskRouted { .. } => "TaskRouted",
        };
        self.conn.execute(
            "INSERT INTO event_log (event_type, payload) VALUES (?1, ?2)",
            params![event_type, payload],
        )?;
        Ok(())
    }

    pub fn get_all(&self) -> SqlResult<Vec<Event>> {
        let mut stmt = self.conn.prepare("SELECT payload FROM event_log ORDER BY id ASC")?;
        let event_iter = stmt.query_map([], |row| {
            let payload: String = row.get(0)?;
            let event: Event = serde_json::from_str(&payload).unwrap();
            Ok(event)
        })?;

        let mut events = Vec::new();
        for event in event_iter {
            events.push(event?);
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::events::Event;

    #[test]
    fn test_event_store() {
        let store = EventStore::in_memory().unwrap();
        let event = Event::TaskRouted { task_id: "1".to_string(), agent_name: "test".to_string() };
        store.append(&event).unwrap();
        
        let events = store.get_all().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }
}
