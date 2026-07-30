use std::path::PathBuf;

use clap::Parser;

/**
makes the task into a project with a parent project
*/
#[derive(Parser, Clone)]
pub(crate) struct PromoteTask {
    name: String,
    new_path: PathBuf,
}
