use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::{cli::Opts, interface::ProjectStorage};

#[derive(Parser, Clone)]
pub(crate) struct List {
    #[arg(short, long)]
    color: bool,
    #[arg(short, long)]
    location: bool,
}

impl List {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let paths = storage.get_projects_path()?;
        let home_dir = dirs::home_dir();

        if self.location {
            for path in paths {
                log::debug!("getting path: {path:?}",);
                let project = storage.get_project(path.clone())?;
                match project.location {
                    Some(crate::repr::Location::Local(project_location)) => {
                        let display_location = if let Some(home) = &home_dir {
                            match project_location.strip_prefix(home) {
                                Ok(relative) => {
                                    if relative.as_os_str().is_empty() {
                                        String::from("~")
                                    } else {
                                        PathBuf::from("~").join(relative).display().to_string()
                                    }
                                }
                                Err(_) => project_location.display().to_string(),
                            }
                        } else {
                            project_location.display().to_string()
                        };

                        if self.color {
                            println!("{path} @ \x1b[1;34m{display_location}\x1b[0m");
                        } else {
                            println!("{path} @ {display_location}");
                        }
                    }
                    Some(crate::repr::Location::URL(project_location)) => {
                        println!("{path} @ {project_location}");
                    }
                    None => {
                        println!("{path} @ <unknown>");
                    }
                }
            }
        } else {
            for path in paths {
                println!("{path}",);
            }
        }

        Ok(())
    }
}
