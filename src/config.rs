use crate::{extra_keys::ExtraKey, settings::Settings};
use rustc_hash::FxHashSet;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Configuration {
    pub extra_keys: Option<Vec<ExtraKey>>,
    pub drop_empty_cells: Option<bool>,
    pub drop_output: Option<bool>,
    pub drop_count: Option<bool>,
    pub drop_id: Option<bool>,
    pub drop_tagged_cells: Option<Vec<String>>,
    pub strip_init_cell: Option<bool>,
    pub keep_keys: Option<Vec<ExtraKey>>,
}

const EXTRA_KEYS: &[&str] = &[
    "metadata.signature",
    "metadata.widgets",
    "cell.metadata.collapsed",
    "cell.metadata.ExecuteTime",
    "cell.metadata.execution",
    "cell.metadata.heading_collapsed",
    "cell.metadata.hidden",
    "cell.metadata.scrolled",
];

fn default_extra_keys() -> FxHashSet<ExtraKey> {
    EXTRA_KEYS
        .iter()
        .filter_map(|s| ExtraKey::from_str(s).ok())
        .collect()
}

impl Configuration {
    pub fn into_settings(self) -> Settings {
        let mut extra_keys = default_extra_keys();
        extra_keys.extend(self.extra_keys.unwrap_or_default());
        for key in &self.keep_keys.unwrap_or_default() {
            extra_keys.remove(key);
        }
        Settings {
            extra_keys,
            drop_empty_cells: self.drop_empty_cells.unwrap_or(false),
            drop_output: self.drop_output.unwrap_or(true),
            drop_count: self.drop_count.unwrap_or(true),
            drop_id: self.drop_id.unwrap_or(false),
            drop_tagged_cells: self
                .drop_tagged_cells
                .map(FxHashSet::from_iter)
                .unwrap_or_default(),
            strip_init_cell: self.strip_init_cell.unwrap_or(false),
        }
    }
}
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct Pyproject {
    tools: Option<Tools>,
}
#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Tools {
    nbwipers: Option<Configuration>,
}

pub fn find_pyproject() -> Option<PathBuf> {
    let cwd = path_absolutize::path_dedot::CWD.as_path();

    for ancestor in cwd.ancestors() {
        let pyproject = ancestor.join("pyproject.toml");
        if pyproject.is_file() {
            return Some(pyproject);
        }
    }
    None
}

pub fn read_pyproject(path: &Path) -> Option<Configuration> {
    let contents = std::fs::read_to_string(path).ok()?;
    let pyproject: Pyproject = toml::from_str(&contents).ok()?;

    pyproject.tools.and_then(|tools| tools.nbwipers)
}

pub fn resolve(config_file: Option<&Path>) -> Configuration {
    let to_read = config_file.map_or_else(find_pyproject, |x| Some(x.to_owned()));
    to_read.and_then(|p| read_pyproject(&p)).unwrap_or_default()
}
