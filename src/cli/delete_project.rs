use anyhow::Result;
use clap::Parser;

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr::Location,
};

#[derive(Parser, Clone)]
pub(crate) struct DeleteProject {
    name: String,

    #[arg(long = "soft-delete", default_value_t = false)]
    soft: bool,
}

impl DeleteProject {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let loc = ProjectDir::parse(&format!("{}/", self.name))?;
        log::debug!("deleting project: {}", loc);
        if !self.soft {
            if let Location::Local(path) = storage.get_storage_location(loc.clone())? {
                log::debug!("deleting toml path: {}", path.display());
                if let Err(e) = std::fs::remove_file(path) {
                    log::error!("{}", e);
                }
            }
        }
        storage.delete_project(loc)?;
        storage.commit_changes()?;

        Ok(())
    }
}
