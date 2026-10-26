use clap::Parser;

use crate::cli::{get_project_path, Opts};
use wiptool_core::ProjectStorage;

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
        let path = get_project_path(&self.project, storage)?;

        let mut task_path = path.clone();
        task_path.add_task(&self.name)?;

        storage.delete_task(task_path)?;
        storage.commit_changes()?;

        Ok(())
    }
}
