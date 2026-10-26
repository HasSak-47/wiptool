use wiptool_core::{repr, Location, PathSegment, ProjectDir, ProjectStorage, Status, Version};

use anyhow::{anyhow, bail, ensure, Result};
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TomlProjectHeader {
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
pub struct TomlTask {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_estimate: Option<u64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TomlMilestone {
    pub name: String,
    pub status: Status,
    pub version: Option<Version>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub todo: HashMap<String, TomlTask>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub done: HashMap<String, TomlTask>,
}

// status project repr
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TomlProject {
    pub project: TomlProjectHeader,

    pub milestones: Vec<TomlMilestone>,
}

/**
represents a file at location that contains a toml describing the project
idealy it should be a status.toml at the root of the project dir
it also contains the project
*/
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusDB {
    pub project: TomlProject,
    pub storage_location: Location,
}

impl StatusDB {
    pub const DEFAULT_MILESTONE: &'static str = "default";

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
                log::debug!("StatusDB loading toml file at: {}", path.display());
                let mut file = File::open(path)?;
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                buf
            }
            Location::URL(_) => bail!("loading project data from URL is not supported yet"),
        };

        let project: TomlProject = toml::from_str(string.as_str())?;

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

    // return milestone task?
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

    fn milestone(&self, name: &str) -> Result<&TomlMilestone> {
        self.project
            .milestones
            .iter()
            .find(|milestone| milestone.name == name)
            .ok_or_else(|| anyhow!("milestone not found: {name}"))
    }

    fn milestone_mut(&mut self, name: &str) -> Result<&mut TomlMilestone> {
        self.project
            .milestones
            .iter_mut()
            .find(|milestone| milestone.name == name)
            .ok_or_else(|| anyhow!("milestone not found: {name}"))
    }

    fn ensure_milestone_mut(&mut self, name: &str) -> &mut TomlMilestone {
        if let Some(idx) = self
            .project
            .milestones
            .iter()
            .position(|milestone| milestone.name == name)
        {
            return &mut self.project.milestones[idx];
        }

        self.project.milestones.push(TomlMilestone {
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
            vec: vec![PathSegment::project(self.project.project.name.clone())],
        }]);
    }

    fn promote_task(&mut self, _: ProjectDir) -> anyhow::Result<()> {
        bail!("promoting not available for toml database");
    }

    fn get_project(&mut self, path: ProjectDir) -> Result<repr::Project> {
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
                        repr::Task {
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
                        repr::Task {
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
                repr::Milestone {
                    status: milestone.status.clone(),
                    version: milestone.version.clone(),
                    tasks,
                },
            );
        }

        return Ok(repr::Project {
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

    fn get_task(&mut self, path: ProjectDir) -> Result<repr::Task> {
        let (milestone_name, task_name) = self.path_milestone_and_task(&path)?;
        let task_name = task_name.ok_or_else(|| anyhow!("path does not point to a task"))?;
        let milestone = self.milestone(&milestone_name)?;

        if let Some(task) = milestone.todo.get(&task_name) {
            return Ok(repr::Task {
                todo: true,
                priority: task.priority,
                difficulty: task.difficulty,
                time_estimate: task.time_estimate,
            });
        }

        if let Some(task) = milestone.done.get(&task_name) {
            return Ok(repr::Task {
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

    fn create_milestone(&mut self, path: ProjectDir, milestone: repr::Milestone) -> Result<()> {
        let (milestone_name, task_name) = self.path_milestone_and_task(&path)?;
        ensure!(
            task_name.is_none(),
            "path must point to a milestone or project"
        );

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

        self.project.milestones.push(TomlMilestone {
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

    fn get_milestones(&mut self, path: ProjectDir) -> Result<Vec<(String, repr::Milestone)>> {
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
                            repr::Task {
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
                            repr::Task {
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
                    repr::Milestone {
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

        log::debug!("StatusDB:: data saved {self:#?}");
        let mut file = File::create(path)?;
        let buf = toml::to_string_pretty(&self.project)?;
        file.write_all(buf.as_bytes())?;
        log::debug!("StatusDB:: saved data to {}", path.display());

        return Ok(());
    }

    fn create_project(&mut self, _: ProjectDir, _: repr::Project, _: Location) -> Result<()> {
        bail!("Creating project not available for Basic TOML DB")
    }

    fn get_storage_location(&mut self, path: ProjectDir) -> Result<Location> {
        self.ensure_project(&path)?;
        Ok(self.storage_location.clone())
    }

    fn create_task(&mut self, path: ProjectDir, task: repr::Task) -> Result<()> {
        let (milestone_name, task_name) = self.path_milestone_and_task(&path)?;
        let task_name = task_name.ok_or_else(|| anyhow!("path does not point to a task"))?;
        let milestone = self.ensure_milestone_mut(&milestone_name);

        if task.todo {
            milestone.done.remove(&task_name);
            milestone.todo.insert(
                task_name,
                TomlTask {
                    priority: task.priority,
                    difficulty: task.difficulty,
                    time_estimate: task.time_estimate,
                },
            );
        } else {
            milestone.todo.remove(&task_name);
            milestone.done.insert(
                task_name,
                TomlTask {
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
        unreachable!()
    }

    fn find_project_at_location(&self, _: Location) -> Result<ProjectDir> {
        unreachable!()
    }
}
