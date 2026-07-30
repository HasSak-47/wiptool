use std::path::PathBuf;

use super::version::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Location {
    Local(PathBuf),
    URL(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Project {
    pub version: Option<Version>,
    pub edition: Version,

    pub location: Option<Location>,
    pub name: String,
    pub description: String,
    pub subprojects: Vec<Project>,
    pub todo: Vec<Task>,
    pub done: Vec<Task>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub priority: f64,
    pub difficulty: f64,
}

impl Project {
    /* priority */
    fn tasks_get_priority(tasks: &Vec<Task>) -> f64 {
        tasks.iter().fold(0., |x, task| x + task.priority)
    }

    pub fn get_priority(&self) -> f64 {
        let mut total = Self::tasks_get_priority(&self.todo);

        for project in &self.subprojects {
            total += project.get_priority();
        }

        return total;
    }

    /* difficulty */
    fn tasks_get_difficulty(tasks: &Vec<Task>) -> f64 {
        tasks.iter().fold(0., |x, task| x + task.difficulty)
    }

    pub fn get_done_difficulty(&self) -> f64 {
        let mut done = Self::tasks_get_difficulty(&self.done);

        for project in &self.subprojects {
            done += project.get_done_difficulty();
        }

        return done;
    }

    pub fn get_todo_difficulty(&self) -> f64 {
        let mut todo = Self::tasks_get_difficulty(&self.todo);

        for project in &self.subprojects {
            todo += project.get_todo_difficulty();
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

        for project in &self.todo {
            total += project.difficulty;
        }

        for project in &self.subprojects {
            total += project.get_difficulty();
        }

        return total;
    }
}
