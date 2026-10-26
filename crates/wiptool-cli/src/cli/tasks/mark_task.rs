use clap::Parser;

use crate::cli::{get_project_path, Opts};
use wiptool_core::ProjectStorage;

/**
marks if the task todo is done or not
*/
#[derive(Parser, Clone)]
pub(crate) struct MarkTask {
    name: String,

    #[arg(short, long)]
    done: bool,

    project: Option<String>,
}

impl MarkTask {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> anyhow::Result<()> {
        let path = get_project_path(&self.project, storage)?;

        let mut task_path = path.clone();
        task_path.add_task(&self.name)?;

        let mut task = storage.get_task(task_path.clone())?;
        task.todo = !self.done;

        storage.create_task(task_path, task)?;
        storage.commit_changes()?;

        Ok(())
    }
}
