use anyhow::Result;
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr::Status,
};

/**
sets milestone status
*/
#[derive(Parser, Clone)]
pub(crate) struct SetStatus {
    path: String,
    status: Status,
}

impl SetStatus {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let path =
            ProjectDir::parse(self.path.clone() + if self.path.ends_with("/") { "" } else { "/" })?;

        storage.set_status(path, self.status.clone())?;
        storage.commit_changes()?;

        Ok(())
    }
}
