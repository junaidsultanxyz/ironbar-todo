use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gdk::Key;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Button, CheckButton, DropDown, Entry, EventControllerKey,
    Label, Orientation, PolicyType, ScrolledWindow, StringList, ToggleButton,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::todo_cli::{TaskItem, TaskType, TodoClient};

#[derive(Clone)]
pub struct AppState {
    pub client: TodoClient,
    pub show_basic: bool,
    pub show_daily: bool,
}

pub struct TodoWidget {
    pub window: ApplicationWindow,
    state: Rc<RefCell<AppState>>,
    remaining_box: Box,
    completed_box: Box,
    remaining_header: Label,
    completed_header: Label,
    basic_filter_btn: ToggleButton,
    daily_filter_btn: ToggleButton,
}

impl TodoWidget {
    pub fn build(app: &Application) -> Rc<Self> {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("ironbar-todo")
            .default_width(380)
            .default_height(480)
            .css_classes(["todo-popup"])
            .build();

        // Configure Layer Shell for Wayland/Hyprland
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_namespace(Some("ironbar-todo"));
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 28);
        window.set_margin(Edge::Right, 12);
        window.set_keyboard_mode(KeyboardMode::OnDemand);

        let state = Rc::new(RefCell::new(AppState {
            client: TodoClient::new(),
            show_basic: true,
            show_daily: true,
        }));

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.add_css_class("popup-container");

        // Top Header Bar: Title, Filters, Clear, Close
        let header_bar = Box::new(Orientation::Horizontal, 6);
        header_bar.add_css_class("header-bar");

        let title_label = Label::new(Some("TODO"));
        title_label.add_css_class("title-label");
        header_bar.append(&title_label);

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header_bar.append(&spacer);

        // Filter toggles: Basic, Daily
        let basic_btn = ToggleButton::with_label("Basic");
        basic_btn.set_active(true);
        basic_btn.add_css_class("filter-btn");

        let daily_btn = ToggleButton::with_label("Daily");
        daily_btn.set_active(true);
        daily_btn.add_css_class("filter-btn");

        header_bar.append(&basic_btn);
        header_bar.append(&daily_btn);

        // Clear All tasks button (todo clear)
        let clear_btn = Button::with_label("Clear All");
        clear_btn.add_css_class("clear-btn");
        clear_btn.set_tooltip_text(Some("Delete all tasks (todo clear)"));
        header_bar.append(&clear_btn);

        // Close button
        let close_btn = Button::with_label("✕");
        close_btn.add_css_class("close-btn");
        close_btn.set_tooltip_text(Some("Close popup (Esc)"));
        header_bar.append(&close_btn);

        main_box.append(&header_bar);

        // Scrolled Window for task groups
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .vexpand(true)
            .hexpand(true)
            .build();

        let tasks_container = Box::new(Orientation::Vertical, 4);

        // Remaining Tasks Group
        let remaining_header = Label::new(Some("REMAINING (0)"));
        remaining_header.add_css_class("section-title");
        remaining_header.set_xalign(0.0);
        tasks_container.append(&remaining_header);

        let remaining_box = Box::new(Orientation::Vertical, 2);
        remaining_box.add_css_class("task-list");
        tasks_container.append(&remaining_box);

        // Completed Tasks Group
        let completed_header = Label::new(Some("COMPLETED (0)"));
        completed_header.add_css_class("section-title");
        completed_header.set_xalign(0.0);
        tasks_container.append(&completed_header);

        let completed_box = Box::new(Orientation::Vertical, 2);
        completed_box.add_css_class("task-list");
        tasks_container.append(&completed_box);

        scrolled.set_child(Some(&tasks_container));
        main_box.append(&scrolled);

        // Bottom Input Area: Text Entry + Dropdown + Add Button
        let input_box = Box::new(Orientation::Horizontal, 6);
        input_box.add_css_class("input-container");

        let entry = Entry::builder()
            .placeholder_text("Add a new task...")
            .hexpand(true)
            .css_classes(["task-entry"])
            .build();

        let dropdown_model = StringList::new(&["Basic", "Daily"]);
        let dropdown = DropDown::builder()
            .model(&dropdown_model)
            .selected(0)
            .css_classes(["type-dropdown"])
            .build();

        let add_btn = Button::builder()
            .label("Add")
            .css_classes(["add-btn"])
            .build();

        input_box.append(&entry);
        input_box.append(&dropdown);
        input_box.append(&add_btn);
        main_box.append(&input_box);

        window.set_child(Some(&main_box));

