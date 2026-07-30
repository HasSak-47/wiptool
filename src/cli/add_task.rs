use anyhow::{bail, Result};
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr,
};

#[derive(Parser, Clone)]
pub(crate) struct AddTask {
    project: String,
    name: String,
    #[arg(short, long)]
    done: bool,
    difficulty: f64,
    priority: f64,
}

impl AddTask {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let task = repr::Task {
            name: self.name.clone(),
            priority: self.priority,
            difficulty: self.difficulty,
        };

        let project = self.project.clone() + if self.project.ends_with("/") { "" } else { "/" };

        let path = ProjectDir::parse(&project)?;

        let mut task_path = path.clone();
        task_path.add_task(&self.name)?;

        if storage.task_exists(task_path)? {
            bail!("Task [{}]: already exist", self.name);
        }

        if self.done {
            storage.insert_task_todo(path, task)?;
        } else {
            storage.insert_task_todo(path, task)?;
        }
        storage.commit_changes()?;

        Ok(())
    }
}
