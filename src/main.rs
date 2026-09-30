pub mod ipc;
pub mod style;
pub mod ui;

use clap::{Parser, Subcommand};
use gtk4::Application;
use gtk4::glib::{self, ControlFlow};
use gtk4::prelude::*;
use std::rc::Rc;
use std::time::Duration;

use crate::ipc::{IpcCommand, get_cursor_pos, start_ipc_server, try_send_command};
use crate::style::load_css;
use crate::ui::TodoWidget;

#[derive(Parser, Debug)]
#[command(name = "ironbar-todo")]
#[command(author, version, about = "Ironbar todo module and floating widget", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Toggle visibility of the floating todo panel (default)
    Toggle,
    /// Show the floating todo panel
    Show,
    /// Hide the floating todo panel
    Hide,
    /// Run as a daemon waiting for toggle/show events
    Daemon,
    /// Print Ironbar configuration snippet for this module
    InstallConfig,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Toggle);

    match command {
        Commands::InstallConfig => {
            println!(
                r#"Add this to your Ironbar config (usually ~/.config/ironbar/config.toml):

[[end]]
type = "custom"
class = "todo-module"
bar = [
  {{ type = "button", name = "todo-btn", label = "todo", on_click = "!ironbar-todo toggle" }}
]
"#
            );
            return Ok(());
        }
        Commands::Hide => {
            if try_send_command("hide")? {
                return Ok(());
            }
            // If no instance running, nothing to hide
            return Ok(());
        }
        Commands::Toggle => {
            let coords = get_cursor_pos();
            let msg = if let Some((x, y)) = coords {
                format!("toggle:{x},{y}")
            } else {
                "toggle".to_string()
            };
            if try_send_command(&msg)? {
                return Ok(());
            }
        }
        Commands::Show => {
            let coords = get_cursor_pos();
            let msg = if let Some((x, y)) = coords {
                format!("show:{x},{y}")
            } else {
                "show".to_string()
            };
            if try_send_command(&msg)? {
                return Ok(());
            }
        }
        Commands::Daemon => {
            if try_send_command("refresh")? {
                println!("ironbar-todo daemon is already running.");
                return Ok(());
            }
        }
    }

    // Start GTK Application
    let app = Application::builder()
        .application_id("org.ironbar.todo")
        .build();

    let start_shown = !matches!(command, Commands::Daemon);

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(move |app| {
        // Prevent auto-exit when window is hidden
        let _hold_guard = app.hold();

        let initial_coords = get_cursor_pos();
        let widget = TodoWidget::build(app);
        if start_shown {
            widget.show_window(initial_coords);
        } else {
            widget.hide_window();
        }

        // Start IPC server to listen for CLI commands (toggle, show, hide, etc.)
        match start_ipc_server() {
            Ok(rx) => {
                let widget_clone: Rc<TodoWidget> = Rc::clone(&widget);
                let app_clone = app.clone();

                glib::timeout_add_local(Duration::from_millis(40), move || {
                    let _ = &_hold_guard;
                    while let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            IpcCommand::Toggle(coords) => widget_clone.toggle_visibility(coords),
                            IpcCommand::Show(coords) => widget_clone.show_window(coords),
                            IpcCommand::Hide => widget_clone.hide_window(),
                            IpcCommand::Refresh => widget_clone.refresh(),
                            IpcCommand::Quit => {
                                app_clone.quit();
                                return ControlFlow::Break;
                            }
                        }
                    }
                    ControlFlow::Continue
                });
            }
            Err(e) => {
                eprintln!("Warning: Failed to start IPC server: {e}");
            }
        }
    });

    app.run_with_args::<&str>(&[]);
    Ok(())
}
