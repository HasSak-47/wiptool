use clap::Parser;

/**
adds subproject to project
*/
#[derive(Parser, Clone)]
pub(crate) struct SetSubproject {
    parent: String,
    child: String,
}
