use std::env::current_dir;

use anyhow::bail;
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr::Location,
};

/**
deletes a task from the project tree
*/
#[derive(Parser, Clone)]
pub(crate) struct DeleteTask {
    name: String,
    project: Option<String>,
}

impl DeleteTask {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> anyhow::Result<()> {
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

        storage.delete_task(task_path)?;
        storage.commit_changes()?;

        Ok(())
    }
}
