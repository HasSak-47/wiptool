use std::env::current_dir;

use anyhow::{bail, Result};
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr::{Location, Status},
};

/**
sets milestone status
*/
#[derive(Parser, Debug, Clone)]
pub(crate) struct SetStatus {
    status: Status,
    path: Option<String>,
}

impl SetStatus {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        log::debug!("{self:?}");
        let path = if let Some(p) = &self.path {
            ProjectDir::parse(p.clone() + if p.ends_with("/") { "" } else { "/" })?
        } else {
            let cwd = current_dir()?;
            let mut p = None;

            for path in storage.get_projects_path()? {
                let project = storage.get_project(path.clone())?;
                let location = project.location.clone().unwrap();
                if let Location::Local(project_path) = &location {
                    if cwd == *project_path {
                        p = Some(path);
                        break;
                    }
                }
            }
            if let None = p {
                bail!("there is no project in cwd: {}", cwd.display());
            }
            p.unwrap()
        };

        storage.set_status(path, self.status.clone())?;
        storage.commit_changes()?;

        Ok(())
    }
}
