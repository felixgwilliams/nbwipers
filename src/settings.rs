use crate::cli::ConfigOverrides;
use crate::config::{resolve, Configuration};
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
    pub strip_kernel_info: bool,
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
        isolated: bool,
        overrides: &ConfigOverrides,
    ) -> Result<Self, anyhow::Error> {
        let mut config = if isolated {
            Configuration::default()
        } else {
            let (config_sec, config_path) = resolve(config_file)?;
            config_sec.make_configuration(config_path.as_deref())
        };
        config = overrides.override_config(config);

        config.into_settings()
    }
}
