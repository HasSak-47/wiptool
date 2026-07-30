use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Clone)]
pub(crate) struct PromoteTask {
    name: String,
    new_path: PathBuf,
}
