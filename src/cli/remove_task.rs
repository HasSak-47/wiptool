use clap::Parser;

#[derive(Parser, Clone)]
pub(crate) struct RemoveTask {
    name: String,
}
