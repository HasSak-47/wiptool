use clap::Parser;

#[derive(Parser, Clone)]
pub(crate) struct MarkTask {
    name: String,
    todo: bool,
}
