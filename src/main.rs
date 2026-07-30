pub mod dbs;
pub mod inner_log;
pub mod interface;
pub mod repr;
pub mod version;

use std::{env::current_dir, fs::File, path::PathBuf};

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};

use crate::{
    inner_log::init_log,
    interface::{ProjectDir, ProjectStorage},
    repr::Location,
};

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct CLI {
    #[command(flatten)]
    opts: Opts,

    #[command(subcommand)]
    command: Commands,
}

#[allow(dead_code)]
fn data_dir() -> PathBuf {
    let mut d = dirs::data_dir().unwrap();
    d.push("project_manager");

    return d;
}

#[derive(Args, Debug, Default, Clone)]
struct Opts {
    #[arg(long)]
    debug: bool,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long, default_value_os_t = data_dir())]
    db_path: PathBuf,
}

#[derive(Parser, Clone)]
struct NewProject {
    name: String,

    #[arg(short, long, default_value_os_t = current_dir().unwrap())]
    location: PathBuf,
}

impl NewProject {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let mut path = ProjectDir::new();
        path.add_project(self.name.clone())?;

        let mut project = repr::Project::default();
        project.name = self.name.clone();

        let mut location = self.location.clone();
        location.push("status");
        location.set_extension("toml");

        if std::fs::exists(&location)? {
            bail!("project at {} already exists", location.display());
        }

        storage.create_project(path, project, repr::Location::Local(location))?;
        storage.commit_changes()?;

        return Ok(());
    }
}

#[derive(Parser, Clone)]
struct SetSubproject {
    parent: String,
    child: String,
}

#[derive(Parser, Clone)]
struct InitProject {
    name: String,
    location: PathBuf,
}

#[derive(Parser, Clone)]
struct DeleteProject {
    name: String,

    #[arg(long = "soft-delete", default_value_t = false)]
    soft: bool,
}

impl DeleteProject {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> anyhow::Result<()> {
        let loc = ProjectDir::parse(&format!("{}/", self.name))?;
        log::debug!("deleting project: {}", loc);
        if !self.soft {
            let location = storage.get_project_location(loc.clone())?;
            if let Location::Local(path) = location {
                log::debug!("deleting toml path: {}", path.display());
                if let Err(e) = std::fs::remove_file(path) {
                    log::error!("{}", e);
                }
            }
        }
        storage.delete_project(loc)?;
        storage.commit_changes()?;
        return Ok(());
    }
}

#[derive(Parser, Clone)]
struct AddTask {
    project: String,
    name: String,
    #[arg(short, long)]
    done: bool,
    difficulty: f64,
    priority: f64,
}

impl AddTask {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let task = repr::Task {
            name: self.name.clone(),
            priority: self.priority,
            difficulty: self.difficulty,
        };

        let project = self.project.clone() + if self.project.ends_with("/") { "" } else { "/" };

        let path = ProjectDir::parse(&project)?;

        let mut task_path = path.clone();
        task_path.add_task(&self.name)?;

        if storage.task_exists(task_path)? {
            bail!("Task [{}]: already exist", self.name);
        }

        if self.done {
            storage.insert_task_todo(path, task)?;
        } else {
            storage.insert_task_todo(path, task)?;
        }
        storage.commit_changes()?;
        return Ok(());
    }
}

#[derive(Parser, Clone)]
struct RemoveTask {
    name: String,
}

#[derive(Parser, Clone)]
struct PromoteTask {
    name: String,
    new_path: PathBuf,
}

#[derive(Parser, Clone)]
struct MarkTask {
    name: String,
    todo: bool,
}

#[derive(Parser, Clone)]
struct List {
    #[arg(short, long)]
    color: bool,
    #[arg(short, long)]
    location: bool,
}

impl List {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let paths = storage.get_projects_path()?;
        let home_dir = dirs::home_dir();

        if self.location {
            for path in paths {
                log::debug!("getting path: {path:?}",);
                let project = storage.get_project(path.clone())?;
                match project.location {
                    Some(crate::repr::Location::Local(project_location)) => {
                        let display_location = if let Some(home) = &home_dir {
                            match project_location.strip_prefix(home) {
                                Ok(relative) => {
                                    if relative.as_os_str().is_empty() {
                                        String::from("~")
                                    } else {
                                        PathBuf::from("~").join(relative).display().to_string()
                                    }
                                }
                                Err(_) => project_location.display().to_string(),
                            }
                        } else {
                            project_location.display().to_string()
                        };

                        if self.color {
                            println!("{path} @ \x1b[1;34m{display_location}\x1b[0m");
                        } else {
                            println!("{path} @ {display_location}");
                        }
                    }
                    Some(crate::repr::Location::URL(project_location)) => {
                        println!("{path} @ {project_location}");
                    }
                    None => {
                        println!("{path} @ <unknown>");
                    }
                }
            }
        } else {
            for path in paths {
                println!("{path}",);
            }
        }

        return Ok(());
    }
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
}

fn main() -> Result<()> {
    init_log();

    let cli = CLI::parse();
    let mut opts = cli.opts;

    if opts.debug {
        ::log::set_max_level(log::LevelFilter::Debug);
    } else if opts.verbose {
        ::log::set_max_level(log::LevelFilter::Info);
    } else if opts.verbose {
        ::log::set_max_level(log::LevelFilter::Warn);
    }
    log::debug!("running on debug mode");

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
        Commands::AddTask(task) => task.run(&opts, &mut storage)?,
        Commands::DeleteProject(delete) => delete.run(&opts, &mut storage)?,
        _ => todo!("Todo"),
    }
    return Ok(());
}
