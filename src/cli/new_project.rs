use std::{env::current_dir, path::PathBuf};

use anyhow::{bail, Result};
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr::{self, Location},
};

/**
creates new project and status.toml
*/
#[derive(Parser, Clone)]
pub(crate) struct NewProject {
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dbs::toml::StatusCluster;
    use std::{
        fs::File,
        time::{SystemTime, UNIX_EPOCH},
    };

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
