use crate::{
    interface::{ProjectDir, ProjectStorage},
    repr::Location,
    version::Version,
};

use anyhow::{anyhow, bail, ensure, Result};
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectHeader {
    pub version: Option<Version>,
    pub edition: Version,
    pub name: String,
    pub description: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subprojects: Vec<ProjectDir>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Task {
    pub priority: f64,
    pub difficulty: f64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Project {
    pub project: ProjectHeader,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub todo: HashMap<String, Task>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub done: HashMap<String, Task>,
}

/**
represents a file at location that contains a toml describing the project
idealy it should be a status.toml at the root of the project dir
*/
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusDB {
    pub project: Project,
    pub storage_location: Location,
}

impl StatusDB {
    pub fn ensure_project(&self, path: &ProjectDir) -> Result<()> {
        ensure!(
            self.project.project.name == path.get_section(0).get_name(),
            "you can only get a name of the root project name"
        );

        return Ok(());
    }

    pub fn new(storage_location: Location) -> Result<Self> {
        let string = match &storage_location {
            Location::Local(path) => {
                log::debug!("loading toml file at: {}", path.display());
                let mut file = File::open(path)?;
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                buf
            }
            Location::URL(_) => bail!("loading project data from URL is not supported yet"),
        };

        let project: Project = toml::from_str(string.as_str())?;

        return Ok(StatusDB {
            project: project,
            storage_location,
        });
    }
}

impl ProjectStorage for StatusDB {
    fn project_exists(&mut self, path: ProjectDir) -> Result<bool> {
        self.ensure_project(&path)?;
        ensure!(path.len() == 1);

        return Ok(path.get_section(0).get_name() == self.project.project.name);
    }

    fn task_exists(&mut self, path: ProjectDir) -> Result<bool> {
        self.ensure_project(&path)?;

        let name = path.get_section(1).get_name();
        if self.project.todo.contains_key(&name) || self.project.todo.contains_key(&name) {
            return Ok(true);
        }
        return Ok(false);
    }

    fn get_projects_path(&mut self) -> Result<Vec<ProjectDir>> {
        return Ok(vec![ProjectDir {
            vec: vec![crate::interface::PathSegment::project(
                self.project.project.name.clone(),
            )],
        }]);
    }

    fn promote_task(&mut self, _: crate::interface::ProjectDir) -> anyhow::Result<()> {
        bail!("promoting not available for toml database");
    }

    fn get_project(&mut self, path: crate::interface::ProjectDir) -> Result<crate::repr::Project> {
        self.ensure_project(&path)?;

        return Ok(crate::repr::Project {
            version: self.project.project.version.clone(),
            edition: self.project.project.edition.clone(),

            location: None,
            name: self.project.project.name.clone(),
            description: self.project.project.description.clone(),
            subprojects: Vec::new(),
            tasks: self
                .project
                .todo
                .iter()
                .map(|(k, v)| crate::repr::Task {
                    name: k.clone(),
                    todo: true,
                    priority: v.priority,
                    difficulty: v.difficulty,
                })
                .chain(self.project.done.iter().map(|(k, v)| crate::repr::Task {
                    name: k.clone(),
                    todo: false,
                    priority: v.priority,
                    difficulty: v.difficulty,
                }))
                .collect(),
        });
    }

    fn get_task(&mut self, path: crate::interface::ProjectDir) -> Result<crate::repr::Task> {
        ensure!(
            self.project.project.name == path.get_section(0).get_name(),
            "you can only get a name the root project name"
        );

        ensure!(path.len() == 2, "subprojects are not handled yet...");

        ensure!(
            path.get_section(1).is_task(),
            "you can only get a name the root project name"
        );

        let task_name = path.get_section(1).get_name();

        if let Some(task) = self.project.todo.get(&task_name) {
            return Ok(crate::repr::Task {
                name: task_name,
                todo: true,
                priority: task.priority,
                difficulty: task.difficulty,
            });
        }

        if let Some(task) = self.project.done.get(&task_name) {
            return Ok(crate::repr::Task {
                name: task_name,
                priority: task.priority,
                todo: false,
                difficulty: task.difficulty,
            });
        }

        bail!("no exisiting task")
    }

    fn commit_changes(&mut self) -> Result<()> {
        let path = if let Location::Local(path) = &self.storage_location {
            path
        } else {
            bail!("Cannot edit url data")
        };

        let mut file = File::create(path)?;
        let buf = toml::to_string_pretty(&self.project)?;
        file.write_all(buf.as_bytes())?;

        return Ok(());
    }

    fn create_project(&mut self, _: ProjectDir, _: repr::Project, _: Location) -> Result<()> {
        bail!("Creating project not available for Basic TOML DB")
    }

    fn get_storage_location(&mut self, path: ProjectDir) -> Result<Location> {
        self.ensure_project(&path)?;
        Ok(self.storage_location.clone())
    }

    fn create_task(
        &mut self,
        path: crate::interface::ProjectDir,
        task: crate::repr::Task,
    ) -> Result<()> {
        self.ensure_project(&path)?;

        if task.todo {
            self.project.todo.insert(
                task.name,
                Task {
                    priority: task.priority,
                    difficulty: task.difficulty,
                },
            );
        } else {
            self.project.done.insert(
                task.name,
                Task {
                    priority: task.priority,
                    difficulty: task.difficulty,
                },
            );
        }
        return Ok(());
    }

    fn delete_task(&mut self, _: ProjectDir) -> Result<()> {
        todo!()
    }

    fn delete_project(&mut self, _: ProjectDir) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusInstance {
    storage_location: Location,
    project_location: Option<Location>,

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

use crate::repr;

impl StatusCluster {
    fn root_path(path: &ProjectDir) -> Result<ProjectDir> {
        ensure!(path.len() >= 1, "path must contain at least one segment");
        ensure!(
            !path.get_section(0).is_task(),
            "path root segment must be a project"
        );

        Ok(ProjectDir {
            vec: vec![crate::interface::PathSegment::project(
                path.get_section(0).get_name(),
            )],
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

    fn commit_changes(&mut self) -> Result<()> {
        for instance in self.instances.values_mut() {
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
        if self.instances.contains_key(&path) {
            bail!("project already exists at given path");
        }

        let project_location = project.location;
        let todo_tasks = project.tasks.iter().filter(|task| task.todo);
        let done_tasks = project.tasks.iter().filter(|task| task.todo);
        let db_project = Project {
            project: ProjectHeader {
                version: project.version,
                edition: project.edition,
                name: project.name,
                description: project.description,
                subprojects: Vec::new(),
            },
            todo: todo_tasks
                .map(|task| {
                    (
                        task.name.clone(),
                        Task {
                            priority: task.priority,
                            difficulty: task.difficulty,
                        },
                    )
                })
                .collect(),
            done: done_tasks
                .map(|task| {
                    (
                        task.name.clone(),
                        Task {
                            priority: task.priority,
                            difficulty: task.difficulty,
                        },
                    )
                })
                .collect(),
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

    fn create_task(
        &mut self,
        path: crate::interface::ProjectDir,
        task: crate::repr::Task,
    ) -> Result<()> {
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
}
