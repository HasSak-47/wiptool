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

    #[arg(short = 'l', long = "location", alias = "project-location", default_value_os_t = current_dir().unwrap())]
    project_location: PathBuf,

    #[arg(short = 's', long = "storage-location")]
    storage_location: Option<PathBuf>,

    #[arg(
        short = 'S',
        long = "storage-in-data",
        conflicts_with = "storage-location"
    )]
    storage_in_data: bool,
}

impl NewProject {
    pub fn run(&self, opts: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let mut path = ProjectDir::new();
        path.add_project(self.name.clone())?;

        let mut project = repr::Project::default();
        project.name = self.name.clone();
        project.location = Some(Location::Local(self.project_location.clone()));

        let db_location = if self.storage_in_data {
            let data_dir = opts
                .db_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("db path has no parent directory"))?;
            data_dir.join(format!("{}.toml", self.name))
        } else if let Some(storage_location) = &self.storage_location {
            storage_location.clone()
        } else {
            let mut db_location = self.project_location.clone();
            db_location.push("status");
            db_location.set_extension("toml");
            db_location
        };

        if std::fs::exists(&db_location)? {
            bail!("project at {} already exists", db_location.display());
        }

        storage
            .create_project(path, project, repr::Location::Local(db_location.clone()))
            .expect(&format!(
                "failed to create project to: {}",
                db_location.display()
            ));
        storage
            .commit_changes()
            .expect(&format!("failed to commit to: {}", db_location.display()));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dbs::toml::StatusCluster;
    use crate::repr::{Milestone, Status};
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
        let status_path = root.join("project-state.toml");

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
            project_location: project_dir.clone(),
            storage_location: Some(status_path.clone()),
            storage_in_data: false,
        }
        .run(&opts, &mut storage)
        .expect("create project");

        let project_path = ProjectDir::parse("sample/").expect("valid project path");

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

        let status_toml = std::fs::read_to_string(root.join("project-state.toml"))
            .expect("read custom status toml");
        assert!(!status_toml.contains("location"));

        std::fs::remove_dir_all(root).expect("cleanup test dir");
    }

    #[test]
    fn new_project_can_store_status_in_data_dir_with_project_name() {
        let root = test_dir("data_dir_storage");
        let project_dir = root.join("project");
        let db_dir = root.join("db");
        let db_path = db_dir.join("projects.toml");
        let status_path = db_dir.join("sample.toml");

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
            project_location: project_dir.clone(),
            storage_location: None,
            storage_in_data: true,
        }
        .run(&opts, &mut storage)
        .expect("create project");

        let project_path = ProjectDir::parse("sample/").expect("valid project path");

        assert_eq!(
            storage
                .get_storage_location(project_path)
                .expect("get storage location"),
            Location::Local(status_path.clone())
        );
        assert!(status_path.exists(), "status file was not created");

        std::fs::remove_dir_all(root).expect("cleanup test dir");
    }

    #[test]
    fn storage_can_add_and_update_milestones() {
        let root = test_dir("milestones");
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
            project_location: project_dir.clone(),
            storage_location: None,
            storage_in_data: false,
        }
        .run(&opts, &mut storage)
        .expect("create project");

        let project_path = ProjectDir::parse("sample/").expect("valid project path");
        let milestone_path = ProjectDir::parse("sample/@v1/").expect("valid milestone path");

        storage
            .create_milestone(
                milestone_path.clone(),
                Milestone {
                    status: Status::default(),
                    version: None,
                    tasks: Default::default(),
                },
            )
            .expect("create milestone");
        storage
            .set_current_milestone(project_path.clone(), "v1".to_string())
            .expect("set current milestone");
        storage
            .set_status(
                milestone_path,
                "in-progress.active".parse().expect("valid status"),
            )
            .expect("set status");

        let project = storage
            .get_project(project_path.clone())
            .expect("get project");
        assert_eq!(project.current_milestone, "v1");
        assert_eq!(
            project
                .milestones
                .get("v1")
                .expect("milestone exists")
                .status
                .to_string(),
            "in-progress.active"
        );
        assert_eq!(
            storage
                .get_milestones(project_path)
                .expect("get milestones")
                .len(),
            2
        );

        std::fs::remove_dir_all(root).expect("cleanup test dir");
    }
}
