use anyhow::Result;
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr::Milestone,
};

/**
lists milestones for a project
*/
#[derive(Parser, Clone)]
pub(crate) struct ListMilestones {
    project: String,
}

impl ListMilestones {
    fn completion(milestone: &Milestone) -> f64 {
        let done = milestone
            .tasks
            .values()
            .filter(|task| !task.todo)
            .fold(0., |total, task| total + task.difficulty);
        let todo = milestone
            .tasks
            .values()
            .filter(|task| task.todo)
            .fold(0., |total, task| total + task.difficulty);
        let total = done + todo;

        if total == 0. {
            1.
        } else {
            done / total
        }
    }

    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let path = ProjectDir::parse(
            self.project.clone() + if self.project.ends_with("/") { "" } else { "/" },
        )?;
        let project = storage.get_project(path.clone())?;
        let current = project.current_milestone;

        for (name, milestone) in storage.get_milestones(path)? {
            let marker = if name == current { "*" } else { " " };
            println!(
                "{marker} @{} | {} | {:.1}%",
                name,
                milestone.status,
                Self::completion(&milestone) * 100.
            );
        }

        Ok(())
    }
}
