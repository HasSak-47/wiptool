use clap::Subcommand;

pub mod add_milestone;
pub mod list_milestones;
pub mod set_current_milestone;

use add_milestone::*;
use list_milestones::*;
use set_current_milestone::*;

#[derive(Subcommand, Clone)]
pub(crate) enum MilestoneCommands {
    Add(AddMilestone),
    List(ListMilestones),
    Set(SetCurrentMilestone),
}
