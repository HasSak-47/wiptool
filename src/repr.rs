use std::{collections::HashMap, fmt, path::PathBuf, str::FromStr};

use super::version::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Location {
    Local(PathBuf),
    URL(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TodoStatus {
    #[default]
    Idea,
    Planned,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum InProgressStatus {
    #[default]
    Prototype,
    Active,
    Halted,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompleteStatus {
    #[default]
    Done,
    Stable,
    Experimental,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancelledStatus {
    #[default]
    Abandoned,
    Replaced,
    Invalidated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Todo(TodoStatus),
    InProgress(InProgressStatus),
    Complete(CompleteStatus),
    Cancelled(CancelledStatus),
}

impl Default for Status {
    fn default() -> Self {
        return Status::Todo(TodoStatus::Idea);
    }
}

impl fmt::Display for TodoStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            TodoStatus::Idea => "idea",
            TodoStatus::Planned => "planned",
        };
        write!(f, "{value}")
    }
}

impl fmt::Display for InProgressStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            InProgressStatus::Prototype => "prototype",
            InProgressStatus::Active => "active",
            InProgressStatus::Halted => "halted",
        };
        write!(f, "{value}")
    }
}

impl fmt::Display for CompleteStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            CompleteStatus::Done => "done",
            CompleteStatus::Stable => "stable",
            CompleteStatus::Experimental => "experimental",
        };
        write!(f, "{value}")
    }
}

impl fmt::Display for CancelledStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            CancelledStatus::Abandoned => "abandoned",
            CancelledStatus::Replaced => "replaced",
            CancelledStatus::Invalidated => "invalidated",
        };
        write!(f, "{value}")
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Todo(status) => write!(f, "todo.{status}"),
            Status::InProgress(status) => write!(f, "in-progress.{status}"),
            Status::Complete(status) => write!(f, "complete.{status}"),
            Status::Cancelled(status) => write!(f, "cancelled.{status}"),
        }
    }
}

impl FromStr for Status {
    type Err = String;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let normalized = input.trim().to_ascii_lowercase().replace('_', "-");
        let status = match normalized.as_str() {
            "todo" => Status::Todo(TodoStatus::default()),
            "todo.idea" => Status::Todo(TodoStatus::Idea),
            "todo.planned" => Status::Todo(TodoStatus::Planned),

            "in-progress" => Status::InProgress(InProgressStatus::default()),
            "in-progress.prototype" => Status::InProgress(InProgressStatus::Prototype),
            "in-progress.active" => Status::InProgress(InProgressStatus::Active),
            "in-progress.halted" => Status::InProgress(InProgressStatus::Halted),

            "complete" => Status::Complete(CompleteStatus::default()),
            "complete.done" => Status::Complete(CompleteStatus::Done),
            "complete.stable" => Status::Complete(CompleteStatus::Stable),
            "complete.experimental" => Status::Complete(CompleteStatus::Experimental),

            "cancelled" => Status::Cancelled(CancelledStatus::default()),
            "cancelled.abandoned" => Status::Cancelled(CancelledStatus::Abandoned),
            "cancelled.replaced" => Status::Cancelled(CancelledStatus::Replaced),
            "cancelled.invalidated" => Status::Cancelled(CancelledStatus::Invalidated),
            _ => {
                return Err(format!(
                    "invalid status '{input}', expected one of: {}",
                    Status::accepted_values().join(", ")
                ))
            }
        };

        Ok(status)
    }
}

