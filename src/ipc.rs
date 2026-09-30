use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub fn get_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("ironbar-todo.sock")
    } else {
        let user = std::env::var("USER").unwrap_or_else(|_| "current".to_string());
        PathBuf::from(format!("/tmp/ironbar-todo-{user}.sock"))
    }
}

pub fn get_cursor_pos() -> Option<(i32, i32)> {
    let output = std::process::Command::new("hyprctl")
        .arg("cursorpos")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split(',').collect();
    if parts.len() == 2 {
        let x = parts[0].trim().parse::<i32>().ok()?;
        let y = parts[1].trim().parse::<i32>().ok()?;
        Some((x, y))
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcCommand {
    Toggle(Option<(i32, i32)>),
    Show(Option<(i32, i32)>),
    Hide,
    Refresh,
    Quit,
}

impl IpcCommand {
    pub fn from_str(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.starts_with("toggle") {
            let rest = trimmed.strip_prefix("toggle").unwrap_or("");
            let coords = parse_coords(rest.trim_start_matches(':'));
            Some(Self::Toggle(coords))
        } else if trimmed.starts_with("show") || trimmed.starts_with("open") {
            let rest = if trimmed.starts_with("show") {
                trimmed.strip_prefix("show").unwrap_or("")
            } else {
                trimmed.strip_prefix("open").unwrap_or("")
            };
            let coords = parse_coords(rest.trim_start_matches(':'));
            Some(Self::Show(coords))
        } else if trimmed == "hide" || trimmed == "close" {
            Some(Self::Hide)
        } else if trimmed == "refresh" {
            Some(Self::Refresh)
        } else if trimmed == "quit" || trimmed == "exit" {
            Some(Self::Quit)
        } else {
            None
        }
    }
}

fn parse_coords(s: &str) -> Option<(i32, i32)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 2 {
        let x = parts[0].trim().parse::<i32>().ok()?;
        let y = parts[1].trim().parse::<i32>().ok()?;
        Some((x, y))
    } else {
        None
    }
}

/// Tries to send a command to an existing running daemon instance.
/// Returns Ok(true) if sent successfully, Ok(false) if no instance is listening.
pub fn try_send_command(cmd: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let socket_path = get_socket_path();
    if !socket_path.exists() {
        return Ok(false);
    }

    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            writeln!(stream, "{cmd}")?;
            stream.flush()?;
            Ok(true)
        }
        Err(_) => {
            // Stale socket file
            let _ = std::fs::remove_file(&socket_path);
            Ok(false)
        }
    }
}

/// Starts the IPC server thread and returns a Receiver for incoming IPC commands.
pub fn start_ipc_server() -> Result<Receiver<IpcCommand>, Box<dyn std::error::Error>> {
    let socket_path = get_socket_path();
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    let listener = UnixListener::bind(&socket_path)?;
    let (tx, rx): (Sender<IpcCommand>, Receiver<IpcCommand>) = channel();

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    if let Ok(_) = reader.read_line(&mut line) {
                        if let Some(cmd) = IpcCommand::from_str(&line) {
                            if tx.send(cmd).is_err() {
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting IPC connection: {e}");
                }
            }
        }
        let _ = std::fs::remove_file(socket_path);
    });

    Ok(rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_command_parsing() {
        assert_eq!(IpcCommand::from_str("toggle"), Some(IpcCommand::Toggle(None)));
        assert_eq!(
            IpcCommand::from_str("toggle:500,20"),
            Some(IpcCommand::Toggle(Some((500, 20))))
        );
        assert_eq!(
            IpcCommand::from_str("show:100,50"),
            Some(IpcCommand::Show(Some((100, 50))))
        );
        assert_eq!(IpcCommand::from_str("hide"), Some(IpcCommand::Hide));
        assert_eq!(IpcCommand::from_str("refresh"), Some(IpcCommand::Refresh));
        assert_eq!(IpcCommand::from_str("quit"), Some(IpcCommand::Quit));
    }
}
