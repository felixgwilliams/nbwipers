use std::path::PathBuf;

use clap::{command, Parser, Subcommand, ValueEnum};

use crate::{
    config::{Configuration, FilePattern},
    extra_keys::ExtraKey,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long, hide = true)]
    pub markdown_help: bool,
    #[command(subcommand)]
    pub command: Commands,
}
#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Debug, Clone)]
pub struct CommonArgs {
    /// path to pyproject.toml/.nbwipers.toml/nbwipers.toml file containing nbwipers settings. If not given use the file in the current working directory or the first such file in its containing folders.
    #[arg(long, short)]
    pub config: Option<PathBuf>,
    /// Ignore all configuration files.
    #[arg(long, conflicts_with = "config")]
    pub isolated: bool,

    /// Do not return an error if no notebooks are found
    #[arg(long)]
    pub allow_no_notebooks: bool,

    /// extra keys to remove in the notebook or cell metadata, separated by commas. Must start with `metadata` or `cell.metadata`
    #[arg(long, value_delimiter = ',')]
    pub extra_keys: Option<Vec<ExtraKey>>,

    /// drop empty cells. Disable with `--keep-empty-cells`
    #[arg(long, overrides_with("keep_empty_cells"))]
    pub drop_empty_cells: bool,

    #[arg(long, overrides_with("drop_empty_cells"), hide = true)]
    pub keep_empty_cells: bool,

    /// keep cell output. Disable with `--drop-output`
    #[arg(long, overrides_with("drop_output"))]
    pub keep_output: bool,

    #[arg(long, overrides_with("keep_output"), hide = true)]
    pub drop_output: bool,

    /// keep cell execution count. Disable with `--drop count`
    #[arg(long, overrides_with("drop_count"))]
    pub keep_count: bool,

    #[arg(long, overrides_with("keep_count"), hide = true)]
    pub drop_count: bool,

    /// replace cell ids with sequential ids. Disable with `--keep-id`
    #[arg(long, overrides_with("keep_id"))]
    pub drop_id: bool,

    #[arg(long, overrides_with("drop_id"), hide = true)]
    pub keep_id: bool,

    /// Strip init cell. Disable with `--keep-init-cell`
    #[arg(long, overrides_with("keep_init_cell"))]
    pub strip_init_cell: bool,

    #[arg(long, overrides_with("strip_init_cell"), hide = true)]
    pub keep_init_cell: bool,
    /// Strip kernel info. Namely, metadata.kernelspec and metadata.language_info.python_version. Disable with `--keep-kernel-info`
    #[arg(long, overrides_with("keep_kernel_info"))]
    pub strip_kernel_info: bool,

    #[arg(long, overrides_with("strip_kernel_info"), hide = true)]
    pub keep_kernel_info: bool,

    /// comma-separated list of tags that will cause the cell to be dropped
    #[arg(long, value_delimiter = ',')]
    pub drop_tagged_cells: Option<Vec<String>>,

    /// List of metadata keys that should be kept, regardless of if they appear in
    #[arg(long, value_delimiter = ',')]
    pub keep_keys: Option<Vec<ExtraKey>>,
    /// List of file patterns to ignore
    #[arg(long, value_delimiter = ',')]
    pub exclude: Option<Vec<FilePattern>>,
    /// List of additional file patterns to ignore
    #[arg(long, value_delimiter = ',')]
    pub extend_exclude: Option<Vec<FilePattern>>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Register nbwipers as a git filter for `ipynb` files
    Install(InstallCommand),
    /// clean all notebooks in a given path
    CleanAll(CleanAllCommand),
    /// check notebooks in a given path for elements that would be removed by `clean`
    Check(CheckCommand),
    /// clean a single notebook
    Clean(CleanCommand),
    /// uninstall nbwipers as a git filter
    Uninstall(UninstallCommand),
    /// check whether nbwipers is setup as a git filter
    CheckInstall(CheckInstallCommand),
    /// Show configuration
    ShowConfig(ShowConfigCommand),
    /// Record Kernelspec metadata for notebooks
    Record(RecordCommand),
    /// Add back kernelspec metadata to the notebook as a smudge
    #[clap(hide(true))]
    Smudge(SmudgeCommand),
    /// Commands for pre-commit hooks
    #[command(subcommand)]
    Hook(HookCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum HookCommands {
    /// Check for large files, but measure ipynb sizes after clearning
    CheckLargeFiles(CheckLargeFilesCommand),
}

#[derive(Clone, Debug, Parser)]
pub struct CheckLargeFilesCommand {
    /// Files to check for large files.
    pub filenames: Vec<PathBuf>,
    /// Check all files not just staged files
    #[arg(long, action)]
    pub enforce_all: bool,
    /// Max size in KB to consider a file large
    #[arg(long("maxkb"))]
    pub maxkb: Option<u64>,
    /// path to pyproject.toml/.nbwipers.toml/nbwipers.toml file containing nbwipers settings. If not given use the file in the current working directory or the first such file in its containing folders.
    #[arg(long, short)]
    pub config: Option<PathBuf>,

    /// Ignore all configuration files.
    #[arg(long, conflicts_with = "config")]
    pub isolated: bool,
}

#[derive(Clone, Debug, Parser)]
pub struct ShowConfigCommand {
    /// Show all config including defaults Disable with `--no-show-defaults`
    #[arg(long, overrides_with("no_show_defaults"))]
    pub show_all: bool,

    #[arg(long, overrides_with("show_all"), hide = true)]
    pub no_show_defaults: bool,
    #[clap(flatten)]
    pub common: CommonArgs,
}
#[derive(Clone, Debug, Parser)]
pub struct CheckCommand {
    /// paths containing ipynb files to check. Use `-` to read from stdin
    pub files: Vec<PathBuf>,

    /// desired output format for diagnostics
    #[arg(long, short)]
    pub output_format: Option<OutputFormat>,

    #[clap(flatten)]
    pub common: CommonArgs,
}
#[derive(Clone, Debug, Parser)]
pub struct CleanCommand {
    /// path to ipynb file to clean. Use `-` to read from stdin and write to stdout
    pub file: PathBuf,

    /// write cleaned file to stdout instead of to the file
    #[arg(long, short)]
    pub textconv: bool,

    #[clap(flatten)]
    pub common: CommonArgs,
}
#[derive(Clone, Debug, Parser)]
pub struct CleanAllCommand {
    /// paths containing ipynb files to clean. Stdin is not supported.
    pub files: Vec<PathBuf>,

    /// set to true to avoid writing to files
    #[arg(long, short)]
    pub dry_run: bool,

    /// skip confirmation and assume yes
    #[arg(long, short)]
    pub yes: bool,

    #[clap(flatten)]
    pub common: CommonArgs,
}

#[derive(Clone, Debug, ValueEnum, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}
#[derive(Clone, Debug, Parser)]
pub struct InstallCommand {
    /// Git config type that determines which file to modify
    #[clap(value_enum)]
    pub config_type: GitConfigType,

    /// Optional path to git config file
    #[arg(long, short)]
    pub git_config_file: Option<PathBuf>,

    /// optional attribute file. If not specified, will write to .git/info/attributes
    #[arg(long, short)]
    pub attribute_file: Option<PathBuf>,
}
#[derive(Clone, Debug, Parser)]
pub struct UninstallCommand {
    /// Git config type that determines which file to modify
    #[clap(value_enum)]
    pub config_type: GitConfigType,

    /// Optional path to git config file
    #[arg(long, short)]
    pub git_config_file: Option<PathBuf>,

    /// optional attribute file. If not specified, will write to .git/info/attributes
    #[arg(long, short)]
    pub attribute_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Parser)]
pub struct RecordCommand {
    pub path: Option<PathBuf>,

