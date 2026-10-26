use anyhow::Result;
use clap::Parser;

use crate::cli::Opts;
use wiptool_core::{ProjectDir, ProjectStorage};

/// Renames a milestone.
#[derive(Parser, Clone)]
pub(crate) struct RenameMilestone {
    project: String,
    name: String,
    new_name: String,
}

impl RenameMilestone {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let mut path = ProjectDir::parse(format!("{}/", self.project.trim_end_matches('/')))?;
        path.add_milestone(&self.name)?;
        storage.rename_milestone(path, self.new_name.clone())?;
        storage.commit_changes()
    }
}
