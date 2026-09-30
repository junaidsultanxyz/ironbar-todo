use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

pub fn get_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("ironbar-todo.sock")
    } else {
        let user = std::env::var("USER").unwrap_or_else(|_| "current".to_string());
        PathBuf::from(format!("/tmp/ironbar-todo-{user}.sock"))
    }
}

pub enum IpcCommand {
    Toggle,
    Show,
    Hide,
    Refresh,
    Quit,
}

impl IpcCommand {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "toggle" => Some(Self::Toggle),
            "show" | "open" => Some(Self::Show),
            "hide" | "close" => Some(Self::Hide),
            "refresh" => Some(Self::Refresh),
            "quit" | "exit" => Some(Self::Quit),
            _ => None,
        }
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
                    match reader.read_line(&mut line) {
                        Ok(_) => {
                            if let Some(cmd) = IpcCommand::from_string(&line)
                                && tx.send(cmd).is_err()
                            {
                                break;
                            }
                        }
                        Err(_) => todo!(),
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
