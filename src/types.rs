use std::{fmt::Display, fs::File, io::BufReader, path::Path, str::FromStr};

use crate::schema::{Cell, CodeCell, RawNotebook, SourceValue};
use rustc_hash::FxHashSet;
use serde::{de, Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

impl SourceValue {
    fn is_empty(&self) -> bool {
        match self {
            SourceValue::String(ref s) => s.trim().is_empty(),
            SourceValue::StringArray(ref s_vec) => s_vec.iter().all(|s| s.trim().is_empty()),
        }
    }
}

impl CodeCell {
    pub fn is_clear_outputs(&self) -> bool {
        self.outputs.is_empty()
    }
    pub fn is_clear_exec_count(&self) -> bool {
        let clear_exec_count = self.execution_count.is_none();

        let output_exec_counts = self
            .outputs
            .iter()
            .filter_map(|v| v.as_object())
            .filter_map(|x| x.get("execution_count"))
            .any(|v| v.as_number().is_some());
        clear_exec_count && !output_exec_counts
    }
    pub fn is_clear_id(&self) -> bool {
        self.id.is_none()
    }
    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }
    pub fn clear_counts(&mut self) {
        self.execution_count = None;
        self.outputs
            .iter_mut()
            .filter_map(|v| v.as_object_mut())
            .for_each(|x| {
                x.insert("execution_count".into(), Value::Null);
            });
    }
    pub fn should_clear_output(&self, drop_output: bool, strip_init_cell: bool) -> bool {
        // drop_output
        let Some(cell_metadata) = self.metadata.as_object() else {
            return drop_output;
        };
        if let Some(init_cell) = cell_metadata.get("init_cell") {
            return !init_cell.as_bool().unwrap_or(false) || strip_init_cell;
        };

        if drop_output {
            let keep_output_metadata = cell_metadata.contains_key("keep_output");
            let keep_output_tags = self
                .metadata
                .as_object()
                .and_then(|x| x.get("tags"))
                .and_then(|x| x.as_array())
                .is_some_and(|x| x.iter().any(|s| s.as_str() == Some("keep_output")));
            !(keep_output_metadata || keep_output_tags)
        } else {
            false
        }
    }
}

impl Cell {
    #[allow(dead_code)]
    pub fn as_codecell(&self) -> Option<&CodeCell> {
        if let Cell::Code(codecell) = self {
            Some(codecell)
        } else {
            None
        }
    }
    pub fn as_codecell_mut(&mut self) -> Option<&mut CodeCell> {
        if let Cell::Code(codecell) = self {
            Some(codecell)
        } else {
            None
        }
    }

    pub fn get_source(&self) -> &SourceValue {
        match self {
            Cell::Code(ref c) => &c.source,
            Cell::Markdown(ref c) => &c.source,
            Cell::Raw(ref c) => &c.source,
        }
    }

    pub fn get_metadata(&self) -> &Value {
        match self {
            Cell::Code(ref c) => &c.metadata,
            Cell::Markdown(ref c) => &c.metadata,
            Cell::Raw(ref c) => &c.metadata,
        }
    }

    pub fn should_drop(
        &self,
        drop_empty_cells: bool,
        drop_tagged_cells: &FxHashSet<String>,
    ) -> bool {
        if drop_empty_cells && self.get_source().is_empty() {
            return true;
        }
        if drop_tagged_cells.is_empty() {
            return false;
        }
        let tags = self
            .get_metadata()
            .as_object()
            .and_then(|x| x.get("tags"))
            .and_then(|x| x.as_array());

        if let Some(tags) = tags {
            tags.iter()
                .filter_map(|v| v.as_str())
                .any(|s| drop_tagged_cells.contains(s))
        } else {
            false
        }
    }
}

fn get_value_child_mut<'a, T: AsRef<str>>(
    value: &'a mut Value,
    path: &[T],
) -> Option<&'a mut Value> {
    let mut cur = value;
    for segment in path {
        cur = cur
            .as_object_mut()
            .and_then(|x| x.get_mut(segment.as_ref()))?;
    }
    Some(cur)
}
pub fn get_value_child<'a, T: AsRef<str>>(value: &'a Value, path: &[T]) -> Option<&'a Value> {
    let mut cur = value;
    for segment in path {
        cur = cur.as_object().and_then(|x| x.get(segment.as_ref()))?;
    }
    Some(cur)
}

