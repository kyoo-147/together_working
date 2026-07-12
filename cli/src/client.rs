use interprocess::local_socket::{prelude::*, GenericNamespaced, Stream};
use std::io::{BufRead, BufReader, Write};

pub const DEFAULT_SOCKET_NAME: &str = "together.sock";

#[cfg(test)]
mod tests {
    use super::*;
    use core::ipc::Command;

    #[test]
    fn test_send_command_fails_without_server() {
        let socket_name = format!("together-test-{}-client.sock", std::process::id());
        let result = send_command(&socket_name, "test");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_command_serialization() {
        let sub_cmd = Command::Sub;
        let sub_str = serde_json::to_string(&sub_cmd).unwrap();
        assert_eq!(sub_str, "\"Sub\"");
        let deserialized_sub: Command = serde_json::from_str(&sub_str).unwrap();
        assert!(matches!(deserialized_sub, Command::Sub));
        
        let create_cmd = Command::CreateTask { yaml: "test".to_string() };
        let create_str = serde_json::to_string(&create_cmd).unwrap();
        assert_eq!(create_str, "{\"CreateTask\":{\"yaml\":\"test\"}}");
        let deserialized_create: Command = serde_json::from_str(&create_str).unwrap();
        assert!(matches!(deserialized_create, Command::CreateTask { yaml } if yaml == "test"));
    }
}

pub fn send_command(socket_name: &str, cmd: &str) -> Result<String, std::io::Error> {
    let name = socket_name.to_ns_name::<GenericNamespaced>()?;
    let mut conn = Stream::connect(name)?;
    
    let msg = format!("{}\n", cmd);
    conn.write_all(msg.as_bytes())?;
    
    let mut reader = BufReader::new(&mut conn);
    let mut response = String::new();
    reader.read_line(&mut response)?;
    Ok(response)
}

pub fn subscribe(socket_name: &str) -> Result<impl Iterator<Item = Result<core::events::Event, std::io::Error>>, std::io::Error> {
    let name = socket_name.to_ns_name::<GenericNamespaced>()?;
    let mut conn = Stream::connect(name)?;
    
    let cmd = core::ipc::Command::Sub;
    let msg = format!("{}\n", serde_json::to_string(&cmd).unwrap());
    conn.write_all(msg.as_bytes())?;
    
    let reader = BufReader::new(conn);
    Ok(reader.lines().map(|line_res| {
        let line = line_res?;
        serde_json::from_str(&line).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }))
}
