use clap::Parser;

#[derive(Parser, Clone)]
pub(crate) struct SetSubproject {
    parent: String,
    child: String,
}
