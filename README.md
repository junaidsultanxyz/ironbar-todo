# ironbar-todo

A sleek, lightweight GTK4 layer-shell floating module and popup for [Ironbar](https://github.com/JakeStanger/ironbar), integrating with your Rust `todo` CLI.

<img width="407" height="529" alt="image" src="https://github.com/user-attachments/assets/e0a337fd-5df5-452d-9e0e-c75293ea7638" />


## Requirements

- Linux with a Wayland compositor supporting `wlr-layer-shell` (e.g. Hyprland, Sway).
- GTK4 & `gtk4-layer-shell` (`pkg-config gtk4 gtk4-layer-shell-0`).
- The `todo` CLI installed in your `PATH`.
- Rust 2024 edition.

## Installation

```bash
git clone https://github.com/junaidsultanxyz/ironbar-todo
cd ironbar-todo
cargo install --path .
```

This installs the binary to `~/.cargo/bin/ironbar-todo`. Ensure `~/.cargo/bin` is in your `$PATH`.

## Ironbar Configuration

Add the following to your Ironbar configuration file (`~/.config/ironbar/config.toml`):

```toml
[[start]]
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