impl Status {
    pub fn accepted_values() -> &'static [&'static str] {
        &[
            // todo
            "todo.idea",
            "todo",
            "todo.planned",
            // in progress
            "in-progress",
            "in-progress.prototype",
            "in-progress.active",
            "in-progress.halted",
            // complete
            "complete",
            "complete.done",
            "complete.stable",
            "complete.experimental",
            // cancelled
            "cancelled",
            "cancelled.abandoned",
            "cancelled.replaced",
            "cancelled.invalidated",
        ]
    }

    pub fn aproximate(&self, other: &Status) -> bool {
        match (self, other) {
            (Status::Todo(_), Status::Todo(_)) => true,
            (Status::InProgress(_), Status::InProgress(_)) => true,
            (Status::Complete(_), Status::Complete(_)) => true,
            (Status::Cancelled(_), Status::Cancelled(_)) => true,

            _ => false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Project {
    pub name: String,
    pub edition: Version,

    pub current_milestone: String,
    pub miliestones: HashMap<String, Status>,

    pub kinds: Vec<String>,
    pub description: String,

    pub location: Option<Location>,
    pub subprojects: HashMap<String, Project>,

    pub milestones: HashMap<String, Milestone>,
}

#[derive(Debug, Clone, Default)]
pub struct Milestone {
    pub status: Status,
    pub version: Option<Version>,

    pub tasks: HashMap<String, Task>,
}

#[derive(Debug, Default, Clone)]
pub struct Task {
    pub todo: bool,
    pub priority: f64,
    pub difficulty: f64,
}

impl Project {
    /* priority */
    fn tasks_get_priority(tasks: Vec<(&String, &Task)>) -> f64 {
        tasks.iter().fold(0., |x, task| x + task.1.priority)
    }

    pub fn get_todo_tasks(&self) -> Vec<(&String, &Task)> {
        let curr = self.milestones.get(&self.current_milestone).unwrap();

        curr.tasks.iter().filter(|a| a.1.todo).collect()
    }

    pub fn get_done_tasks(&self) -> Vec<(&String, &Task)> {
        let curr = self.milestones.get(&self.current_milestone).unwrap();

        curr.tasks.iter().filter(|a| !a.1.todo).collect()
    }

    pub fn get_priority(&self) -> f64 {
        let mut total = Self::tasks_get_priority(self.get_todo_tasks());

        for project in &self.subprojects {
            total += project.1.get_priority();
        }

        return total;
    }

    /* difficulty */
    fn tasks_get_difficulty(tasks: Vec<(&String, &Task)>) -> f64 {
        tasks.iter().fold(0., |x, task| x + task.1.difficulty)
    }

    pub fn get_done_difficulty(&self) -> f64 {
        let mut done = Self::tasks_get_difficulty(self.get_done_tasks());

        for project in &self.subprojects {
            done += project.1.get_done_difficulty();
        }

        return done;
    }

    pub fn get_todo_difficulty(&self) -> f64 {
        let mut todo = Self::tasks_get_difficulty(self.get_todo_tasks());

        for project in &self.subprojects {
            todo += project.1.get_todo_difficulty();
        }

        return todo;
    }

    pub fn get_difficulty_completion(&self) -> f64 {
        let todo = self.get_todo_difficulty();
        let done = self.get_done_difficulty();

        let total = todo + done;
        return if total != 0. { todo / total } else { 1. };
    }

    pub fn get_difficulty(&self) -> f64 {
        let mut total = 0.;

        for project in self.get_todo_tasks() {
            total += project.1.difficulty;
        }

        for project in &self.subprojects {
            total += project.1.get_difficulty();
        }

        return total;
    }
}

#[cfg(test)]
mod tests {
    use super::{InProgressStatus, Status};
    use std::str::FromStr;

    #[test]
    fn status_parses_from_canonical_string() {
        let status = Status::from_str("in-progress.active").expect("valid status");

        assert!(matches!(
            status,
            Status::InProgress(InProgressStatus::Active)
        ));
    }

    #[test]
    fn status_parser_accepts_underscore_bucket_alias() {
        let status = Status::from_str("in_progress.active").expect("valid status");

        assert_eq!(status.to_string(), "in-progress.active");
    }

    #[test]
    fn status_display_uses_canonical_string() {
        let status = Status::from_str("complete.experimental").expect("valid status");

        assert_eq!(status.to_string(), "complete.experimental");
    }

    #[test]
    fn status_parser_rejects_unknown_status() {
        let error = Status::from_str("started").expect_err("invalid status");

        assert!(error.contains("todo.idea"), "unexpected error: {error}");
    }
}
