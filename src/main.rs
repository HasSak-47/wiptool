pub mod cli;
pub mod dbs;
pub mod inner_log;
pub mod interface;
pub mod repr;
pub mod version;

use anyhow::Result;

fn main() -> Result<()> {
    inner_log::init_log();
    cli::run()
}
