use std::fmt::Display;

use super::repr::*;

use anyhow::{self, bail, ensure, Result};
use serde::de::Error as DeError;
use serde::ser::Error as SerError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PathSegment {
    name: String,
    is_task: bool,
}

impl PathSegment {
    pub fn task(name: String) -> Self {
        return Self {
            name,
            is_task: true,
        };
    }

    pub fn project(name: String) -> Self {
        return Self {
            name,
            is_task: false,
        };
    }
    pub fn is_task(&self) -> bool {
        return self.is_task;
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }
}

/**
uri of the project
Path: (project_name/)+(task_name)?
*/
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ProjectDir {
    pub vec: Vec<PathSegment>,
}

impl Display for ProjectDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in &self.vec {
            if !segment.is_task {
                write!(f, "{}/", segment.name)?;
            } else {
                write!(f, "{}", segment.name)?;
            }
        }
        return Ok(());
    }
}

impl TryFrom<&str> for ProjectDir {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let value = value.trim();
        ensure!(!value.is_empty(), "path is empty");

        let parts: Vec<&str> = value.split('/').collect();
        ensure!(!parts.is_empty(), "path is empty");

        let ends_with_slash = parts.last().copied() == Some("");
        for (i, p) in parts.iter().enumerate() {
            if p.is_empty() {
                if !(ends_with_slash && i + 1 == parts.len()) {
                    bail!("invalid path: empty segment (leading or consecutive '/')");
                }
            }
        }

        let mut nonempty: Vec<&str> = parts.into_iter().filter(|p| !p.is_empty()).collect();
        ensure!(!nonempty.is_empty(), "invalid path: no segments");

        let (project_parts, task_part) = if ends_with_slash {
            (nonempty.as_slice(), None)
        } else {
            ensure!(
                nonempty.len() >= 2,
                "invalid path: needs at least one project and a task (or end with '/')"
            );
            let task = nonempty.pop().unwrap();
            (nonempty.as_slice(), Some(task))
        };

        ensure!(
            !project_parts.is_empty(),
            "invalid path: must contain at least one project"
        );

        fn validate_name(kind: &str, s: &str) -> Result<()> {
            ensure!(!s.trim().is_empty(), "{kind} name is empty");
            ensure!(
                !s.chars().any(|c| c.is_whitespace() || c == '/'),
                "{kind} name contains invalid characters"
            );
            Ok(())
        }

        let mut vec = Vec::with_capacity(project_parts.len() + task_part.is_some() as usize);

        for p in project_parts {
            validate_name("project", p)?;
            vec.push(PathSegment::project((*p).to_string()));
        }

        if let Some(t) = task_part {
            validate_name("task", t)?;
            vec.push(PathSegment::task(t.to_string()));
        }

        return Ok(ProjectDir { vec });
    }
}

impl ProjectDir {
    pub fn new() -> Self {
        return ProjectDir { vec: Vec::new() };
    }

    pub fn len(&self) -> usize {
        return self.vec.len();
    }

    pub fn parse<S: AsRef<str>>(s: S) -> Result<Self> {
        return ProjectDir::try_from(s.as_ref());
    }

    pub fn add_task<S: Into<String>>(&mut self, name: S) -> Result<()> {
        let name = name.into();
        if let Some(last) = self.vec.last_mut() {
            if last.is_task() {
                anyhow::bail!("Path already at a task, cannot add a task")
            }
        }

        self.vec.push(PathSegment::task(name));

        Ok(())
    }

    pub fn add_project(&mut self, name: String) -> Result<()> {
        if let Some(last) = self.vec.last_mut() {
            if last.is_task() {
                anyhow::bail!("Path already at a task, cannot add a project")
            }
        }

        self.vec.push(PathSegment::project(name));

        Ok(())
    }

    pub fn remove_task(&mut self) -> Result<()> {
        let last = self.vec.last().ok_or(anyhow::anyhow!("no task in path"))?;

        if last.is_task {
            self.vec.pop();
        }

        return Ok(());
    }

    fn to_path_string(&self) -> Result<String> {
        ensure!(!self.vec.is_empty(), "path has no segments");

        let mut parts = Vec::with_capacity(self.vec.len());
        for (i, segment) in self.vec.iter().enumerate() {
            let is_last = i + 1 == self.vec.len();
            if segment.is_task() && !is_last {
                bail!("invalid path: task segment can only appear at the end");
            }
            parts.push(segment.name.as_str());
        }

        let mut s = parts.join("/");
        if !self.vec.last().expect("checked non-empty").is_task() {
            s.push('/');
        }
        Ok(s)
    }

    pub fn get_section_mut(&mut self, idx: usize) -> &mut PathSegment {
        return &mut self.vec[idx];
    }

    pub fn get_section(&self, idx: usize) -> &PathSegment {
        return &self.vec[idx];
    }
}

impl Serialize for ProjectDir {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_path_string().map_err(SerError::custom)?;
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for ProjectDir {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ProjectDir::parse(s).map_err(D::Error::custom)
    }
}

pub trait ProjectStorage {
    fn task_exists(&mut self, path: ProjectDir) -> Result<bool>;
    fn project_exists(&mut self, path: ProjectDir) -> Result<bool>;

    fn get_projects_path(&mut self) -> Result<Vec<ProjectDir>>;
    fn get_project(&mut self, path: ProjectDir) -> Result<Project>;
    fn get_storage_location(&mut self, path: ProjectDir) -> Result<Location>;
    fn promote_task(&mut self, path: ProjectDir) -> Result<()>;
    fn get_task(&mut self, path: ProjectDir) -> Result<Task>;

    /** saves data */
    fn commit_changes(&mut self) -> Result<()>;

    /** creates or overrides project data */
    fn create_project(
        &mut self,
        path: ProjectDir,
        project: Project,
        storage_location: Location,
    ) -> Result<()>;

    /** add todo task */
    fn insert_task_todo(&mut self, path: ProjectDir, task: Task) -> Result<()>;
    /** add done task */
    fn insert_task_done(&mut self, path: ProjectDir, task: Task) -> Result<()>;
    /** marks task as done */
    fn mark_done_task(&mut self, path: ProjectDir) -> Result<()>;
    /** marks task as todo */
    fn mark_todo_task(&mut self, path: ProjectDir) -> Result<()>;

    /** marks task as todo */
    fn delete_project(&mut self, path: ProjectDir) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::ProjectDir;

    #[test]
    fn path_deserializes_from_string() {
        let path: ProjectDir = serde_json::from_str("\"root/sub/task\"").expect("valid path");
        assert_eq!(
            path.vec[0].get_name(),
            "root",
            "first part is not named root",
        );
        assert_eq!(
            path.vec[1].get_name(),
            "sub",
            "first part is not named root",
        );
        assert_eq!(
            path.vec[2].get_name(),
            "task",
            "first part is not named root",
        );

        assert!(!path.vec[0].is_task(), "first part is not project");
        assert!(!path.vec[1].is_task(), "first part is not project");
        assert!(path.vec[2].is_task(), "first part is not task");
    }

    #[test]
    fn project_only_path_serializes_with_trailing_slash() {
        let path = ProjectDir::parse("root/sub/").expect("valid project path");
        assert_eq!(
            path.vec[0].get_name(),
            "root",
            "first part is not named root",
        );
        assert_eq!(
            path.vec[1].get_name(),
            "sub",
            "first part is not named root",
        );

        assert!(!path.vec[0].is_task(), "first part is not project");
        assert!(!path.vec[1].is_task(), "first part is not project");
    }
}
