use anyhow::Result;
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
};

/**
sets the current milestone for a project
*/
#[derive(Parser, Clone)]
pub(crate) struct SetCurrentMilestone {
    project: String,
    milestone: String,
}

impl SetCurrentMilestone {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let path = ProjectDir::parse(
            self.project.clone() + if self.project.ends_with("/") { "" } else { "/" },
        )?;

        storage.set_current_milestone(path, self.milestone.clone())?;
        storage.commit_changes()?;

        Ok(())
    }
}
