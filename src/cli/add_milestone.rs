use anyhow::Result;
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr::{Milestone, Status},
    version::Version,
};

/**
adds a milestone to a project
*/
#[derive(Parser, Clone)]
pub(crate) struct AddMilestone {
    milestone: String,
    project: String,

    #[arg(long, default_value_t = Status::default())]
    status: Status,

    #[arg(long)]
    version: Option<Version>,
}

impl AddMilestone {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let mut path = ProjectDir::parse(
            self.project.clone() + if self.project.ends_with("/") { "" } else { "/" },
        )?;
        path.add_milestone(&self.milestone)?;

        storage.create_milestone(
            path,
            Milestone {
                status: self.status.clone(),
                version: self.version.clone(),
                tasks: Default::default(),
            },
        )?;
        storage.commit_changes()?;

        Ok(())
    }
}
