use std::env::current_dir;

use anyhow::{bail, Result};
use clap::Parser;

use crate::cli::Opts;
use wiptool_core::{Location, ProjectDir, ProjectStorage};

/// Renames a task while preserving its state and metadata.
#[derive(Parser, Clone)]
pub(crate) struct RenameTask {
    name: String,
    new_name: String,

    #[arg(short, long)]
    milestone: Option<String>,

    project: Option<String>,
}

impl RenameTask {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let mut path = if let Some(project) = &self.project {
            ProjectDir::parse(format!("{}/", project.trim_end_matches('/')))?
        } else {
            let cwd = current_dir()?;
            let mut found = None;
            for path in storage.get_projects_path()? {
                if let Some(Location::Local(project_path)) =
                    storage.get_project(path.clone())?.location
                {
                    if cwd == project_path {
                        found = Some(path);
                        break;
                    }
                }
            }
            found.ok_or_else(|| anyhow::anyhow!("there is no project in cwd: {}", cwd.display()))?
        };

        if let Some(milestone) = &self.milestone {
            if path.vec.iter().any(|segment| segment.is_milestone()) {
                bail!("milestone specified both in path and with --milestone");
            }
            path.add_milestone(milestone)?;
        }
        path.add_task(&self.name)?;
        storage.rename_task(path, self.new_name.clone())?;
        storage.commit_changes()
    }
}
