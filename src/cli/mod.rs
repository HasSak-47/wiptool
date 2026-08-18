mod add_milestone;
mod add_task;
mod completion;
mod delete_project;
mod init_project;
mod list;
mod list_milestones;
mod mark_task;
mod new_project;
mod promote_task;
mod remove_task;
mod set_current_milestone;
mod set_status;
mod set_subproject;

use std::{fs::File, path::PathBuf};

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};

use crate::interface::ProjectStorage;

use self::{
    add_milestone::AddMilestone, add_task::AddTask, completion::Completion,
    delete_project::DeleteProject, init_project::InitProject, list::List,
    list_milestones::ListMilestones, mark_task::MarkTask, new_project::NewProject,
    promote_task::PromoteTask, remove_task::RemoveTask, set_current_milestone::SetCurrentMilestone,
    set_status::SetStatus, set_subproject::SetSubproject,
};

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    opts: Opts,

    #[command(subcommand)]
    command: Commands,
}

#[allow(dead_code)]
fn data_dir() -> PathBuf {
    let mut d = dirs::data_dir().unwrap();
    d.push("wiptool");

    d
}

#[derive(Args, Debug, Default, Clone)]
pub(crate) struct Opts {
    #[arg(long, default_value_t = false)]
    debug: bool,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long, default_value_os_t = data_dir())]
    pub(crate) db_path: PathBuf,
}

#[derive(Subcommand, Clone)]
enum Commands {
    NewProject(NewProject),
    SetSubproject(SetSubproject),
    InitProject(InitProject),
    DeleteProject(DeleteProject),
    AddTask(AddTask),
    RemoveTask(RemoveTask),
    PromoteTask(PromoteTask),
    MarkTask(MarkTask),
    List(List),
    Completion(Completion),
    AddMilestone(AddMilestone),
    SetCurrentMilestone(SetCurrentMilestone),
    ListMilestones(ListMilestones),
    SetStatus(SetStatus),
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut opts = cli.opts;

    if opts.debug {
        ::log::set_max_level(log::LevelFilter::Debug);
        log::debug!("running on debug by opts");
    } else if opts.verbose {
        ::log::set_max_level(log::LevelFilter::Info);
        log::debug!("running on info by opts");
    } else {
        ::log::set_max_level(log::LevelFilter::Warn);
        log::debug!("running on warn by opts");
    }
    log::debug!("running on debug mode");

    if let Commands::Completion(completion) = &cli.command {
        completion.run();
        return Ok(());
    }

    if let Err(e) = std::fs::create_dir(&opts.db_path) {
        match e.kind() {
            std::io::ErrorKind::AlreadyExists => {}
            a => bail!("Project Manager Data Error {a:?}"),
        }
    }

    opts.db_path.push("projects");
    opts.db_path.set_extension("toml");
    if !std::fs::exists(&opts.db_path)? {
        let _ = File::create(&opts.db_path);
    }

    let cluster = crate::dbs::toml::StatusCluster::load(&opts.db_path)?;
    log::debug!("storage: {cluster:#?}");
    let mut storage: Box<dyn ProjectStorage> = Box::new(cluster);

    match cli.command {
        Commands::List(l) => l.run(&opts, &mut storage)?,
        Commands::NewProject(new) => new.run(&opts, &mut storage)?,
        Commands::DeleteProject(delete) => delete.run(&opts, &mut storage)?,
        Commands::AddTask(task) => task.run(&opts, &mut storage)?,
        Commands::AddMilestone(milestone) => milestone.run(&opts, &mut storage)?,
        Commands::RemoveTask(remove_task) => remove_task.run(&opts, &mut storage)?,
        Commands::MarkTask(mark_task) => mark_task.run(&opts, &mut storage)?,
        Commands::SetCurrentMilestone(set) => set.run(&opts, &mut storage)?,
        Commands::ListMilestones(list) => list.run(&opts, &mut storage)?,
        Commands::SetStatus(set) => set.run(&opts, &mut storage)?,
        Commands::Completion(_) => unreachable!("completion exits before storage initialization"),
        _ => todo!("Todo"),
    }

    Ok(())
}
