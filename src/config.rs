use crate::files::get_cwd;
use crate::{extra_keys::ExtraKey, settings::Settings};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
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

pub const EXTRA_KEYS: &[&str] = &[
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
    #[allow(clippy::unwrap_used)]
    EXTRA_KEYS
        .iter()
        .map(|s| ExtraKey::from_str(s).unwrap())
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
    tool: Option<Tools>,
}
#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Tools {
    nbwipers: Option<Configuration>,
}

pub fn nbwipers_enabled<P: AsRef<Path>>(path: P) -> Result<bool, PyprojectError> {
    let config = read_pyproject(path)?;
    Ok(config.is_some())
}

fn settings_for_dir<P: AsRef<Path>>(path: P) -> Result<Option<PathBuf>, PyprojectError> {
    let nbwipers_toml = path.as_ref().join(".nbwipers.toml");
    if nbwipers_toml.is_file() {
        return Ok(Some(nbwipers_toml));
    }
    let nbwipers_toml = path.as_ref().join("nbwipers.toml");
    if nbwipers_toml.is_file() {
        return Ok(Some(nbwipers_toml));
    }
    // Check for `pyproject.toml`.
    let pyproject_toml = path.as_ref().join("pyproject.toml");
    if pyproject_toml.is_file() && nbwipers_enabled(&pyproject_toml)? {
        return Ok(Some(pyproject_toml));
    }
    Ok(None)
}

pub fn find_settings() -> Result<Option<PathBuf>, PyprojectError> {
    let cwd = get_cwd();

    for ancestor in cwd.ancestors() {
        if let Some(settings_file) = settings_for_dir(ancestor)? {
            return Ok(Some(settings_file));
        }
    }
    Ok(None)
}

#[derive(Debug, Error)]

pub enum PyprojectError {
    #[error("Pyproject IO Error")]
    IOError(#[from] io::Error),
    #[error("Pyproject Parse Error")]
    ParseError(#[from] toml::de::Error),
}

pub fn read_pyproject<P: AsRef<Path>>(path: P) -> Result<Option<Configuration>, PyprojectError> {
    let contents = std::fs::read_to_string(path)?;
    let pyproject: Pyproject = toml::from_str(&contents)?;
    let config = pyproject.tool.and_then(|tools| tools.nbwipers);
    Ok(config)
}
pub fn read_nbwipers<P: AsRef<Path>>(path: P) -> Result<Option<Configuration>, PyprojectError> {
    let contents = std::fs::read_to_string(path)?;
    let config: Configuration = toml::from_str(&contents)?;
    Ok(Some(config))
}

fn read_settings<P: AsRef<Path>>(path: P) -> Result<Option<Configuration>, PyprojectError> {
    if path.as_ref().ends_with("pyproject.toml") {
        read_pyproject(&path)
    } else {
        read_nbwipers(path)
    }
}

pub fn resolve(config_file: Option<&Path>) -> Result<Configuration, PyprojectError> {
    // config_file.unwrap().is_file()
    if let Some(config_file) = config_file {
        let config = read_settings(config_file)?;
        Ok(config.unwrap_or_default())
    } else if let Some(settings_file) = find_settings()? {
        let config = read_settings(settings_file)?;
        Ok(config.unwrap_or_default())
    } else {
        Ok(Configuration::default())
    }
    // let to_read = config_file.map_or_else(find_pyproject, |x| Some(x.to_owned()));
    // to_read.and_then(|p| read_pyproject(&p)).unwrap_or_default()
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::with_dir;
    use std::fs;

    #[test]
    fn test_nbwipers_priority() {
        let temp_dir = tempfile::tempdir().unwrap();
        with_dir(&temp_dir, || {
            let dot_nbwipers = temp_dir.path().join(".nbwipers.toml");
            fs::write(&dot_nbwipers, "extra-keys = [\"metadata.bananas\"]").unwrap();
            let nbwipers = temp_dir.path().join("nbwipers.toml");
            fs::write(&nbwipers, "extra-keys = [\"metadata.kiwis\"]").unwrap();
            let pyproject = temp_dir.path().join("pyproject.toml");
            fs::write(
                pyproject,
                "[tool.nbwipers]\nextra-keys = [\"metadata.pineapples\"]\n",
            )
            .unwrap();

            let settings = resolve(None).unwrap();
            assert_eq!(
                settings.extra_keys,
                Some(vec![ExtraKey::from_str("metadata.bananas").unwrap()])
            );
            fs::remove_file(dot_nbwipers).unwrap();
            let settings = resolve(None).unwrap();

            assert_eq!(
                settings.extra_keys,
                Some(vec![ExtraKey::from_str("metadata.kiwis").unwrap()])
            );
            fs::remove_file(nbwipers).unwrap();
            let settings = resolve(None).unwrap();

            assert_eq!(
                settings.extra_keys,
                Some(vec![ExtraKey::from_str("metadata.pineapples").unwrap()])
            );
        });
    }
}
