use crate::{settings::Settings, wipers::ExtraKey};
use rustc_hash::FxHashSet;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Configuration {
    pub extra_keys: Option<Vec<ExtraKey>>,
    pub drop_empty_cells: Option<bool>,
    pub drop_output: Option<bool>,
    pub drop_count: Option<bool>,
    pub drop_id: Option<bool>,
    pub drop_tagged_cells: Option<Vec<String>>,
}

impl Configuration {
    pub fn into_settings(self) -> Settings {
        Settings {
            extra_keys: self.extra_keys.unwrap_or_default(),
            drop_empty_cells: self.drop_empty_cells.unwrap_or(false),
            drop_output: self.drop_output.unwrap_or(true),
            drop_count: self.drop_count.unwrap_or(true),
            drop_id: self.drop_id.unwrap_or(false),
            drop_tagged_cells: self
                .drop_tagged_cells
                .map(FxHashSet::from_iter)
                .unwrap_or_default(),
        }
    }
}
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct Pyproject {
    tools: Option<Tools>,
}
#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Tools {
    wipers: Option<Configuration>,
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

    pyproject.tools.and_then(|tools| tools.wipers)
}

pub fn resolve(config_file: Option<&Path>) -> Configuration {
    let to_read = config_file.map_or_else(find_pyproject, |x| Some(x.to_owned()));
    to_read.and_then(|p| read_pyproject(&p)).unwrap_or_default()
}
