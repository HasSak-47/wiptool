use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueEnum};

use crate::{
    cli::Opts,
    interface::{ProjectDir, ProjectStorage},
    repr::{Project, Status},
};

#[derive(ValueEnum, Default, Clone, Debug)]
enum SortBy {
    #[default]
    Name,
    Completion,
    Category,
}

/**
list all the projects
*/
#[derive(Parser, Clone)]
pub(crate) struct ProjectList {
    #[arg(short, long)]
    color: bool,

    #[arg(short, long)]
    location: bool,

    #[arg(short, long)]
    state: bool,

    #[arg(long)]
    sort_by: Option<SortBy>,

    #[arg(long, value_delimiter = ',', default_value = "")]
    kinds: Vec<String>,

    #[arg(long)]
    status: Option<Status>,

    #[arg(long, conflicts_with = "status")]
    exact_status: Option<Status>,
}

impl ProjectList {
    fn generate_state_string(project: &Project) -> String {
        let done_difficulty = project.get_done_difficulty();
        let todo_difficulty = project.get_todo_difficulty();
        let overall_difficulty = done_difficulty + todo_difficulty;
        let completion = if overall_difficulty == 0. {
            1.
        } else {
            done_difficulty / overall_difficulty
        };

        return format!(
            "completion: {:.1}% | dif: {overall_difficulty:.2} | todo dif: {todo_difficulty:.2}",
            completion * 100.
        );
    }

    fn generate_location_string(project: &Project, home_dir: &Option<PathBuf>) -> String {
        return format!(
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
        );
    }

    fn compare_todo(
        project_a: &(&ProjectDir, Project),
        project_b: &(&ProjectDir, Project),
    ) -> std::cmp::Ordering {
        return project_a
            .1
            .get_difficulty_completion()
            .total_cmp(&project_b.1.get_difficulty_completion());
    }

    fn compare_category(
        project_a: &(&ProjectDir, Project),
        project_b: &(&ProjectDir, Project),
    ) -> std::cmp::Ordering {
        let mut a = project_a.1.kinds.clone();
        a.sort();
        let mut b = project_b.1.kinds.clone();
        b.sort();

        return a.first().unwrap().cmp(b.first().unwrap());
    }

    fn compare_name(
        project_a: &(&ProjectDir, Project),
        project_b: &(&ProjectDir, Project),
    ) -> std::cmp::Ordering {
        let a = &project_a.1.name;
        let b = &project_b.1.name;

        return a.cmp(b);
    }

    pub fn run(&self, _: &Opts, storage: &mut Box<dyn ProjectStorage>) -> Result<()> {
        let paths = storage.get_projects_path()?;
        let home_dir = dirs::home_dir();

        let mut projects: Vec<_> = paths
            .iter()
            .filter_map(|path| {
                log::debug!("getting path: {path:?}",);
                let project = storage.get_project(path.to_owned().to_owned()).ok()?;
                let project_status = &project
                    .milestones
                    .get(&project.current_milestone)
                    .unwrap()
                    .status;

                if let Some(s) = &self.exact_status {
                    if *s != *project_status {
                        return None;
                    }
                }

                if let Some(s) = &self.status {
                    if !project_status.aproximate(s) {
                        return None;
                    }
                }

                return Some((path, project));
            })
            .collect();

        if let Some(sort) = &self.sort_by {
            match sort {
                SortBy::Name => projects.sort_by(ProjectList::compare_name),
                SortBy::Completion => projects.sort_by(ProjectList::compare_todo),
                SortBy::Category => projects.sort_by(ProjectList::compare_category),
            }
        }

        for (path, project) in projects {
            print!("{path}");
            if self.location {
                print!(
                    " @ {}",
                    ProjectList::generate_location_string(&project, &home_dir)
                );
            }
            if self.state {
                print!(" | {}", ProjectList::generate_state_string(&project));
            }
            println!()
        }

        Ok(())
    }
}
