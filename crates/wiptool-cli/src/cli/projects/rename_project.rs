use anyhow::Result;
use clap::Parser;

use crate::cli::Opts;
use wiptool_core::{ProjectDir, ProjectStorage};

/// Renames a tracked project.
#[derive(Parser, Clone)]
pub(crate) struct RenameProject {
    name: String,
    new_name: String,
}

impl RenameProject {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let path = ProjectDir::parse(format!("{}/", self.name))?;
        storage.rename_project(path, self.new_name.clone())?;
        storage.commit_changes()
    }
}
