use crate::{
    interface::{ProjectDir, ProjectStorage},
    repr::{Location, Status},
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
    pub name: String,
    pub edition: Version,

    pub milestone: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kinds: Vec<String>,

    pub description: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subprojects: Vec<ProjectDir>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Task {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_estimate: Option<u64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub status: Status,
    pub version: Option<Version>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub todo: HashMap<String, Task>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub done: HashMap<String, Task>,
}

// status project repr
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Project {
    pub project: ProjectHeader,

    pub milestones: Vec<Milestone>,
}

/**
represents a file at location that contains a toml describing the project
idealy it should be a status.toml at the root of the project dir
it also contains the project
*/
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusDB {
    pub project: Project,
    pub storage_location: Location,
}

impl StatusDB {
    const DEFAULT_MILESTONE: &'static str = "default";

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

    fn current_milestone_name(&self) -> String {
        self.project
            .project
            .milestone
            .clone()
            .or_else(|| self.project.milestones.first().map(|m| m.name.clone()))
            .unwrap_or_else(|| Self::DEFAULT_MILESTONE.to_string())
    }

    fn path_milestone_and_task(&self, path: &ProjectDir) -> Result<(String, Option<String>)> {
        self.ensure_project(path)?;

        if path.len() == 1 {
            return Ok((self.current_milestone_name(), None));
        }

        let mut idx = 1;
        let milestone = if path.get_section(idx).is_milestone() {
            let milestone = path.get_section(idx).get_name();
            idx += 1;
            milestone
        } else {
            self.current_milestone_name()
        };

        if idx == path.len() {
            return Ok((milestone, None));
        }

        ensure!(
            path.get_section(idx).is_task(),
            "subprojects are not handled by the basic TOML database yet"
        );
        ensure!(
            idx + 1 == path.len(),
            "task segment can only appear at the end"
        );

        Ok((milestone, Some(path.get_section(idx).get_name())))
    }

    fn milestone(&self, name: &str) -> Result<&Milestone> {
        self.project
            .milestones
            .iter()
            .find(|milestone| milestone.name == name)
            .ok_or_else(|| anyhow!("milestone not found: {name}"))
    }

    fn milestone_mut(&mut self, name: &str) -> Result<&mut Milestone> {
        self.project
            .milestones
            .iter_mut()
            .find(|milestone| milestone.name == name)
            .ok_or_else(|| anyhow!("milestone not found: {name}"))
    }

    fn ensure_milestone_mut(&mut self, name: &str) -> &mut Milestone {
        if let Some(idx) = self
            .project
            .milestones
            .iter()
            .position(|milestone| milestone.name == name)
        {
            return &mut self.project.milestones[idx];
        }

        self.project.milestones.push(Milestone {
            name: name.to_string(),
            status: Status::default(),
            version: None,
            todo: HashMap::new(),
            done: HashMap::new(),
        });

        self.project
            .milestones
            .last_mut()
            .expect("milestone inserted above")
    }
}

impl ProjectStorage for StatusDB {
    fn project_exists(&mut self, path: ProjectDir) -> Result<bool> {
        self.ensure_project(&path)?;
        ensure!(path.len() == 1);

        return Ok(path.get_section(0).get_name() == self.project.project.name);
    }

    fn task_exists(&mut self, path: ProjectDir) -> Result<bool> {
        let (milestone_name, task_name) = self.path_milestone_and_task(&path)?;
        let task_name = task_name.ok_or_else(|| anyhow!("path does not point to a task"))?;
        let milestone = self.milestone(&milestone_name)?;

        Ok(milestone.todo.contains_key(&task_name) || milestone.done.contains_key(&task_name))
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

        let current_milestone = self.current_milestone_name();
        let mut milestones = HashMap::new();

        for milestone in &self.project.milestones {
            let tasks = milestone
                .todo
                .iter()
                .map(|(name, task)| {
                    (
                        name.clone(),
                        crate::repr::Task {
                            todo: true,
                            priority: task.priority,
                            difficulty: task.difficulty,
                            time_estimate: task.time_estimate,
                        },
                    )
                })
                .chain(milestone.done.iter().map(|(name, task)| {
                    (
                        name.clone(),
                        crate::repr::Task {
                            todo: false,
                            priority: task.priority,
                            difficulty: task.difficulty,
                            time_estimate: task.time_estimate,
                        },
                    )
                }))
                .collect();

            milestones.insert(
                milestone.name.clone(),
                crate::repr::Milestone {
                    status: milestone.status.clone(),
                    version: milestone.version.clone(),
                    tasks,
                },
            );
        }

        return Ok(crate::repr::Project {
            name: self.project.project.name.clone(),
            edition: self.project.project.edition.clone(),
            current_milestone,
            kinds: self.project.project.kinds.clone(),
            description: self.project.project.description.clone(),
            location: None,
            subprojects: HashMap::new(),
            milestones,
        });
    }

    fn get_task(&mut self, path: crate::interface::ProjectDir) -> Result<crate::repr::Task> {
        let (milestone_name, task_name) = self.path_milestone_and_task(&path)?;
        let task_name = task_name.ok_or_else(|| anyhow!("path does not point to a task"))?;
        let milestone = self.milestone(&milestone_name)?;

        if let Some(task) = milestone.todo.get(&task_name) {
            return Ok(crate::repr::Task {
                todo: true,
                priority: task.priority,
                difficulty: task.difficulty,
                time_estimate: task.time_estimate,
            });
        }

        if let Some(task) = milestone.done.get(&task_name) {
            return Ok(crate::repr::Task {
                priority: task.priority,
                todo: false,
                difficulty: task.difficulty,
                time_estimate: task.time_estimate,
            });
        }

        bail!("no exisiting task")
    }

    fn rename_project(&mut self, path: ProjectDir, new_name: String) -> Result<()> {
        self.ensure_project(&path)?;
        ensure!(path.len() == 1, "renaming subprojects is not supported");
        ProjectDir::parse(format!("{new_name}/"))?;
        self.project.project.name = new_name;
        Ok(())
    }

    fn rename_milestone(&mut self, path: ProjectDir, new_name: String) -> Result<()> {
        let (old_name, task_name) = self.path_milestone_and_task(&path)?;
        ensure!(task_name.is_none(), "path must point to a milestone");
        ensure!(
            path.vec.iter().any(|segment| segment.is_milestone()),
            "path must explicitly identify a milestone"
        );
        ProjectDir::parse(format!("{}/@{new_name}/", self.project.project.name))?;

        if old_name == new_name {
            self.milestone(&old_name)?;
            return Ok(());
        }
        ensure!(
            self.project
                .milestones
                .iter()
                .all(|milestone| milestone.name != new_name),
            "milestone already exists: {new_name}"
        );

        self.milestone_mut(&old_name)?.name = new_name.clone();
        if self.project.project.milestone.as_deref() == Some(&old_name) {
            self.project.project.milestone = Some(new_name);
        }
        Ok(())
    }

    fn rename_task(&mut self, path: ProjectDir, new_name: String) -> Result<()> {
        let (milestone_name, old_name) = self.path_milestone_and_task(&path)?;
        let old_name = old_name.ok_or_else(|| anyhow!("path does not point to a task"))?;
        ProjectDir::parse(format!(
            "{}/@{milestone_name}/{new_name}",
            self.project.project.name
        ))?;

        let milestone = self.milestone_mut(&milestone_name)?;
        if old_name == new_name {
            ensure!(
                milestone.todo.contains_key(&old_name) || milestone.done.contains_key(&old_name),
                "task not found: {old_name}"
            );
            return Ok(());
        }
        ensure!(
            !milestone.todo.contains_key(&new_name) && !milestone.done.contains_key(&new_name),
            "task already exists: {new_name}"
        );

        if let Some(task) = milestone.todo.remove(&old_name) {
            milestone.todo.insert(new_name, task);
        } else if let Some(task) = milestone.done.remove(&old_name) {
            milestone.done.insert(new_name, task);
        } else {
            bail!("task not found: {old_name}");
        }
        Ok(())
    }

    fn create_milestone(
        &mut self,
        path: ProjectDir,
        milestone: crate::repr::Milestone,
    ) -> Result<()> {
        let (milestone_name, task_name) = self.path_milestone_and_task(&path)?;
        ensure!(
            task_name.is_none(),
            "path must point to a milestone or project"
        );

        let mut todo = HashMap::new();
        let mut done = HashMap::new();
        for (task_name, task) in milestone.tasks {
            let db_task = Task {
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

        self.project.milestones.push(Milestone {
            name: milestone_name.clone(),
            status: milestone.status,
            version: milestone.version,
            todo,
            done,
        });

        if self.project.project.milestone.is_none() {
            self.project.project.milestone = Some(milestone_name);
        }

        Ok(())
    }

    fn get_milestones(
        &mut self,
        path: ProjectDir,
    ) -> Result<Vec<(String, crate::repr::Milestone)>> {
        self.ensure_project(&path)?;
        ensure!(
            path.len() == 1,
            "listing milestones for subprojects is not handled yet"
        );

        Ok(self
            .project
            .milestones
            .iter()
            .map(|milestone| {
                let tasks = milestone
                    .todo
                    .iter()
                    .map(|(name, task)| {
                        (
                            name.clone(),
                            crate::repr::Task {
                                todo: true,
                                priority: task.priority,
                                difficulty: task.difficulty,
                                time_estimate: task.time_estimate,
                            },
                        )
                    })
                    .chain(milestone.done.iter().map(|(name, task)| {
                        (
                            name.clone(),
                            crate::repr::Task {
                                todo: false,
                                priority: task.priority,
                                difficulty: task.difficulty,
                                time_estimate: task.time_estimate,
                            },
                        )
                    }))
                    .collect();

                (
                    milestone.name.clone(),
                    crate::repr::Milestone {
                        status: milestone.status.clone(),
                        version: milestone.version.clone(),
                        tasks,
                    },
                )
            })
            .collect())
    }

    fn set_current_milestone(&mut self, path: ProjectDir, milestone: String) -> Result<()> {
        self.ensure_project(&path)?;
        ensure!(
            path.len() == 1,
            "setting current milestone for subprojects is not handled yet"
        );
        self.milestone(&milestone)?;
        self.project.project.milestone = Some(milestone);

        Ok(())
    }

    fn set_status(&mut self, path: ProjectDir, status: Status) -> Result<()> {
        let (milestone_name, task_name) = self.path_milestone_and_task(&path)?;
        ensure!(task_name.is_none(), "task status is handled by mark-task");

        let milestone = self.milestone_mut(&milestone_name)?;
        milestone.status = status;

        Ok(())
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
        let (milestone_name, task_name) = self.path_milestone_and_task(&path)?;
        let task_name = task_name.ok_or_else(|| anyhow!("path does not point to a task"))?;
        let milestone = self.ensure_milestone_mut(&milestone_name);

        if task.todo {
            milestone.done.remove(&task_name);
            milestone.todo.insert(
                task_name,
                Task {
                    priority: task.priority,
                    difficulty: task.difficulty,
                    time_estimate: task.time_estimate,
                },
            );
        } else {
            milestone.todo.remove(&task_name);
            milestone.done.insert(
                task_name,
                Task {
                    priority: task.priority,
                    difficulty: task.difficulty,
                    time_estimate: task.time_estimate,
                },
            );
        }
        return Ok(());
    }

    fn delete_task(&mut self, path: ProjectDir) -> Result<()> {
        let (milestone_name, task_name) = self.path_milestone_and_task(&path)?;
        let task_name = task_name.ok_or_else(|| anyhow!("path does not point to a task"))?;
        let milestone = self.milestone_mut(&milestone_name)?;

        let removed = milestone.todo.remove(&task_name).is_some()
            || milestone.done.remove(&task_name).is_some();
        ensure!(removed, "task not found: {task_name}");

        Ok(())
    }

    fn delete_project(&mut self, _: ProjectDir) -> Result<()> {
        todo!()
    }
}

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

        let mut milestones: Vec<Milestone> = project
            .milestones
            .into_iter()
            .map(|(name, milestone)| {
                let mut todo = HashMap::new();
                let mut done = HashMap::new();

                for (task_name, task) in milestone.tasks {
                    let db_task = Task {
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

                Milestone {
                    name,
                    status: milestone.status,
                    version: milestone.version,
                    todo,
                    done,
                }
            })
            .collect();

        if milestones.is_empty() {
            milestones.push(Milestone {
                name: current_milestone.clone(),
                status: Status::default(),
                version: None,
                todo: HashMap::new(),
                done: HashMap::new(),
            });
        }

        let db_project = Project {
            project: ProjectHeader {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Milestone, Project, ProjectHeader, StatusDB, Task};
    use crate::{
        interface::{ProjectDir, ProjectStorage},
        repr::{Location, Status},
        version::Version,
    };

    #[test]
    fn task_metrics_are_optional_in_toml() {
        let task: Task = toml::from_str("").expect("empty task should deserialize");

        assert_eq!(task.priority, None);
        assert_eq!(task.difficulty, None);
        assert_eq!(task.time_estimate, None);
        assert_eq!(toml::to_string(&task).expect("task should serialize"), "");
    }

    #[test]
    fn existing_numeric_task_metrics_still_deserialize() {
        let task: Task = toml::from_str("priority = 2.5\ndifficulty = 3.0\ntime_estimate = 45\n")
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
            Task {
                priority: Some(2.0),
                difficulty: None,
                time_estimate: Some(30),
            },
        );
        let mut db = StatusDB {
            project: Project {
                project: ProjectHeader {
                    name: "old-project".to_string(),
                    edition: Version::default(),
                    milestone: Some("old-milestone".to_string()),
                    kinds: Vec::new(),
                    description: String::new(),
                    subprojects: Vec::new(),
                },
                milestones: vec![Milestone {
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
