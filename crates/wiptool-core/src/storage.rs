use anyhow::Result;

use crate::{Location, Milestone, Project, ProjectDir, Status, Task};

pub trait ProjectStorage {
    fn task_exists(&mut self, path: ProjectDir) -> Result<bool>;
    fn project_exists(&mut self, path: ProjectDir) -> Result<bool>;

    fn get_projects_path(&mut self) -> Result<Vec<ProjectDir>>;
    fn get_project(&mut self, path: ProjectDir) -> Result<Project>;
    fn get_storage_location(&mut self, path: ProjectDir) -> Result<Location>;
    fn promote_task(&mut self, path: ProjectDir) -> Result<()>;
    fn get_task(&mut self, path: ProjectDir) -> Result<Task>;

    fn rename_project(&mut self, path: ProjectDir, new_name: String) -> Result<()>;
    fn rename_milestone(&mut self, path: ProjectDir, new_name: String) -> Result<()>;
    fn rename_task(&mut self, path: ProjectDir, new_name: String) -> Result<()>;

    /// Creates or replaces milestone data.
    fn create_milestone(&mut self, path: ProjectDir, milestone: Milestone) -> Result<()>;
    fn get_milestones(&mut self, path: ProjectDir) -> Result<Vec<(String, Milestone)>>;

    fn set_current_milestone(&mut self, path: ProjectDir, milestone: String) -> Result<()>;
    fn set_status(&mut self, path: ProjectDir, status: Status) -> Result<()>;
    fn commit_changes(&mut self) -> Result<()>;

    /// Creates or replaces project data.
    fn create_project(
        &mut self,
        path: ProjectDir,
        project: Project,
        storage_location: Location,
    ) -> Result<()>;

    fn delete_project(&mut self, path: ProjectDir) -> Result<()>;

    /// Creates or replaces task data.
    fn create_task(&mut self, path: ProjectDir, task: Task) -> Result<()>;
    fn delete_task(&mut self, path: ProjectDir) -> Result<()>;

    fn find_project_at_location(&self, location: Location) -> Result<ProjectDir>;
}
