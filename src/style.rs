use gtk4::CssProvider;
use gtk4::gdk::Display;
use std::fs;
use std::path::PathBuf;

pub fn load_css() {
    let provider = CssProvider::new();

    // Default CSS matching Ironbar's theme
    let default_css = r#"
:root {
    --color-dark-primary: #1c1c1c;
    --color-dark-secondary: #2d2d2d;
    --color-white: #ffffff;
    --color-active: #6699cc;
    --color-urgent: #8f0a0a;
    --color-muted: #888888;
}

* {
    border-radius: 0;
    border: none;
    box-shadow: none;
    background-image: none;
    font-family: monospace;
}

window.todo-popup {
    background-color: transparent;
}

.popup-container {
    background-color: var(--color-dark-primary);
    border: 1px solid var(--color-dark-secondary);
    padding: 10px;
    min-width: 360px;
    max-width: 420px;
    min-height: 400px;
    max-height: 580px;
}

.header-bar {
    margin-bottom: 8px;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--color-dark-secondary);
}

.title-label {
    font-weight: bold;
    font-size: 1.1em;
    color: var(--color-active);
}

.filter-btn {
    padding: 3px 8px;
    margin-right: 4px;
    background-color: var(--color-dark-secondary);
    color: var(--color-muted);
    font-size: 0.85em;
}

.filter-btn:checked {
    background-color: var(--color-active);
    color: var(--color-white);
    font-weight: bold;
}

.clear-btn {
    padding: 3px 8px;
    background-color: var(--color-dark-secondary);
    color: var(--color-urgent);
    font-size: 0.85em;
}

.clear-btn:hover {
    background-color: var(--color-urgent);
    color: var(--color-white);
}

.close-btn {
    padding: 3px 8px;
    background-color: transparent;
    color: var(--color-muted);
}

.close-btn:hover {
    color: var(--color-white);
}

.section-title {
    font-weight: bold;
    font-size: 0.8em;
    color: var(--color-muted);
    margin-top: 6px;
    margin-bottom: 4px;
    padding: 2px 4px;
    background-color: #222222;
}

.task-list {
    margin-bottom: 6px;
}

.task-row {
    padding: 4px 6px;
    margin-bottom: 2px;
    background-color: #232323;
}

.task-row:hover {
    background-color: var(--color-dark-secondary);
}

.task-row.completed {
    opacity: 0.65;
}

.task-row.completed label.task-title {
    text-decoration: line-through;
    color: var(--color-muted);
}

.task-title {
    color: var(--color-white);
    font-size: 0.9em;
}

.badge-basic {
    font-size: 0.75em;
    color: #a0a0a0;
    background-color: #333333;
    padding: 1px 4px;
    margin-left: 6px;
}

.badge-daily {
    font-size: 0.75em;
    color: #ffffff;
    background-color: #446688;
    padding: 1px 4px;
    margin-left: 6px;
}

.delete-btn {
    color: var(--color-muted);
    background-color: transparent;
    padding: 2px 6px;
    font-size: 0.85em;
}

.delete-btn:hover {
    color: var(--color-urgent);
    background-color: rgba(143, 10, 10, 0.2);
}

.empty-label {
    color: var(--color-muted);
    font-style: italic;
    font-size: 0.85em;
    padding: 8px 4px;
}

.input-container {
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid var(--color-dark-secondary);
}

entry.task-entry {
    background-color: var(--color-dark-secondary);
    color: var(--color-white);
    padding: 5px 8px;
    font-size: 0.9em;
    caret-color: var(--color-active);
}

entry.task-entry:focus {
    border: 1px solid var(--color-active);
}

dropdown.type-dropdown {
    background-color: var(--color-dark-secondary);
    color: var(--color-white);
    padding: 3px 6px;
    font-size: 0.85em;
}

.add-btn {
    background-color: var(--color-active);
    color: var(--color-white);
    font-weight: bold;
    padding: 5px 12px;
    font-size: 0.9em;
}

.add-btn:hover {
    background-color: #5588bb;
}

checkbutton check {
    border: 1px solid var(--color-muted);
    background-color: var(--color-dark-secondary);
    min-width: 14px;
    min-height: 14px;
}

checkbutton:checked check {
    background-color: var(--color-active);
    border-color: var(--color-active);
}
"#;

    let mut css_data = default_css.to_string();

    // If ~/.config/ironbar/style.css exists, append it so user overrides/colors are respected!
    if let Some(home) = std::env::var_os("HOME") {
        let ironbar_css_path = PathBuf::from(home).join(".config/ironbar/style.css");
        if ironbar_css_path.exists()
            && let Ok(custom_css) = fs::read_to_string(&ironbar_css_path)
        {
            css_data.push_str("\n/* User Ironbar CSS */\n");
            css_data.push_str(&custom_css);
        }
    }

    provider.load_from_data(&css_data);

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