        // Close on Escape key
        let key_controller = EventControllerKey::new();
        let win_clone = window.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == Key::Escape {
                win_clone.set_visible(false);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        window.add_controller(key_controller);

        let win_close = window.clone();
        close_btn.connect_clicked(move |_| {
            win_close.set_visible(false);
        });

        let widget = Rc::new(Self {
            window,
            state,
            remaining_box,
            completed_box,
            remaining_header,
            completed_header,
            basic_filter_btn: basic_btn,
            daily_filter_btn: daily_btn,
        });

        // Add task handlers
        let w_clone = Rc::clone(&widget);
        let entry_clone = entry.clone();
        let dropdown_clone = dropdown.clone();
        let on_add = move || {
            let text = entry_clone.text().trim().to_string();
            if !text.is_empty() {
                let is_daily = dropdown_clone.selected() == 1;
                let client = { w_clone.state.borrow().client.clone() };
                if let Err(e) = client.add(&text, is_daily) {
                    eprintln!("Failed to add task: {e}");
                } else {
                    entry_clone.set_text("");
                    w_clone.refresh();
                }
            }
        };

        let on_add_btn = on_add.clone();
        add_btn.connect_clicked(move |_| {
            on_add_btn();
        });

        let on_add_entry = on_add.clone();
        entry.connect_activate(move |_| {
            on_add_entry();
        });

        // Filter toggle handlers
        let w_basic = Rc::clone(&widget);
        widget.basic_filter_btn.connect_toggled(move |btn| {
            w_basic.state.borrow_mut().show_basic = btn.is_active();
            w_basic.refresh();
        });

        let w_daily = Rc::clone(&widget);
        widget.daily_filter_btn.connect_toggled(move |btn| {
            w_daily.state.borrow_mut().show_daily = btn.is_active();
            w_daily.refresh();
        });

        // Clear button handler
        let w_clear = Rc::clone(&widget);
        clear_btn.connect_clicked(move |_| {
            let client = { w_clear.state.borrow().client.clone() };
            if let Err(e) = client.clear() {
                eprintln!("Failed to clear tasks: {e}");
            } else {
                w_clear.refresh();
            }
        });

        // Initial task load
        widget.refresh();

        widget
    }

    pub fn toggle_visibility(&self) {
        let is_vis = self.window.is_visible();
        if is_vis {
            self.window.set_visible(false);
        } else {
            self.refresh();
            self.window.set_visible(true);
            self.window.present();
        }
    }

    pub fn show_window(&self) {
        self.refresh();
        self.window.set_visible(true);
        self.window.present();
    }

    pub fn hide_window(&self) {
        self.window.set_visible(false);
    }

    pub fn refresh(&self) {
        // Clear current boxes
        while let Some(child) = self.remaining_box.first_child() {
            self.remaining_box.remove(&child);
        }
        while let Some(child) = self.completed_box.first_child() {
            self.completed_box.remove(&child);
        }

        let (show_basic, show_daily, tasks) = {
            let state = self.state.borrow();
            let tasks = match state.client.list_all() {
                Ok(t) => t,
                Err(err) => {
                    eprintln!("Error fetching tasks: {err}");
                    Vec::new()
                }
            };
            (state.show_basic, state.show_daily, tasks)
        };

        let mut remaining_count = 0;
        let mut completed_count = 0;

        for task in tasks {
            // Apply task type filter
            let passes_type = match task.task_type {
                TaskType::Basic => show_basic,
                TaskType::Daily => show_daily,
            };

            if !passes_type {
                continue;
            }

            if task.is_completed {
                completed_count += 1;
                let row = self.create_task_row(&task);
                self.completed_box.append(&row);
            } else {
                remaining_count += 1;
                let row = self.create_task_row(&task);
                self.remaining_box.append(&row);
            }
        }

        self.remaining_header
            .set_text(&format!("REMAINING ({remaining_count})"));
        self.completed_header
            .set_text(&format!("COMPLETED ({completed_count})"));

        if remaining_count == 0 {
            let empty_lbl = Label::new(Some("No remaining tasks"));
            empty_lbl.add_css_class("empty-label");
            empty_lbl.set_xalign(0.0);
            self.remaining_box.append(&empty_lbl);
        }

        if completed_count == 0 {
            let empty_lbl = Label::new(Some("No completed tasks"));
            empty_lbl.add_css_class("empty-label");
            empty_lbl.set_xalign(0.0);
            self.completed_box.append(&empty_lbl);
        }
    }

    fn create_task_row(&self, task: &TaskItem) -> Box {
        Self::create_row_static(
            task,
            &self.state,
            &self.remaining_box,
            &self.completed_box,
            &self.remaining_header,
            &self.completed_header,
        )
    }

