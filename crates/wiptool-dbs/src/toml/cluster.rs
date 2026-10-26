use wiptool_core::{repr, Location, PathSegment, ProjectDir, ProjectStorage, Status};

use anyhow::{anyhow, bail, ensure, Result};
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use crate::{
    toml::{TomlMilestone, TomlProject, TomlProjectHeader, TomlTask},
    StatusDB,
};

#[derive(Debug, Serialize, Deserialize)]
struct StatusInstance {
    storage_location: Location,
    project_location: Option<Location>,

    /**
     * the project may or may not be loaded into mem yet
     */
    #[serde(skip)]
    db: Option<StatusDB>,
}

/**
keeps track of all the status.toml databases
*/
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatusCluster {
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    instances: HashMap<ProjectDir, StatusInstance>,
    #[serde(skip)]
    db_path: PathBuf,
}

impl StatusCluster {
    fn root_path(path: &ProjectDir) -> Result<ProjectDir> {
        ensure!(path.len() >= 1, "path must contain at least one segment");
        ensure!(
            !path.get_section(0).is_task(),
            "path root segment must be a project"
        );

        Ok(ProjectDir {
            vec: vec![PathSegment::project(path.get_section(0).get_name())],
        })
    }

    pub fn save(&self) -> Result<()> {
        log::info!("saving db @ {}", self.db_path.display());
        let mut file = File::create(&self.db_path)?;
        let buf = toml::to_string_pretty(self)?;
        file.write_all(buf.as_bytes())?;

        Ok(())
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let mut file = File::open(&path)?;
        let mut buf = String::new();

        file.read_to_string(&mut buf)?;
        let mut data: StatusCluster = toml::from_str(buf.as_str())?;
        data.db_path = path.as_ref().to_path_buf();
        return Ok(data);
    }

    fn get_instance_db(&mut self, path: &ProjectDir) -> Result<&mut StatusDB> {
        let root = Self::root_path(path)?;
        let instance = self
            .instances
            .get_mut(&root)
            .ok_or(anyhow!("could not find project"))?;

        if instance.db.is_none() {
            instance.db = Some(StatusDB::new(instance.storage_location.clone())?);
        }

        Ok(instance.db.as_mut().unwrap())
    }
}

impl ProjectStorage for StatusCluster {
    fn project_exists(&mut self, path: ProjectDir) -> Result<bool> {
        return Ok(self.get_instance_db(&path).is_ok());
    }

    fn task_exists(&mut self, path: ProjectDir) -> Result<bool> {
        let mut project_path = path.clone();
        project_path.remove_task()?;

        self.get_instance_db(&project_path)?.task_exists(path)
    }

    fn get_projects_path(&mut self) -> Result<Vec<ProjectDir>> {
        let mut projects = Vec::new();

        for path in self.instances.keys() {
            ensure!(
                path.len() >= 1,
                "instance path must contain at least one segment"
            );
            projects.push(path.clone());
        }

        return Ok(projects);
    }

    fn get_project(&mut self, path: ProjectDir) -> Result<repr::Project> {
        let root = Self::root_path(&path)?;
        let project_location = self
            .instances
            .get(&root)
            .ok_or(anyhow!("project not found"))?
            .project_location
            .clone();

        let mut project = self.get_instance_db(&path)?.get_project(path)?;
        project.location = project_location;
        Ok(project)
    }

    fn get_storage_location(&mut self, path: ProjectDir) -> Result<Location> {
        let root = Self::root_path(&path)?;
        self.instances
            .get(&root)
            .map(|instance| instance.storage_location.clone())
            .ok_or(anyhow!("project not found"))
    }

    fn promote_task(&mut self, path: ProjectDir) -> Result<()> {
        self.get_instance_db(&path)?.promote_task(path)
    }

    fn get_task(&mut self, path: ProjectDir) -> Result<repr::Task> {
        self.get_instance_db(&path)?.get_task(path)
    }

    fn rename_project(&mut self, path: ProjectDir, new_name: String) -> Result<()> {
        let old_root = Self::root_path(&path)?;
        ensure!(path.len() == 1, "renaming subprojects is not supported");
        let new_root = ProjectDir::parse(format!("{new_name}/"))?;

        if old_root == new_root {
            ensure!(self.instances.contains_key(&old_root), "project not found");
            return Ok(());
        }
        ensure!(
            !self.instances.contains_key(&new_root),
            "project already exists: {new_name}"
        );

        self.get_instance_db(&old_root)?
            .rename_project(old_root.clone(), new_name)?;
        let instance = self
            .instances
            .remove(&old_root)
            .ok_or_else(|| anyhow!("project not found"))?;
        self.instances.insert(new_root, instance);
        Ok(())
    }

    fn rename_milestone(&mut self, path: ProjectDir, new_name: String) -> Result<()> {
        self.get_instance_db(&path)?
            .rename_milestone(path, new_name)
    }

    fn rename_task(&mut self, path: ProjectDir, new_name: String) -> Result<()> {
        self.get_instance_db(&path)?.rename_task(path, new_name)
    }

    fn create_milestone(&mut self, path: ProjectDir, milestone: repr::Milestone) -> Result<()> {
        self.get_instance_db(&path)?
            .create_milestone(path, milestone)
    }

    fn get_milestones(&mut self, path: ProjectDir) -> Result<Vec<(String, repr::Milestone)>> {
        self.get_instance_db(&path)?.get_milestones(path)
    }

    fn set_current_milestone(&mut self, path: ProjectDir, milestone: String) -> Result<()> {
        self.get_instance_db(&path)?
            .set_current_milestone(path, milestone)
    }

