use std::io::Read;
use std::process::Command;
use portable_pty::{Child, MasterPty};

pub struct PtyProcess {
    pub reader: Box<dyn Read + Send>,
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
        
        Ok(PtyProcess {
            reader,
            child,
            master: pair.master,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::io::Read;

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
                    if output.contains("hello") { break; }
                }
                _ => break,
            }
        }
        
        let exit_status = pty_process.child.wait().expect("failed to wait on child");
        println!("Child exited with: {:?}", exit_status);
        println!("Output: {:?}", output);
        assert!(output.contains("hello"));
    }
}
