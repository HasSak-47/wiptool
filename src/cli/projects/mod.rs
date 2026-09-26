use clap::Subcommand;

pub mod delete_project;
pub mod init_project;
pub mod list;
pub mod new_project;

use delete_project::*;
use init_project::*;
use list::*;
use new_project::*;

use crate::{cli::Opts, interface::ProjectStorage};

#[derive(Subcommand, Clone)]
pub(crate) enum ProjectCommands {
    Delete(DeleteProject),
    Init(InitProject),
    List(ProjectList),
    New(NewProject),
}

impl ProjectCommands {
    pub fn run(&self, opts: &Opts, storage: &mut Box<dyn ProjectStorage>) -> anyhow::Result<()> {
        match self {
            ProjectCommands::Init(cmd) => cmd.run(opts, storage)?,
            ProjectCommands::New(cmd) => cmd.run(opts, storage)?,
            ProjectCommands::List(cmd) => cmd.run(opts, storage)?,
            ProjectCommands::Delete(cmd) => cmd.run(opts, storage)?,
        }
        return Ok(());
    }
}
