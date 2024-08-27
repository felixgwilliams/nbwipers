use crate::files::{get_cwd, normalize_path_to};
use crate::{extra_keys::ExtraKey, settings::Settings};
use globset::{Glob, GlobSet, GlobSetBuilder};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigurationSection {
    pub extra_keys: Option<Vec<ExtraKey>>,
    pub drop_empty_cells: Option<bool>,
    pub drop_output: Option<bool>,
    pub drop_count: Option<bool>,
    pub drop_id: Option<bool>,
    pub drop_tagged_cells: Option<Vec<String>>,
    pub strip_init_cell: Option<bool>,
    pub keep_keys: Option<Vec<ExtraKey>>,
    pub exclude: Option<Vec<String>>,
    pub extend_exclude: Option<Vec<String>>,
}

impl ConfigurationSection {
    pub fn make_configuration(self, own_path: Option<&Path>) -> Configuration {
        let parent = own_path.map_or_else(get_cwd, |own_path| {
            own_path
                .parent()
                .expect("parent of own path should exist")
                .to_owned()
        });
        let exclude = self.exclude.map(|excludes| {
            excludes
                .into_iter()
                .map(|p| FilePattern::new_with_path(&p, &parent))
                .collect()
        });
        let extend_exclude = self
            .extend_exclude
            .unwrap_or_default()
            .into_iter()
            .map(|p| FilePattern::new_with_path(&p, &parent))
            .collect();

        Configuration {
            extra_keys: self.extra_keys,
            drop_empty_cells: self.drop_empty_cells,
            drop_output: self.drop_output,
            drop_count: self.drop_count,
            drop_id: self.drop_id,
            drop_tagged_cells: self.drop_tagged_cells,
            strip_init_cell: self.strip_init_cell,
            keep_keys: self.keep_keys,
            exclude,
            extend_exclude,
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Configuration {
    pub extra_keys: Option<Vec<ExtraKey>>,
    pub drop_empty_cells: Option<bool>,
    pub drop_output: Option<bool>,
    pub drop_count: Option<bool>,
    pub drop_id: Option<bool>,
    pub drop_tagged_cells: Option<Vec<String>>,
    pub strip_init_cell: Option<bool>,
    pub keep_keys: Option<Vec<ExtraKey>>,
    pub exclude: Option<Vec<FilePattern>>,
    pub extend_exclude: Vec<FilePattern>,
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
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct FilePattern {
    pattern: String,
    absolute: PathBuf,
}

impl FilePattern {
    pub fn add_to(self, builder: &mut GlobSetBuilder) -> anyhow::Result<()> {
        builder.add(Glob::new(&self.absolute.to_string_lossy())?);

        // Add basename path.
        if !self.pattern.contains(std::path::MAIN_SEPARATOR) {
            builder.add(Glob::new(&self.pattern)?);
        }

        Ok(())
    }
    pub fn new_with_path(pattern: &str, path: &Path) -> Self {
        let absolute = normalize_path_to(pattern, path);
        Self {
            pattern: pattern.to_owned(),
            absolute,
        }
    }
}

impl Display for FilePattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.pattern.as_str())
    }
}

impl FromStr for FilePattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pattern = s.to_string();
        let absolute = crate::files::normalize_path(&pattern);
        Ok(Self { pattern, absolute })
    }
}

impl Serialize for FilePattern {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.pattern)
    }
}
fn make_globset<I: IntoIterator<Item = FilePattern>>(patterns: I) -> anyhow::Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    #[allow(clippy::unwrap_used)]
    for pattern in patterns {
        pattern.add_to(&mut builder)?;
    }
    builder.build().map_err(std::convert::Into::into)
}

impl Configuration {
    pub fn into_settings(self) -> Result<Settings, anyhow::Error> {
        let mut extra_keys = default_extra_keys();
        extra_keys.extend(self.extra_keys.unwrap_or_default());
        for key in &self.keep_keys.unwrap_or_default() {
            extra_keys.remove(key);
        }

        let exclude = self
            .exclude
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|x| x.pattern.clone())
            .collect();
        let extend_exclude = self
            .extend_exclude
            .iter()
            .map(|x| x.pattern.clone())
            .collect();
        let exclude_ = make_globset(self.exclude.unwrap_or_default())?;
        let extend_exclude_ = make_globset(self.extend_exclude)?;

        Ok(Settings {
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
            exclude,
            exclude_,
            extend_exclude,
            extend_exclude_,
        })
    }
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct Pyproject {
    tool: Option<Tools>,
}
#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Tools {
    nbwipers: Option<ConfigurationSection>,
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

pub fn read_pyproject<P: AsRef<Path>>(
    path: P,
) -> Result<Option<ConfigurationSection>, PyprojectError> {
    let contents = std::fs::read_to_string(path)?;
    let pyproject: Pyproject = toml::from_str(&contents)?;
    let config = pyproject.tool.and_then(|tools| tools.nbwipers);
    Ok(config)
}
pub fn read_nbwipers<P: AsRef<Path>>(
    path: P,
) -> Result<Option<ConfigurationSection>, PyprojectError> {
    let contents = std::fs::read_to_string(path)?;
    let config: ConfigurationSection = toml::from_str(&contents)?;
    Ok(Some(config))
}

fn read_settings<P: AsRef<Path>>(path: P) -> Result<Option<ConfigurationSection>, PyprojectError> {
    if path.as_ref().ends_with("pyproject.toml") {
        read_pyproject(&path)
    } else {
        read_nbwipers(path)
    }
}

pub fn resolve(
    config_file: Option<&Path>,
) -> Result<(ConfigurationSection, Option<PathBuf>), PyprojectError> {
    if let Some(config_file) = config_file {
        let config = read_settings(config_file)?;
        Ok((config.unwrap_or_default(), Some(config_file.to_owned())))
    } else if let Some(settings_file) = find_settings()? {
        let config = read_settings(&settings_file)?;
        Ok((config.unwrap_or_default(), Some(settings_file)))
    } else {
        Ok((ConfigurationSection::default(), None))
    }
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

            let (settings, _) = resolve(None).unwrap();
            assert_eq!(
                settings.extra_keys,
                Some(vec![ExtraKey::from_str("metadata.bananas").unwrap()])
            );
            fs::remove_file(dot_nbwipers).unwrap();
            let (settings, _) = resolve(None).unwrap();

            assert_eq!(
                settings.extra_keys,
                Some(vec![ExtraKey::from_str("metadata.kiwis").unwrap()])
            );
            fs::remove_file(nbwipers).unwrap();
            let (settings, _) = resolve(None).unwrap();

            assert_eq!(
                settings.extra_keys,
                Some(vec![ExtraKey::from_str("metadata.pineapples").unwrap()])
            );
        });
    }
}
