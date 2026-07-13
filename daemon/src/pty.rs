use portable_pty::{Child, MasterPty};
use std::io::Read;
use std::process::Command;

pub struct PtyProcess {
    pub reader: Box<dyn Read + Send>,
    pub writer: Box<dyn std::io::Write + Send>,
    pub child: Box<dyn Child + Send>,
    pub master: Box<dyn MasterPty + Send>,
}

pub struct PtyManager;

impl PtyManager {
    pub fn spawn(command: Command) -> Result<PtyProcess, Box<dyn std::error::Error + Send + Sync>> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system.openpty(portable_pty::PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = portable_pty::CommandBuilder::new(command.get_program());
        cmd.args(command.get_args());
        if let Some(dir) = command.get_current_dir() {
            cmd.cwd(dir);
        }
        for (key, val) in command.get_envs() {
            if let Some(v) = val {
                cmd.env(key, v);
            } else {
                cmd.env_remove(key);
            }
        }
        let child = pair.slave.spawn_command(cmd)?;

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        Ok(PtyProcess {
            reader,
            writer,
            child,
            master: pair.master,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::process::Command;

    #[test]
    fn test_pty_spawn() {
        let cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.arg("/c").arg("echo hello");
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg("echo hello");
            c
        };
        let mut pty_process = PtyManager::spawn(cmd).expect("failed to spawn pty");

        let mut output = String::new();
        let mut buf = [0u8; 1024];
        loop {
            match pty_process.reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    output.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if output.contains("hello") {
                        break;
                    }
                }
                _ => break,
            }
        }

        let exit_status = pty_process.child.wait().expect("failed to wait on child");
        println!("Child exited with: {:?}", exit_status);
        println!("Output: {:?}", output);
        assert!(output.contains("hello"));
    }

    #[test]
    fn test_pty_stdin_writer_sends_input() {
        use std::io::Write;
        use std::sync::mpsc::channel;

        let cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.arg("/q");
            c
        } else {
            Command::new("sh")
        };
        let mut pty_process = PtyManager::spawn(cmd).expect("failed to spawn pty");

        let mut reader = pty_process.reader;
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let mut output = String::new();
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        output.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if output.contains("typed:hello") {
                            let _ = tx.send(output);
                            break;
                        }
                    }
                    _ => break,
                }
            }
        });

        pty_process
            .writer
            .write_all(b"echo typed:hello\r\n")
            .unwrap();
        pty_process.writer.write_all(b"exit\r\n").unwrap();
        pty_process.writer.flush().unwrap();

        let output = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap_or_default();
        let _ = pty_process.child.wait();
        assert!(output.contains("typed:hello"), "Output was: {output:?}");
    }
}
