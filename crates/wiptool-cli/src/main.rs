mod cli;
mod inner_log;

use anyhow::Result;

fn main() -> Result<()> {
    inner_log::init_log();
    cli::run()
}
