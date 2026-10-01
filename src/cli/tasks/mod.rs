pub mod add_task;
pub mod edit_task;
pub mod list_tasks;
pub mod mark_task;
pub mod promote_task;
pub mod remove_task;
pub mod rename_task;

use add_task::*;
use list_tasks::*;
use mark_task::*;
use promote_task::*;
use remove_task::*;
use rename_task::*;

use crate::{
    cli::{tasks::edit_task::EditTask, Opts},
    interface::ProjectStorage,
};
use clap::Subcommand;

#[derive(Subcommand, Clone)]
pub(crate) enum TaskCommands {
    Add(AddTask),
    List(ListTasks),
    Mark(MarkTask),
    Promote(PromoteTask),
    Rename(RenameTask),
    Delete(DeleteTask),
    Edit(EditTask),
}

impl TaskCommands {
    pub fn run(&self, opts: &Opts, storage: &mut Box<dyn ProjectStorage>) -> anyhow::Result<()> {
        match self {
            TaskCommands::Add(cmd) => cmd.run(opts, storage)?,
            TaskCommands::Mark(cmd) => cmd.run(opts, storage)?,
            TaskCommands::List(cmd) => cmd.run(opts, storage)?,
            TaskCommands::Promote(cmd) => cmd.run(opts, storage)?,
            TaskCommands::Delete(cmd) => cmd.run(opts, storage)?,
            TaskCommands::Rename(cmd) => cmd.run(opts, storage)?,
            TaskCommands::Edit(cmd) => cmd.run(opts, storage)?,
        }
        return Ok(());
    }
}