    #[arg(long)]
    pub remove: Vec<PathBuf>,

    #[arg(long)]
    pub clear: bool,
    #[arg(long)]
    pub sync: bool,

    #[clap(flatten)]
    pub common: CommonArgs,
}
#[derive(Clone, Debug, Parser)]
pub struct CheckInstallCommand {
    /// Exit zero regardless of install status
    #[arg(long)]
    pub exit_zero: bool,
    /// Git config type to check
    #[clap(value_enum)]
    pub config_type: Option<GitConfigType>,
}

#[derive(Clone, Debug, Parser)]
pub struct SmudgeCommand {
    pub path: String,
}

#[derive(Clone, Debug, ValueEnum, Copy)]
pub enum GitConfigType {
    /// System-wide git config
    System,
    /// User level git config, typically corresponding to ~/.gitconfig
    Global,
    /// Repository level git config, corresponding to .git/config
    Local,
}

#[derive(Clone, Debug, Default)]
pub struct ConfigOverrides {
    pub strip_kernel_info: Option<bool>,
    pub extra_keys: Option<Vec<ExtraKey>>,
    pub drop_empty_cells: Option<bool>,
    pub drop_output: Option<bool>,
    pub drop_count: Option<bool>,
    pub drop_id: Option<bool>,
    pub strip_init_cell: Option<bool>,
    pub drop_tagged_cells: Option<Vec<String>>,
    pub keep_keys: Option<Vec<ExtraKey>>,
    pub exclude: Option<Vec<FilePattern>>,
    pub extend_exclude: Option<Vec<FilePattern>>,
}

pub struct Args {
    pub config: Option<PathBuf>,
    pub isolated: bool,
    pub allow_no_notebooks: bool,
}

pub fn resolve_bool_arg(yes: bool, no: bool) -> Option<bool> {
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
                allow_no_notebooks: self.allow_no_notebooks,
                isolated: self.isolated,
            },
            ConfigOverrides {
                extra_keys: self.extra_keys,
                drop_empty_cells: resolve_bool_arg(self.drop_empty_cells, self.keep_empty_cells),
                drop_output: resolve_bool_arg(self.drop_output, self.keep_output),
                drop_count: resolve_bool_arg(self.drop_count, self.keep_count),
                drop_id: resolve_bool_arg(self.drop_id, self.keep_id),
                drop_tagged_cells: self.drop_tagged_cells,
                strip_init_cell: resolve_bool_arg(self.strip_init_cell, self.keep_init_cell),
                keep_keys: self.keep_keys,
                extend_exclude: self.extend_exclude,
                exclude: self.exclude,
                strip_kernel_info: resolve_bool_arg(self.strip_kernel_info, self.keep_kernel_info),
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
        if let Some(exclude) = &self.exclude {
            config.exclude = Some(exclude.clone());
        }
        if let Some(extend_exclude) = &self.extend_exclude {
            config.extend_exclude.extend(extend_exclude.clone());
        }
        if let Some(strip_kernel_info) = &self.strip_kernel_info {
            config.strip_kernel_info = Some(*strip_kernel_info);
        }
        config
    }
}
