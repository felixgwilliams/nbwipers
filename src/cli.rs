use std::path::PathBuf;

use clap::{command, Parser};

use crate::wipers::ExtraKey;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    pub notebook: PathBuf,

    #[arg(long, value_delimiter = ',')]
    pub extra_keys: Option<Vec<ExtraKey>>,
}
