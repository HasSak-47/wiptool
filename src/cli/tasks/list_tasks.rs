use std::env::current_dir;

use anyhow::Result;
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr::{Location, Milestone, Task},
};

/**
lists tasks for a project milestone
*/
#[derive(Parser, Clone)]
pub(crate) struct ListTasks {
    project: Option<String>,

    #[arg(short, long)]
    milestone: Option<String>,

    #[arg(long)]
    done: bool,

    #[arg(long)]
    todo: bool,
}

impl ListTasks {
    fn resolve_project_path(
        storage: &mut Box<dyn ProjectStorage>,
        project: &Option<String>,
    ) -> Result<ProjectDir> {
        if let Some(project) = project {
            return ProjectDir::parse(
                project.clone() + if project.ends_with("/") { "" } else { "/" },
            );
        }

        let cwd = current_dir()?;
        let mut found = None;
        for path in storage.get_projects_path()? {
            let project = storage.get_project(path.clone())?;
            let location = project.location.clone().unwrap();
            if let Location::Local(project_path) = &location {
                if cwd == *project_path {
                    found = Some(path);
                    break;
                }
            }
        }

        found.ok_or_else(|| anyhow::anyhow!("there is no project in cwd: {}", cwd.display()))
    }

    fn completion(milestone: &Milestone) -> f64 {
        let done = milestone
            .tasks
            .values()
            .filter(|task| !task.todo)
            .fold(0., |total, task| total + task.difficulty.unwrap_or(0.));
        let todo = milestone
            .tasks
            .values()
            .filter(|task| task.todo)
            .fold(0., |total, task| total + task.difficulty.unwrap_or(0.));
        let total = done + todo;

        if total == 0. {
            1.
        } else {
            done / total
        }
    }

    fn should_print_task(&self, task: &Task) -> bool {
        match (self.todo, self.done) {
            (true, false) => task.todo,
            (false, true) => !task.todo,
            _ => true,
        }
    }

    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let project_path = Self::resolve_project_path(storage, &self.project)?;
        let project = storage.get_project(project_path.clone())?;
        let milestone_name = self
            .milestone
            .clone()
            .unwrap_or_else(|| project.current_milestone.clone());
        let milestone = project
            .milestones
            .get(&milestone_name)
            .ok_or_else(|| anyhow::anyhow!("milestone not found: {milestone_name}"))?;

        println!(
            "{project_path}@{milestone_name}/ | {} | {:.1}%",
            milestone.status,
            Self::completion(milestone) * 100.
        );

        let mut tasks: Vec<_> = milestone
            .tasks
            .iter()
            .filter(|(_, task)| self.should_print_task(task))
            .collect();
        tasks.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (name, task) in tasks {
            let state = if task.todo { "todo" } else { "done" };
            let priority = task
                .priority
                .map(|value| format!("{value:.2}"))
                .unwrap_or_else(|| "—".to_string());
            let difficulty = task
                .difficulty
                .map(|value| format!("{value:.2}"))
                .unwrap_or_else(|| "—".to_string());
            let estimate = task
                .time_estimate
                .map(|value| format!("{value} min"))
                .unwrap_or_else(|| "—".to_string());
            println!(
                "{state:4} {name} | priority: {priority} | difficulty: {difficulty} | estimate: {estimate}"
            );
        }

        Ok(())
    }
}
