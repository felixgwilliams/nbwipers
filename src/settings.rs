use crate::cli::ConfigOverrides;
use crate::config::{resolve, PyprojectError};
use crate::extra_keys::ExtraKey;
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
}

impl Settings {
    pub fn construct(
        config_file: Option<&Path>,
        overrides: &ConfigOverrides,
    ) -> Result<Self, PyprojectError> {
        let mut config = resolve(config_file)?;
        config = overrides.override_config(config);

        Ok(config.into_settings())
    }
}
