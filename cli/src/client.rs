use interprocess::local_socket::{prelude::*, GenericNamespaced, Stream};
use std::io::{BufRead, BufReader, Write};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_command_fails_without_server() {
        let socket_name = format!("together-test-{}-client.sock", std::process::id());
        let result = send_command(&socket_name, "test");
        assert!(result.is_err());
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
