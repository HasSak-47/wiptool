use std::path::PathBuf;

use clap::Parser;

/**
starts tracking a project with an existing status.toml
*/
#[derive(Parser, Clone)]
pub(crate) struct InitProject {
    name: String,
    location: PathBuf,
}
