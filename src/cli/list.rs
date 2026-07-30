use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::{cli::Opts, interface::ProjectStorage};

/**
list all the projects
*/
#[derive(Parser, Clone)]
pub(crate) struct List {
    #[arg(short, long)]
    color: bool,
    #[arg(short, long)]
    location: bool,

    #[arg(short, long)]
    status: bool,
}

impl List {
    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let paths = storage.get_projects_path()?;
        let home_dir = dirs::home_dir();

        for path in paths {
            log::debug!("getting path: {path:?}",);
            let project = storage.get_project(path.clone())?;

            let status_string = || {
                let done_difficulty = project.get_done_difficulty();
                let todo_difficulty = project.get_todo_difficulty();
                let overall_difficulty = done_difficulty + todo_difficulty;
                let completion = if overall_difficulty == 0. {
                    1.
                } else {
                    done_difficulty / overall_difficulty
                };

                format!("completion: {:.1}% | difficulty: {overall_difficulty:.2} | todo difficulty: {todo_difficulty}", completion * 100.)
            };

            let location_string = || {
                format!(
                    "\x1b[1;34m{}\x1b[0m",
                    match &project.location {
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
                            format!("{display_location}")
                        }
                        Some(crate::repr::Location::URL(project_location)) => {
                            format!("{project_location}")
                        }
                        None => {
                            format!("<unknown>")
                        }
                    }
                )
            };

            print!("{path}");
            if self.location {
                print!(" @ {}", location_string());
            }
            if self.status {
                print!(" | {}", status_string());
            }
            println!()
        }

        Ok(())
    }
}
