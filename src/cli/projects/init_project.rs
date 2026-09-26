use std::path::PathBuf;

use clap::Parser;

use crate::{cli::Opts, interface::ProjectStorage};

/**
starts tracking a project with an existing status.toml
*/
#[derive(Parser, Clone)]
pub(crate) struct InitProject {
    name: String,
    location: PathBuf,
}

impl InitProject {
    pub fn run(&self, _: &Opts, _: &mut Box<dyn ProjectStorage>) -> anyhow::Result<()> {
        todo!()
    }
}
