use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    Basic,
    Daily,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::Basic => write!(f, "basic"),
            TaskType::Daily => write!(f, "daily"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    pub is_completed: bool,
    pub task_type: TaskType,
}

#[derive(Debug, Default, Clone)]
pub struct TodoClient;

impl TodoClient {
    pub fn new() -> Self {
        Self
    }

    /// Fetches all tasks using `todo list -a` and parses them into structured `TaskItem`s.
    pub fn list_all(&self) -> Result<Vec<TaskItem>, String> {
        let output = Command::new("todo")
            .arg("list")
            .arg("-a")
            .output()
            .map_err(|e| format!("Failed to execute `todo list -a`: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("`todo list -a` failed: {stderr}"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(Self::parse_list_output(&stdout))
    }

    /// Parses the stdout of `todo list` into `Vec<TaskItem>`.
    pub fn parse_list_output(output: &str) -> Vec<TaskItem> {
        let mut tasks = Vec::new();
        let mut current_type: Option<TaskType> = None;

        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with("-----Basic tasks-----") {
                current_type = Some(TaskType::Basic);
                continue;
            } else if trimmed.starts_with("-----Daily tasks-----") {
                current_type = Some(TaskType::Daily);
                continue;
            }

            // Expected format: "[false] <id> | <title>" or "[true] <id> | <title>"
            if let Some(task_type) = current_type
                && let Some(task) = Self::parse_task_line(trimmed, task_type)
            {
                tasks.push(task);
            }
        }

        tasks
    }

    fn parse_task_line(line: &str, task_type: TaskType) -> Option<TaskItem> {
        // Line format: "[<status>] <id> | <title>"
        if !line.starts_with('[') {
            return None;
        }

        let close_bracket = line.find(']')?;
        let status_str = &line[1..close_bracket];
        let is_completed = match status_str {
            "true" => true,
            "false" => false,
            _ => return None,
        };

        let rest = line[close_bracket + 1..].trim();
        let pipe_pos = rest.find('|')?;

        let id = rest[..pipe_pos].trim().to_string();
        let title = rest[pipe_pos + 1..].trim().to_string();

        if id.is_empty() {
            return None;
        }

        Some(TaskItem {
            id,
            title,
            is_completed,
            task_type,
        })
    }

    /// Adds a new task using `todo add [OPTIONS] <NAME>`.
    pub fn add(&self, name: &str, is_daily: bool) -> Result<(), String> {
        let mut cmd = Command::new("todo");
        cmd.arg("add");
        if is_daily {
            cmd.arg("-d");
        }
        cmd.arg(name);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run `todo add`: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("`todo add` failed: {stderr}"));
        }
        Ok(())
    }

    /// Marks a task as complete using `todo done <ID>`.
    pub fn done(&self, id: &str) -> Result<(), String> {
        let output = Command::new("todo")
            .arg("done")
            .arg(id)
            .output()
            .map_err(|e| format!("Failed to run `todo done`: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("`todo done` failed: {stderr}"));
        }
        Ok(())
    }

    /// Marks a task as incomplete using `todo undone <ID>`.
    pub fn undone(&self, id: &str) -> Result<(), String> {
        let output = Command::new("todo")
            .arg("undone")
            .arg(id)
            .output()
            .map_err(|e| format!("Failed to run `todo undone`: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("`todo undone` failed: {stderr}"));
        }
        Ok(())
    }

    /// Deletes a task by id using `todo delete <ID>`.
    pub fn delete(&self, id: &str) -> Result<(), String> {
        let output = Command::new("todo")
            .arg("delete")
            .arg(id)
            .output()
            .map_err(|e| format!("Failed to run `todo delete`: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("`todo delete` failed: {stderr}"));
        }
        Ok(())
    }

    /// Deletes all tasks using `todo clear`.
    pub fn clear(&self) -> Result<(), String> {
        let output = Command::new("todo")
            .arg("clear")
            .output()
            .map_err(|e| format!("Failed to run `todo clear`: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("`todo clear` failed: {stderr}"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list_output() {
        let sample = "\
-----Basic tasks-----
[false] whlrf_bB0WZLq60Hb9306 | add daily resets to todo app
[true] 999xyz | finished task

-----Daily tasks-----
[false] yoGtTjI1OH-01h5DORllq | client hunting
";

        let tasks = TodoClient::parse_list_output(sample);
        assert_eq!(tasks.len(), 3);

        assert_eq!(tasks[0].id, "whlrf_bB0WZLq60Hb9306");
        assert_eq!(tasks[0].title, "add daily resets to todo app");
        assert!(!tasks[0].is_completed);
        assert_eq!(tasks[0].task_type, TaskType::Basic);

        assert_eq!(tasks[1].id, "999xyz");
        assert_eq!(tasks[1].title, "finished task");
        assert!(tasks[1].is_completed);
        assert_eq!(tasks[1].task_type, TaskType::Basic);

        assert_eq!(tasks[2].id, "yoGtTjI1OH-01h5DORllq");
        assert_eq!(tasks[2].title, "client hunting");
        assert!(!tasks[2].is_completed);
        assert_eq!(tasks[2].task_type, TaskType::Daily);
    }

    #[test]
    fn test_parse_empty_list() {
        let sample = "\
-----Basic tasks-----

-----Daily tasks-----
";
        let tasks = TodoClient::parse_list_output(sample);
        assert!(tasks.is_empty());
    }
}
