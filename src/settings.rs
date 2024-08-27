use crate::cli::ConfigOverrides;
use crate::config::resolve;
use crate::extra_keys::ExtraKey;
use globset::GlobSet;
use rustc_hash::FxHashSet;
use serde::Serialize;
use std::path::Path;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize)]
pub struct Settings {
    pub extra_keys: FxHashSet<ExtraKey>,
    pub drop_tagged_cells: FxHashSet<String>,
    pub drop_empty_cells: bool,
    pub drop_output: bool,
    pub drop_count: bool,
    pub drop_id: bool,
    pub strip_init_cell: bool,
    pub exclude: Vec<String>,
    #[serde(skip_serializing)]
    pub exclude_: GlobSet,
    pub extend_exclude: Vec<String>,
    #[serde(skip_serializing)]
    pub extend_exclude_: GlobSet,
}

impl Settings {
    pub fn construct(
        config_file: Option<&Path>,
        overrides: &ConfigOverrides,
    ) -> Result<Self, anyhow::Error> {
        let (config_sec, config_path) = resolve(config_file)?;
        let mut config = config_sec.make_configuration(config_path.as_deref());
        config = overrides.override_config(config);

        config.into_settings()
    }
}
