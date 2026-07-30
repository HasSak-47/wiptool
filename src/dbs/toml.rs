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
    pub location: Location,
}

impl StatusDB {
    pub fn ensure_project(&self, path: &ProjectDir) -> Result<()> {
        ensure!(
            self.project.project.name == path.get_section(0).get_name(),
            "you can only get a name of the root project name"
        );

        return Ok(());
    }

    pub fn new(location: Location) -> Result<Self> {
        let string = match &location {
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
            location: location,
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

            location: Some(self.location.clone()),
            name: self.project.project.name.clone(),
            description: self.project.project.description.clone(),
            subprojects: Vec::new(),
            todo: self
                .project
                .todo
                .iter()
                .map(|(k, v)| crate::repr::Task {
                    name: k.clone(),
                    priority: v.priority,
                    difficulty: v.difficulty,
                })
                .collect(),
            done: self
                .project
                .done
                .iter()
                .map(|(k, v)| crate::repr::Task {
                    name: k.clone(),
                    priority: v.priority,
                    difficulty: v.difficulty,
                })
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

        let task = if self.project.todo.contains_key(&task_name) {
            &self.project.todo[&task_name]
        } else if self.project.done.contains_key(&task_name) {
            &self.project.done[&task_name]
        } else {
            bail!("no exisiting task")
        };

        return Ok(crate::repr::Task {
            name: task_name,
            priority: task.priority,
            difficulty: task.difficulty,
        });
    }

    fn commit_changes(&mut self) -> Result<()> {
        let path = if let Location::Local(path) = &self.location {
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

    fn insert_task_done(
        &mut self,
        path: crate::interface::ProjectDir,
        task: crate::repr::Task,
    ) -> Result<()> {
        self.ensure_project(&path)?;

        self.project.done.insert(
            task.name,
            Task {
                priority: task.priority,
                difficulty: task.difficulty,
            },
        );
        return Ok(());
    }

    fn insert_task_todo(
        &mut self,
        path: crate::interface::ProjectDir,
        task: crate::repr::Task,
    ) -> Result<()> {
        self.ensure_project(&path)?;

        self.project.todo.insert(
            task.name,
            Task {
                priority: task.priority,
                difficulty: task.difficulty,
            },
        );
        return Ok(());
    }
    fn mark_done_task(&mut self, path: crate::interface::ProjectDir) -> Result<()> {
        self.ensure_project(&path)?;
        let name = path.get_section(1).get_name();
        let task = self.project.todo.remove(&name);
        if let Some(s) = task {
            self.project.done.insert(name, s);
        }

        return Ok(());
    }

    fn mark_todo_task(&mut self, path: crate::interface::ProjectDir) -> Result<()> {
        self.ensure_project(&path)?;
        let name = path.get_section(1).get_name();
        let task = self.project.todo.remove(&name);
        if let Some(s) = task {
            self.project.done.insert(name, s);
        }

        return Ok(());
    }
    fn get_project_location(&mut self, path: ProjectDir) -> Result<Location> {
        self.ensure_project(&path)?;
        return Ok(self.location.clone());
    }

    fn delete_project(&mut self, _: ProjectDir) -> Result<()> {
        todo!()
    }
}

#[derive(Debug)]
struct StatusInstance {
    location: Location,

    db: Option<StatusDB>,
}

/**
keeps track of all the status.toml databases
*/
#[derive(Debug, Default)]
pub struct StatusCluster {
    instances: HashMap<ProjectDir, StatusInstance>,
    db_path: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StatusClusterDB {
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    instances: HashMap<String, Location>,
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

    fn path_to_string(path: &ProjectDir) -> Result<String> {
        ensure!(path.len() >= 1, "path must contain at least one segment");

        let mut parts = Vec::with_capacity(path.len());
        for i in 0..path.len() {
            let section = path.get_section(i);
            ensure!(
                !(section.is_task() && i + 1 != path.len()),
                "invalid path: task segment can only appear at the end"
            );
            parts.push(section.get_name());
        }

        let mut s = parts.join("/");
        if !path.get_section(path.len() - 1).is_task() {
            s.push('/');
        }

        Ok(s)
    }

    pub fn save(&self) -> Result<()> {
        let mut instances = HashMap::with_capacity(self.instances.len());

        for (path, instance) in &self.instances {
            instances.insert(Self::path_to_string(path)?, instance.location.clone());
        }

        let data = StatusClusterDB { instances };
        let mut file = File::create(&self.db_path)?;
        let buf = toml::to_string_pretty(&data)?;
        file.write_all(buf.as_bytes())?;

        Ok(())
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let mut file = File::open(&path)?;
        let mut buf = String::new();

        file.read_to_string(&mut buf)?;
        let data: StatusClusterDB = toml::from_str(buf.as_str())?;
        return Ok(Self {
            db_path: path.as_ref().to_path_buf(),
            instances: data
                .instances
                .iter()
                .map(|(path, instance)| {
                    (
                        // WARN: this is bad lmao
                        ProjectDir::try_from(path.as_str()).unwrap(),
                        StatusInstance {
                            location: instance.clone(),
                            db: None,
                        },
                    )
                })
                .collect(),
        });
    }

    fn get_instance_db(&mut self, path: &ProjectDir) -> Result<&mut StatusDB> {
        let root = Self::root_path(path)?;
        let instance = self
            .instances
            .get_mut(&root)
            .ok_or(anyhow!("could not find project"))?;

        if instance.db.is_none() {
            instance.db = Some(StatusDB::new(instance.location.clone())?);
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
        self.get_instance_db(&path)?.get_project(path)
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
        location: Location,
    ) -> Result<()> {
        if self.instances.contains_key(&path) {
            bail!("project already exists at given path");
        }

        let db_project = Project {
            project: ProjectHeader {
                version: project.version,
                edition: project.edition,
                name: project.name,
                description: project.description,
                subprojects: Vec::new(),
            },
            todo: project
                .todo
                .into_iter()
                .map(|task| {
                    (
                        task.name,
                        Task {
                            priority: task.priority,
                            difficulty: task.difficulty,
                        },
                    )
                })
                .collect(),
            done: project
                .done
                .into_iter()
                .map(|task| {
                    (
                        task.name,
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
            location: location.clone(),
        };
        db.commit_changes()?;

        self.instances.insert(
            path,
            StatusInstance {
                location,
                db: Some(db),
            },
        );

        Ok(())
    }

    /* add todo task */
    fn insert_task_todo(&mut self, path: ProjectDir, task: repr::Task) -> Result<()> {
        log::debug!("inserting: {path}");
        self.get_instance_db(&path)?.insert_task_todo(path, task)
    }
    fn insert_task_done(&mut self, path: ProjectDir, task: repr::Task) -> Result<()> {
        self.get_instance_db(&path)?.insert_task_done(path, task)
    }
    /* makes task as done */
    fn mark_done_task(&mut self, path: ProjectDir) -> Result<()> {
        self.get_instance_db(&path)?.mark_done_task(path)
    }
    /* makes task as todo */
    fn mark_todo_task(&mut self, path: ProjectDir) -> Result<()> {
        self.get_instance_db(&path)?.mark_todo_task(path)
    }

    fn get_project_location(&mut self, path: ProjectDir) -> Result<Location> {
        return self
            .instances
            .get(&path)
            .ok_or(anyhow!("project not found"))
            .and_then(|p| Ok(p.location.clone()));
    }

    fn delete_project(&mut self, path: ProjectDir) -> Result<()> {
        self.instances.remove(&path);
        return Ok(());
    }
}
