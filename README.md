# ironbar-todo

A sleek, lightweight GTK4 layer-shell floating module and popup for [Ironbar](https://github.com/JakeStanger/ironbar), integrating with your Rust `todo` CLI.

## Features

- **Ironbar Integration**: Appears as a clickable `"todo"` button in Ironbar that opens a floating panel relative to the bar.
- **Task Grouping**: Separates tasks into **Remaining** and **Completed** sections with live counts.
- **Task Type Toggles**: Filter tasks by `basic`, `daily`, or both simultaneously with quick toggle buttons.
- **Interactive Controls**:
  - Complete and uncomplete tasks directly via checkboxes.
  - Delete individual tasks with a single click (`✕`).
  - Clear all tasks (`Clear All`) via header control.
- **Add Tasks on the Fly**: Bottom input field with task name input, task type dropdown (`Basic` / `Daily`), and `Add` button (or press `Enter`).
- **Instant IPC & Sub-Millisecond Toggle**: Background daemon with Unix domain socket for instant toggling on click.
- **Ironbar Theming**: Automatically adheres to your system's Ironbar styling (`~/.config/ironbar/style.css`) and defaults to a matching dark minimalist theme.
- **Dismissable**: Close with `Esc`, the close button (`✕`), or by clicking the Ironbar button again to toggle.

## Requirements

- Linux with a Wayland compositor supporting `wlr-layer-shell` (e.g. Hyprland, Sway).
- GTK4 & `gtk4-layer-shell` (`pkg-config gtk4 gtk4-layer-shell-0`).
- The `todo` CLI installed in your `PATH`.
- Rust 2024 edition.

## Installation

```bash
cargo install --path .
```

This installs the binary to `~/.cargo/bin/ironbar-todo`. Ensure `~/.cargo/bin` is in your `$PATH`.

## Ironbar Configuration

Add the following to your Ironbar configuration file (`~/.config/ironbar/config.toml`):

```toml
[[end]]
type = "custom"
class = "todo-btn"
bar = [
  { type = "button", name = "todo-btn", label = "todo", on_click = "!ironbar-todo toggle" }
]
```

Reload Ironbar:

```bash
ironbar reload
```

## CLI Usage

```bash
ironbar-todo [COMMAND]

Commands:
  toggle          Toggle visibility of the floating todo panel (default)
  show            Show the floating todo panel
  hide            Hide the floating todo panel
  daemon          Run daemon in background waiting for toggle/show events
  install-config  Print Ironbar configuration snippet
```

## License

MIT License. See [LICENSE](LICENSE) for details.
