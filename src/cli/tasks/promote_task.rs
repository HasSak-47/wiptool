use std::path::PathBuf;

use clap::Parser;

use crate::{cli::Opts, interface::ProjectStorage};

/**
makes the task into a project with a parent project
*/
#[derive(Parser, Clone)]
pub(crate) struct PromoteTask {
    name: String,
    new_path: PathBuf,
}

impl PromoteTask {
    pub fn run(&self, _: &Opts, _: &mut Box<dyn ProjectStorage>) -> anyhow::Result<()> {
        todo!()
    }
}
