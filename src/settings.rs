use crate::cli::ConfigOverrides;
use crate::config::resolve;
use crate::types::ExtraKey;
use rustc_hash::FxHashSet;
use std::path::Path;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct Settings {
    pub extra_keys: Vec<ExtraKey>,
    pub drop_tagged_cells: FxHashSet<String>,
    pub drop_empty_cells: bool,
    pub drop_output: bool,
    pub drop_count: bool,
    pub drop_id: bool,
}

impl Settings {
    pub fn construct(config_file: Option<&Path>, overrides: &ConfigOverrides) -> Self {
        let mut config = resolve(config_file);
        config = overrides.override_config(config);

        config.into_settings()
    }
}