pub fn pop_value_child<T: AsRef<str>>(
    value: &mut serde_json::Value,
    path: &[T],
) -> Option<serde_json::Value> {
    let (child_label, parent_path) = path.split_last()?;
    let parent = get_value_child_mut(value, parent_path)?;
    parent
        .as_object_mut()
        .and_then(|m| m.remove(child_label.as_ref()))
}

pub fn pop_cell_key(cell: &mut CodeCell, extra_key: &ExtraKey) -> Option<serde_json::Value> {
    let (cell, ExtraKey::CellMeta(cellmeta_key)) = (cell, extra_key) else {
        return None;
    };
    pop_value_child(&mut cell.metadata, cellmeta_key.parts.as_slice())
}
pub fn pop_meta_key(nb: &mut RawNotebook, extra_key: &ExtraKey) -> Option<serde_json::Value> {
    let (nb, ExtraKey::Metadata(meta_key)) = (nb, extra_key) else {
        return None;
    };
    pop_value_child(&mut nb.metadata, meta_key.parts.as_slice())
}

#[derive(Error, Debug)]
pub enum NBReadError {
    #[error("File IO error")]
    IO(#[from] std::io::Error),
    #[error("JSON read error")]
    Serde(#[from] serde_json::Error),
}

pub fn read_nb(path: &Path) -> Result<RawNotebook, NBReadError> {
    let f = File::open(path)?;
    let rdr = BufReader::new(f);

    let out = serde_json::from_reader(rdr)?;
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtraKey {
    CellMeta(StripKey),
    Metadata(StripKey),
}

impl ExtraKey {
    pub fn get_parts(&self) -> &Vec<String> {
        match self {
            ExtraKey::Metadata(ref c) | ExtraKey::CellMeta(ref c) => &c.parts,
        }
    }
}

impl ToString for ExtraKey {
    fn to_string(&self) -> String {
        match self {
            Self::CellMeta(cellmeta) => format!("cell.metadata.{}", cellmeta.parts.join(".")),
            Self::Metadata(meta) => format!("metadata.{}", meta.parts.join(".")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StripKey {
    pub(crate) parts: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, Copy, Error)]
pub enum ExtraKeyParseError {
    #[error("Key must start with `cell.metadata` or `metadata`")]
    NotCellOrMetadata,
    #[error("No dot")]
    NoDot,
    #[error("Empty")]
    Empty,
}

impl StripKey {
    pub fn try_from_slice(parts: &[&str]) -> Result<Self, ExtraKeyParseError> {
        if parts.is_empty() {
            Err(ExtraKeyParseError::Empty)
        } else {
            Ok(StripKey {
                parts: parts.iter().map(|x| String::from(*x)).collect(),
            })
        }
    }
}

impl FromStr for ExtraKey {
    type Err = ExtraKeyParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('.').collect::<Vec<_>>();
        match parts.split_first() {
            Some((&"cell", tail)) => match tail.split_first() {
                Some((&"metadata", tail2)) => {
                    StripKey::try_from_slice(tail2).map(ExtraKey::CellMeta)
                }
                _ => Err(ExtraKeyParseError::NotCellOrMetadata),
            },

            Some((&"metadata", tail)) => StripKey::try_from_slice(tail).map(ExtraKey::Metadata),
            Some(_) => Err(ExtraKeyParseError::NotCellOrMetadata),
            None => Err(ExtraKeyParseError::NoDot),
        }
    }
}

impl<'de> Deserialize<'de> for ExtraKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str_result = String::deserialize(deserializer)?;
        Self::from_str(str_result.as_str()).map_err(|_| {
            de::Error::invalid_value(
                de::Unexpected::Str(str_result.as_str()),
                &"dot separated json path to key starting with `cell` or `metadata`",
            )
        })
    }
}

pub fn partition_extra_keys(extra_keys: &[ExtraKey]) -> (Vec<&ExtraKey>, Vec<&ExtraKey>) {
    let mut meta_keys = vec![];
    let mut cell_keys = vec![];
    for extra_key in extra_keys {
        match extra_key {
            ExtraKey::CellMeta(ref _cell_key) => cell_keys.push(extra_key),
            ExtraKey::Metadata(ref _meta_key) => meta_keys.push(extra_key),
        };
    }
    (cell_keys, meta_keys)
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::types::pop_cell_key;

    use super::{pop_value_child, Cell, ExtraKey};
    use std::str::FromStr;

    #[test]
    fn test_pop_value_child() {
        let mut x = json!({"hello": {"world": "baby", "banana":"pear"}});
        pop_value_child(&mut x, &"hello.world".split('.').collect::<Vec<_>>());
        assert_eq!(x, json!({"hello": {"banana": "pear"}}));
    }
    #[test]
    fn test_pop_key() {
        let cell_value = json!({
         "cell_type": "code",
         "execution_count": null,
         "metadata": {"banana": "pear"},
         "outputs": [],
         "source": [
          "from ipywidgets import interact"
         ]
        });

        let mut cell: Cell = serde_json::from_value(cell_value).unwrap();
        let extra_key = ExtraKey::from_str("cell.metadata.banana").unwrap();
        println!("{cell:?}");
        println!("{extra_key:?}");
        pop_cell_key(cell.as_codecell_mut().unwrap(), &extra_key);
        println!("{cell:?}");
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub enum CheckResult {
    IOError(String),
    InvalidNotebook(String),
    StripMeta {
        extra_key: String,
    },
    DropCells {
        cell_number: usize,
    },
    ClearOutput {
        cell_number: usize,
    },
    ClearCount {
        cell_number: usize,
    },
    ClearId {
        cell_number: usize,
    },
    CellStripMeta {
        cell_number: usize,
        extra_key: String,
    },
}
impl From<NBReadError> for CheckResult {
    fn from(value: NBReadError) -> Self {
        match value {
            NBReadError::IO(e) => Self::IOError(e.to_string()),
            NBReadError::Serde(e) => Self::InvalidNotebook(e.to_string()),
        }
    }
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckResult::IOError(e) => write!(f, "IO Error: {e}"),
            CheckResult::InvalidNotebook(e) => write!(f, "Invalid notebook: {e}"),
            CheckResult::DropCells { cell_number } => {
                write!(f, "cell: {cell_number}: Found cell to be dropped")
            }
            CheckResult::StripMeta { extra_key } => {
                write!(f, "Found notebook metadata: {extra_key}")
            }

            CheckResult::CellStripMeta {
                cell_number,
                extra_key,
            } => write!(f, "cell {cell_number}: Found cell metadata {extra_key}"),
            CheckResult::ClearCount { cell_number } => {
                write!(f, "cell {cell_number}: Found cell with execution count")
            }
            CheckResult::ClearId { cell_number } => {
                write!(f, "cell {cell_number}: Found cell with Id")
            }
            CheckResult::ClearOutput { cell_number } => {
                write!(f, "cell {cell_number}: Found cell with output")
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum StripError {
    #[error("File read Error")]
    ReadError(#[from] NBReadError),
    #[error("File write Error")]
    WriteError(#[from] NBWriteError),
}

#[derive(Debug, Error)]
pub enum NBWriteError {
    #[error("File IO error")]
    IO(#[from] std::io::Error),
    #[error("JSON read error")]
    Serde(#[from] serde_json::Error),
}

impl StripSuccess {
    pub fn from_stripped(stripped: bool) -> Self {
        if stripped {
            Self::Stripped
        } else {
            Self::NoChange
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub enum StripSuccess {
    NoChange,
    Stripped,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub enum StripResult {
    NoChange,
    Stripped,
    ReadError(String),
    WriteError(String),
}

impl From<StripSuccess> for StripResult {
    fn from(value: StripSuccess) -> Self {
        match value {
            StripSuccess::NoChange => Self::NoChange,
            StripSuccess::Stripped => Self::Stripped,
        }
    }
}
impl From<StripError> for StripResult {
    fn from(value: StripError) -> Self {
        match value {
            StripError::ReadError(e) => Self::ReadError(e.to_string()),
            StripError::WriteError(e) => Self::WriteError(e.to_string()),
        }
    }
}
impl From<Result<StripSuccess, StripError>> for StripResult {
    fn from(value: Result<StripSuccess, StripError>) -> Self {
        match value {
            Ok(v) => v.into(),
            Err(v) => v.into(),
        }
    }
}

impl Display for StripResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoChange => write!(f, "No Change"),
            Self::Stripped => write!(f, "Stripped"),
            Self::ReadError(e) => write!(f, "Read error: {e}"),
            Self::WriteError(e) => write!(f, "Write error: {e}"),
        }
    }
}