    fn refresh_with_state(
        state_rc: &Rc<RefCell<AppState>>,
        remaining_box: &Box,
        completed_box: &Box,
        remaining_header: &Label,
        completed_header: &Label,
    ) {
        while let Some(child) = remaining_box.first_child() {
            remaining_box.remove(&child);
        }
        while let Some(child) = completed_box.first_child() {
            completed_box.remove(&child);
        }

        let (show_basic, show_daily, tasks) = {
            let state = state_rc.borrow();
            let tasks = match state.client.list_all() {
                Ok(t) => t,
                Err(err) => {
                    eprintln!("Error fetching tasks: {err}");
                    Vec::new()
                }
            };
            (state.show_basic, state.show_daily, tasks)
        };

        let mut remaining_count = 0;
        let mut completed_count = 0;

        for task in tasks {
            let passes_type = match task.task_type {
                TaskType::Basic => show_basic,
                TaskType::Daily => show_daily,
            };

            if !passes_type {
                continue;
            }

            if task.is_completed {
                completed_count += 1;
                let row = Self::create_row_static(
                    &task,
                    state_rc,
                    remaining_box,
                    completed_box,
                    remaining_header,
                    completed_header,
                );
                completed_box.append(&row);
            } else {
                remaining_count += 1;
                let row = Self::create_row_static(
                    &task,
                    state_rc,
                    remaining_box,
                    completed_box,
                    remaining_header,
                    completed_header,
                );
                remaining_box.append(&row);
            }
        }

        remaining_header.set_text(&format!("REMAINING ({remaining_count})"));
        completed_header.set_text(&format!("COMPLETED ({completed_count})"));

        if remaining_count == 0 {
            let empty_lbl = Label::new(Some("No remaining tasks"));
            empty_lbl.add_css_class("empty-label");
            empty_lbl.set_xalign(0.0);
            remaining_box.append(&empty_lbl);
        }

        if completed_count == 0 {
            let empty_lbl = Label::new(Some("No completed tasks"));
            empty_lbl.add_css_class("empty-label");
            empty_lbl.set_xalign(0.0);
            completed_box.append(&empty_lbl);
        }
    }

    fn create_row_static(
        task: &TaskItem,
        state_rc: &Rc<RefCell<AppState>>,
        remaining_box: &Box,
        completed_box: &Box,
        remaining_header: &Label,
        completed_header: &Label,
    ) -> Box {
        let row = Box::new(Orientation::Horizontal, 6);
        row.add_css_class("task-row");
        if task.is_completed {
            row.add_css_class("completed");
        }

        let check = CheckButton::new();
        check.set_active(task.is_completed);

        let task_id = task.id.clone();
        let is_completed = task.is_completed;
        let state_tgl = Rc::clone(state_rc);
        let rem_box = remaining_box.clone();
        let com_box = completed_box.clone();
        let rem_hdr = remaining_header.clone();
        let com_hdr = completed_header.clone();

        check.connect_toggled(move |btn| {
            let new_active = btn.is_active();
            if new_active != is_completed {
                let client = { state_tgl.borrow().client.clone() };
                let res = if new_active {
                    client.done(&task_id)
                } else {
                    client.undone(&task_id)
                };

                if let Err(e) = res {
                    eprintln!("Failed to toggle task status: {e}");
                }
                Self::refresh_with_state(&state_tgl, &rem_box, &com_box, &rem_hdr, &com_hdr);
            }
        });
        row.append(&check);

        let title_label = Label::new(Some(&task.title));
        title_label.add_css_class("task-title");
        title_label.set_xalign(0.0);
        title_label.set_hexpand(true);
        title_label.set_wrap(true);
        row.append(&title_label);

        let badge_class = match task.task_type {
            TaskType::Basic => "badge-basic",
            TaskType::Daily => "badge-daily",
        };
        let badge_label = Label::new(Some(&task.task_type.to_string()));
        badge_label.add_css_class(badge_class);
        row.append(&badge_label);

        let delete_btn = Button::with_label("✕");
        delete_btn.add_css_class("delete-btn");
        delete_btn.set_tooltip_text(Some("Delete task (todo delete)"));

        let task_id_del = task.id.clone();
        let state_del = Rc::clone(state_rc);
        let rem_box_del = remaining_box.clone();
        let com_box_del = completed_box.clone();
        let rem_hdr_del = remaining_header.clone();
        let com_hdr_del = completed_header.clone();

        delete_btn.connect_clicked(move |_| {
            let client = { state_del.borrow().client.clone() };
            if let Err(e) = client.delete(&task_id_del) {
                eprintln!("Failed to delete task: {e}");
            } else {
                Self::refresh_with_state(
                    &state_del,
                    &rem_box_del,
                    &com_box_del,
                    &rem_hdr_del,
                    &com_hdr_del,
                );
            }
        });
        row.append(&delete_btn);

        row
    }
}
