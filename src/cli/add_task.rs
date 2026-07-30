use std::env::current_dir;

use anyhow::{bail, Result};
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr::{self, Location},
};

/**
adds a task to a project
*/
#[derive(Parser, Clone)]
pub(crate) struct AddTask {
    project: Option<String>,
    name: String,
    #[arg(short, long)]
    todo: bool,
    difficulty: f64,
    priority: f64,
}

impl AddTask {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let task = repr::Task {
            name: self.name.clone(),
            priority: self.priority,
            difficulty: self.difficulty,
            todo: self.todo,
        };

        let path = if let Some(project) = &self.project {
            // shitty ass ser/de lmao
            ProjectDir::parse(project.clone() + if project.ends_with("/") { "" } else { "/" })?
        } else {
            let cwd = current_dir()?;
            let mut p = None;
            for path in storage.get_projects_path()? {
                let location = storage.get_project(path.clone())?.location.unwrap();
                if let Location::Local(project_path) = location {
                    if cwd == project_path {
                        p = Some(path.clone());
                        break;
                    }
                }
            }
            if let None = p {
                bail!("there is no project in cwd: {}", cwd.display());
            }
            p.unwrap()
        };

        let mut task_path = path.clone();
        task_path.add_task(&self.name)?;

        if storage.task_exists(task_path.clone())? {
            bail!("Task [{}]: already exist", self.name);
        }

        storage.create_task(task_path, task)?;
        storage.commit_changes()?;

        Ok(())
    }
}
