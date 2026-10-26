use anyhow::{bail, Result};
use clap::Parser;

use crate::cli::{get_project_path, Opts};
use wiptool_core::ProjectStorage;

/// Renames a task while preserving its state and metadata.
#[derive(Parser, Clone)]
pub(crate) struct RenameTask {
    name: String,
    new_name: String,

    #[arg(short, long)]
    milestone: Option<String>,

    project: Option<String>,
}

impl RenameTask {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let mut path = get_project_path(&self.project, storage)?;

        if let Some(milestone) = &self.milestone {
            if path.vec.iter().any(|segment| segment.is_milestone()) {
                bail!("milestone specified both in path and with --milestone");
            }
            path.add_milestone(milestone)?;
        }
        path.add_task(&self.name)?;
        storage.rename_task(path, self.new_name.clone())?;
        storage.commit_changes()
    }
}
