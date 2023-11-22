use std::path::PathBuf;

use clap::{command, Parser, Subcommand, ValueEnum};

use crate::{config::Configuration, extra_keys::ExtraKey};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Debug, Clone)]
pub struct CommonArgs {
    #[arg(long, short)]
    pub config: Option<PathBuf>,

    #[arg(long, value_delimiter = ',')]
    pub extra_keys: Option<Vec<ExtraKey>>,

    #[arg(long, conflicts_with = "keep_empty_cells")]
    pub drop_empty_cells: bool,

    #[arg(long, conflicts_with = "drop_empty_cells")]
    pub keep_empty_cells: bool,

    #[arg(long, conflicts_with = "keep_output")]
    pub drop_output: bool,

    #[arg(long, conflicts_with = "drop_output")]
    pub keep_output: bool,

    #[arg(long, conflicts_with = "keep_count")]
    pub drop_count: bool,

    #[arg(long, conflicts_with = "drop_count")]
    pub keep_count: bool,

    #[arg(long, conflicts_with = "keep_id")]
    pub drop_id: bool,

    #[arg(long, conflicts_with = "drop_id")]
    pub keep_id: bool,

    #[arg(long, conflicts_with = "keep_init_cells")]
    pub strip_init_cells: bool,

    #[arg(long, conflicts_with = "strip_init_cells")]
    pub keep_init_cells: bool,

    #[arg(long, value_delimiter = ',')]
    pub drop_tagged_cells: Option<Vec<String>>,

    #[arg(long, value_delimiter = ',')]
    pub keep_keys: Option<Vec<ExtraKey>>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Install(InstallCommand),
    CleanAll(CleanAllCommand),
    Check(CheckAllCommand),
    Clean(CleanCommand),
}

#[derive(Clone, Debug, Parser)]
pub struct CheckCommand {
    pub file: PathBuf,

    #[clap(flatten)]
    pub common: CommonArgs,
}
#[derive(Clone, Debug, Parser)]
pub struct CheckAllCommand {
    pub files: Vec<PathBuf>,

    #[clap(flatten)]
    pub common: CommonArgs,
}
#[derive(Clone, Debug, Parser)]
pub struct CleanCommand {
    pub file: PathBuf,

    #[arg(long, short)]
    pub textconv: bool,

    #[clap(flatten)]
    pub common: CommonArgs,
}
#[derive(Clone, Debug, Parser)]
pub struct CleanAllCommand {
    pub files: Vec<PathBuf>,

    #[clap(flatten)]
    pub common: CommonArgs,
}
#[derive(Clone, Debug, Parser)]
pub struct InstallCommand {
    pub config_type: GitConfigType,
    pub attribute_file: Option<PathBuf>,
}

#[derive(Clone, Debug, ValueEnum, Copy)]
pub enum GitConfigType {
    System,
    Global,
    Local,
}

pub struct ConfigOverrides {
    pub extra_keys: Option<Vec<ExtraKey>>,
    pub drop_empty_cells: Option<bool>,
    pub drop_output: Option<bool>,
    pub drop_count: Option<bool>,
    pub drop_id: Option<bool>,
    pub strip_init_cell: Option<bool>,
    pub drop_tagged_cells: Option<Vec<String>>,
    pub keep_keys: Option<Vec<ExtraKey>>,
}

pub struct Args {
    pub config: Option<PathBuf>,
}

fn resolve_bool_arg(yes: bool, no: bool) -> Option<bool> {
    match (yes, no) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
        (..) => unreachable!("Clap should make this impossible"),
    }
}

impl CommonArgs {
    pub fn partition(self) -> (Args, ConfigOverrides) {
        (
            Args {
                config: self.config,
            },
            ConfigOverrides {
                extra_keys: self.extra_keys,
                drop_empty_cells: resolve_bool_arg(self.drop_empty_cells, self.keep_empty_cells),
                drop_output: resolve_bool_arg(self.drop_output, self.keep_output),
                drop_count: resolve_bool_arg(self.drop_count, self.keep_count),
                drop_id: resolve_bool_arg(self.drop_id, self.keep_id),
                drop_tagged_cells: self.drop_tagged_cells,
                strip_init_cell: resolve_bool_arg(self.strip_init_cells, self.keep_init_cells),
                keep_keys: self.keep_keys,
            },
        )
    }
}

impl ConfigOverrides {
    pub fn override_config(&self, mut config: Configuration) -> Configuration {
        if let Some(extra_keys) = &self.extra_keys {
            config.extra_keys = Some(extra_keys.clone());
        }
        if let Some(drop_count) = &self.drop_count {
            config.drop_count = Some(*drop_count);
        }
        if let Some(drop_empty_cells) = &self.drop_empty_cells {
            config.drop_empty_cells = Some(*drop_empty_cells);
        }
        if let Some(drop_id) = &self.drop_id {
            config.drop_id = Some(*drop_id);
        }
        if let Some(drop_output) = &self.drop_output {
            config.drop_output = Some(*drop_output);
        }
        if let Some(drop_tagged_cells) = &self.drop_tagged_cells {
            config.drop_tagged_cells = Some(drop_tagged_cells.clone());
        }
        if let Some(strip_init_cell) = &self.strip_init_cell {
            config.strip_init_cell = Some(*strip_init_cell);
        }
        if let Some(keep_keys) = &self.keep_keys {
            config.keep_keys = Some(keep_keys.clone());
        }
        config
    }
}
