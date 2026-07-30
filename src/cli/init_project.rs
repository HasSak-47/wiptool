use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Clone)]
pub(crate) struct InitProject {
    name: String,
    location: PathBuf,
}