    fn set_status(&mut self, path: ProjectDir, status: Status) -> Result<()> {
        self.get_instance_db(&path)?.set_status(path, status)
    }

    fn commit_changes(&mut self) -> Result<()> {
        for instance in self.instances.values_mut() {
            // check if the db is loaded into memory
            if let Some(db) = &mut instance.db {
                db.commit_changes()?;
            }
        }

        self.save()?;

        Ok(())
    }

    fn create_project(
        &mut self,
        path: ProjectDir,
        project: repr::Project,
        storage_location: Location,
    ) -> Result<()> {
        let project_location = project.location;
        let current_milestone = if project.current_milestone.is_empty() {
            project
                .milestones
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| StatusDB::DEFAULT_MILESTONE.to_string())
        } else {
            project.current_milestone.clone()
        };

        let mut milestones: Vec<TomlMilestone> = project
            .milestones
            .into_iter()
            .map(|(name, milestone)| {
                let mut todo = HashMap::new();
                let mut done = HashMap::new();

                for (task_name, task) in milestone.tasks {
                    let db_task = TomlTask {
                        priority: task.priority,
                        difficulty: task.difficulty,
                        time_estimate: task.time_estimate,
                    };

                    if task.todo {
                        todo.insert(task_name, db_task);
                    } else {
                        done.insert(task_name, db_task);
                    }
                }

                TomlMilestone {
                    name,
                    status: milestone.status,
                    version: milestone.version,
                    todo,
                    done,
                }
            })
            .collect();

        if milestones.is_empty() {
            milestones.push(TomlMilestone {
                name: current_milestone.clone(),
                status: Status::default(),
                version: None,
                todo: HashMap::new(),
                done: HashMap::new(),
            });
        }

        let db_project = TomlProject {
            project: TomlProjectHeader {
                milestone: Some(current_milestone),
                edition: project.edition,
                name: project.name,
                kinds: project.kinds,
                description: project.description,
                subprojects: Vec::new(),
            },
            milestones,
        };

        let mut db = StatusDB {
            project: db_project,
            storage_location: storage_location.clone(),
        };
        db.commit_changes()?;

        self.instances.insert(
            path,
            StatusInstance {
                storage_location,
                project_location,
                db: Some(db),
            },
        );

        Ok(())
    }

    fn create_task(&mut self, path: ProjectDir, task: repr::Task) -> Result<()> {
        self.get_instance_db(&path)?.create_task(path, task)?;

        return Ok(());
    }

    fn delete_project(&mut self, path: ProjectDir) -> Result<()> {
        self.instances.remove(&path);
        return Ok(());
    }

    fn delete_task(&mut self, path: ProjectDir) -> Result<()> {
        self.get_instance_db(&path)?.delete_task(path)?;
        return Ok(());
    }

    fn find_project_at_location(&self, location: Location) -> Result<ProjectDir> {
        for (dir, instance) in &self.instances {
            if let Some(project_location) = &instance.project_location {
                if *project_location == location {
                    return Ok(dir.clone());
                }
            }
        }
        bail!("failed to find project at {location:?}")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{StatusDB, TomlMilestone, TomlProject, TomlProjectHeader, TomlTask};
    use wiptool_core::{Location, ProjectDir, ProjectStorage, Status, Version};

    #[test]
    fn task_metrics_are_optional_in_toml() {
        let task: TomlTask = toml::from_str("").expect("empty task should deserialize");

        assert_eq!(task.priority, None);
        assert_eq!(task.difficulty, None);
        assert_eq!(task.time_estimate, None);
        assert_eq!(toml::to_string(&task).expect("task should serialize"), "");
    }

    #[test]
    fn existing_numeric_task_metrics_still_deserialize() {
        let task: TomlTask =
            toml::from_str("priority = 2.5\ndifficulty = 3.0\ntime_estimate = 45\n")
                .expect("numeric task metrics should deserialize");

        assert_eq!(task.priority, Some(2.5));
        assert_eq!(task.difficulty, Some(3.0));
        assert_eq!(task.time_estimate, Some(45));
    }

    #[test]
    fn status_db_renames_project_milestone_and_task() {
        let mut todo = HashMap::new();
        todo.insert(
            "old-task".to_string(),
            TomlTask {
                priority: Some(2.0),
                difficulty: None,
                time_estimate: Some(30),
            },
        );
        let mut db = StatusDB {
            project: TomlProject {
                project: TomlProjectHeader {
                    name: "old-project".to_string(),
                    edition: Version::default(),
                    milestone: Some("old-milestone".to_string()),
                    kinds: Vec::new(),
                    description: String::new(),
                    subprojects: Vec::new(),
                },
                milestones: vec![TomlMilestone {
                    name: "old-milestone".to_string(),
                    status: Status::default(),
                    version: None,
                    todo,
                    done: HashMap::new(),
                }],
            },
            storage_location: Location::Local("unused.toml".into()),
        };

        db.rename_task(
            ProjectDir::parse("old-project/@old-milestone/old-task").unwrap(),
            "new-task".to_string(),
        )
        .unwrap();
        db.rename_milestone(
            ProjectDir::parse("old-project/@old-milestone/").unwrap(),
            "new-milestone".to_string(),
        )
        .unwrap();
        db.rename_project(
            ProjectDir::parse("old-project/").unwrap(),
            "new-project".to_string(),
        )
        .unwrap();

        assert_eq!(db.project.project.name, "new-project");
        assert_eq!(
            db.project.project.milestone.as_deref(),
            Some("new-milestone")
        );
        assert!(db.project.milestones[0].todo.contains_key("new-task"));
        assert_eq!(
            db.project.milestones[0].todo["new-task"].time_estimate,
            Some(30)
        );
    }
}
