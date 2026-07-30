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
    #[arg(long, default_value_t = false)]
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
        project.location = Some(Location::Local(self.location.clone()));

        let mut db_location = self.location.clone();

        db_location.push("status");
        db_location.set_extension("toml");

        if std::fs::exists(&db_location)? {
            bail!("project at {} already exists", db_location.display());
        }

        storage.create_project(path, project, repr::Location::Local(db_location))?;
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
            if let Location::Local(path) = storage.get_storage_location(loc.clone())? {
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
        log::debug!("running on debug by opts");
    } else if opts.verbose {
        ::log::set_max_level(log::LevelFilter::Info);
        log::debug!("running on info by opts");
    } else {
        ::log::set_max_level(log::LevelFilter::Warn);
        log::debug!("running on warn by opts");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dbs::toml::StatusCluster;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("project_manager_{name}_{timestamp}"))
    }

    #[test]
    fn new_project_keeps_project_location_separate_from_storage_location() {
        let root = test_dir("project_and_storage_locations");
        let project_dir = root.join("project");
        let db_dir = root.join("db");
        let db_path = db_dir.join("projects.toml");

        std::fs::create_dir_all(&project_dir).expect("create project dir");
        std::fs::create_dir_all(&db_dir).expect("create db dir");
        File::create(&db_path).expect("create cluster db");

        let opts = Opts {
            db_path,
            ..Opts::default()
        };
        let cluster = StatusCluster::load(&opts.db_path).expect("load empty cluster db");
        let mut storage: Box<dyn ProjectStorage> = Box::new(cluster);

        NewProject {
            name: "sample".to_string(),
            location: project_dir.clone(),
        }
        .run(&opts, &mut storage)
        .expect("create project");

        let project_path = ProjectDir::parse("sample/").expect("valid project path");
        let status_path = project_dir.join("status.toml");

        assert_eq!(
            storage
                .get_project(project_path.clone())
                .expect("get project")
                .location,
            Some(Location::Local(project_dir.clone()))
        );
        assert_eq!(
            storage
                .get_storage_location(project_path)
                .expect("get storage location"),
            Location::Local(status_path)
        );

        let status_toml =
            std::fs::read_to_string(project_dir.join("status.toml")).expect("read status toml");
        assert!(!status_toml.contains("location"));

        std::fs::remove_dir_all(root).expect("cleanup test dir");
    }
}
