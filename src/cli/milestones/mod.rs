use clap::Subcommand;

pub mod add_milestone;
pub mod list_milestones;
pub mod rename_milestone;
pub mod set_current_milestone;
pub mod set_status;

use add_milestone::*;
use list_milestones::*;
use rename_milestone::*;
use set_current_milestone::*;

use crate::{
    cli::{milestones::set_status::SetStatus, Opts},
    interface::ProjectStorage,
};

#[derive(Subcommand, Clone)]
pub(crate) enum MilestoneCommands {
    /// Add a milestone to a project.
    Add(AddMilestone),
    /// List a project's milestones.
    List(ListMilestones),
    /// Rename a milestone while preserving its tasks.
    Rename(RenameMilestone),
    /// Select a project's current milestone.
    Set(SetCurrentMilestone),
    /// Change a milestone's status.
    SetStatus(SetStatus),
}

impl MilestoneCommands {
    pub fn run(&self, opts: &Opts, storage: &mut Box<dyn ProjectStorage>) -> anyhow::Result<()> {
        match self {
            MilestoneCommands::Add(cmd) => cmd.run(opts, storage)?,
            MilestoneCommands::List(cmd) => cmd.run(opts, storage)?,
            MilestoneCommands::Set(cmd) => cmd.run(opts, storage)?,
            MilestoneCommands::SetStatus(cmd) => cmd.run(opts, storage)?,
            MilestoneCommands::Rename(cmd) => cmd.run(opts, storage)?,
        }
        return Ok(());
    }
}
