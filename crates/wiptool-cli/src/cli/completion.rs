use std::io;

use clap::Parser;
use clap_complete::Shell;

use super::Cli;

/**
generate shell completions
*/
#[derive(Parser, Clone)]
pub(crate) struct Completion {
    shell: Shell,
}

impl Completion {
    pub fn run(&self) {
        let mut command = <Cli as clap::CommandFactory>::command();
        let name = command.get_name().to_string();

        clap_complete::generate(self.shell, &mut command, name, &mut io::stdout());
    }
}
