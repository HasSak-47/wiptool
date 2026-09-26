pub mod add_task;
pub mod list_tasks;
pub mod mark_task;
pub mod promote_task;
pub mod remove_task;

use add_task::*;
use list_tasks::*;
use mark_task::*;
use promote_task::*;
use remove_task::*;

use crate::{cli::Opts, interface::ProjectStorage};
use clap::Subcommand;

#[derive(Subcommand, Clone)]
pub(crate) enum TaskCommands {
    Add(AddTask),
    List(ListTasks),
    Mark(MarkTask),
    Promote(PromoteTask),
    Delete(DeleteTask),
}

impl TaskCommands {
    pub fn run(&self, opts: &Opts, storage: &mut Box<dyn ProjectStorage>) -> anyhow::Result<()> {
        match self {
            TaskCommands::Add(cmd) => cmd.run(opts, storage)?,
            TaskCommands::Mark(cmd) => cmd.run(opts, storage)?,
            TaskCommands::List(cmd) => cmd.run(opts, storage)?,
            TaskCommands::Promote(cmd) => cmd.run(opts, storage)?,
            TaskCommands::Delete(cmd) => cmd.run(opts, storage)?,
        }
        return Ok(());
    }
}
