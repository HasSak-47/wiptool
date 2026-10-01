use std::env::current_dir;

use anyhow::{bail, Result};
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr::Location,
};

/**
adds a task to a project
*/
#[derive(Debug, Parser, Clone)]
pub(crate) struct EditTask {
    name: String,

    #[arg(long)]
    difficulty: Option<f64>,
    #[arg(long)]
    priority: Option<f64>,
    #[arg(long)]
    time_estimate: Option<u64>,

    #[arg(long, conflicts_with = "difficulty")]
    remove_difficulty: bool,
    #[arg(long, conflicts_with = "priority")]
    remove_priority: bool,
    #[arg(long, conflicts_with = "time_estimate")]
    remove_time_estimate: bool,

    #[arg(short, long)]
    milestone: Option<String>,

    project: Option<String>,
}

impl EditTask {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        log::debug!("{self:?}");
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
        if let Some(milestone) = &self.milestone {
            if task_path.vec.iter().any(|segment| segment.is_milestone()) {
                bail!("milestone specified both in path and with --milestone");
            }
            task_path.add_milestone(milestone)?;
        }
        task_path.add_task(&self.name)?;

        let mut storage_task = storage.get_task(task_path.clone())?;

        if self.remove_priority {
            storage_task.priority = None;
        }
        if self.remove_time_estimate {
            storage_task.time_estimate = None;
        }
        if self.remove_difficulty {
            storage_task.difficulty = None;
        }

        if let Some(priority) = storage_task.priority {
            storage_task.priority = Some(priority);
        }
        if let Some(time_estimate) = storage_task.time_estimate {
            storage_task.time_estimate = Some(time_estimate);
        }
        if let Some(difficulty) = storage_task.difficulty {
            storage_task.difficulty = Some(difficulty);
        }

        storage.create_task(task_path, storage_task)?;
        storage.commit_changes()?;

        Ok(())
    }
}
