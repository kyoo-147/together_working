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
        let payload = serde_json::to_string(event)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            
        let event_value: serde_json::Value = serde_json::from_str(&payload)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            
        let event_type = event_value
            .as_object()
            .and_then(|obj| obj.keys().next())
            .map(|k| k.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

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
            let event: Event = serde_json::from_str(&payload)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
            Ok(event)
        })?;

        event_iter.collect::<SqlResult<Vec<Event>>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::events::Event;

    #[test]
    fn test_event_store_in_memory() {
        let store = EventStore::in_memory().unwrap();
        let event = Event::TaskRouted { task_id: "1".to_string(), agent_name: "test".to_string() };
        store.append(&event).unwrap();
        
        let events = store.get_all().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }

    #[test]
    fn test_event_store_file_based_and_multiple_events() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("test_store_{}.db", ts));
        
        {
            let store = EventStore::new(path.to_str().unwrap()).unwrap();
            let event1 = Event::TaskCreated { 
                task_id: "2".to_string(), 
                contract: core::contracts::TaskContract {
                    task_id: "2".to_string(),
                    department: None,
                    agent: None,
                }
            };
            let event2 = Event::TaskRouted { task_id: "2".to_string(), agent_name: "agent1".to_string() };
            
            store.append(&event1).unwrap();
            store.append(&event2).unwrap();
            
            let events = store.get_all().unwrap();
            assert_eq!(events.len(), 2);
            assert_eq!(events[0], event1);
            assert_eq!(events[1], event2);
        }
        
        let _ = std::fs::remove_file(path);
    }
}
