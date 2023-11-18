use std::{fs::File, io::BufReader, path::Path, str::FromStr};

use rustc_hash::FxHashSet;
use serde::{de, Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use thiserror::Error;

/// The root of the JSON of a Jupyter Notebook
///
/// Generated by <https://app.quicktype.io/> from
/// <https://github.com/jupyter/nbformat/blob/16b53251aabf472ad9406ddb1f78b0421c014eeb/nbformat/v4/nbformat.v4.schema.json>
/// Jupyter Notebook v4.5 JSON schema.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RawNotebook {
    /// Array of cells of the current notebook.
    pub cells: Vec<Cell>,
    /// Notebook root-level metadata.
    pub metadata: Value,
    /// Notebook format (major number). Incremented between backwards incompatible changes to the
    /// notebook format.
    pub nbformat: i64,
    /// Notebook format (minor number). Incremented for backward compatible changes to the
    /// notebook format.
    pub nbformat_minor: i64,
}

/// String identifying the type of cell.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "cell_type")]
pub enum Cell {
    #[serde(rename = "code")]
    Code(CodeCell),
    #[serde(rename = "markdown")]
    Markdown(MarkdownCell),
    #[serde(rename = "raw")]
    Raw(RawCell),
}

/// Notebook raw nbconvert cell.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RawCell {
    pub attachments: Option<Value>,
    /// Technically, id isn't required (it's not even present) in schema v4.0 through v4.4, but
    /// it's required in v4.5. Main issue is that pycharm creates notebooks without an id
    /// <https://youtrack.jetbrains.com/issue/PY-59438/Jupyter-notebooks-created-with-PyCharm-are-missing-the-id-field-in-cells-in-the-.ipynb-json>
    pub id: Option<String>,
    /// Cell-level metadata.
    pub metadata: Value,
    pub source: SourceValue,
}

/// Notebook markdown cell.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MarkdownCell {
    pub attachments: Option<Value>,
    /// Technically, id isn't required (it's not even present) in schema v4.0 through v4.4, but
    /// it's required in v4.5. Main issue is that pycharm creates notebooks without an id
    /// <https://youtrack.jetbrains.com/issue/PY-59438/Jupyter-notebooks-created-with-PyCharm-are-missing-the-id-field-in-cells-in-the-.ipynb-json>
    pub id: Option<String>,
    /// Cell-level metadata.
    pub metadata: Value,
    pub source: SourceValue,
}

/// Notebook code cell.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CodeCell {
    /// The code cell's prompt number. Will be null if the cell has not been run.
    pub execution_count: Option<i64>,
    /// Technically, id isn't required (it's not even present) in schema v4.0 through v4.4, but
    /// it's required in v4.5. Main issue is that pycharm creates notebooks without an id
    /// <https://youtrack.jetbrains.com/issue/PY-59438/Jupyter-notebooks-created-with-PyCharm-are-missing-the-id-field-in-cells-in-the-.ipynb-json>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Cell-level metadata.
    pub metadata: Value,
    /// Execution, display, or stream outputs.
    pub outputs: Vec<Value>,
    pub source: SourceValue,
}

// /// Notebook root-level metadata.
// #[skip_serializing_none]
// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
// pub struct RawNotebookMetadata {
//     /// The author(s) of the notebook document
//     pub authors: Option<Value>,
//     /// Kernel information.
//     pub kernelspec: Option<Value>,
//     /// Kernel information.
//     pub language_info: Option<LanguageInfo>,
//     /// Original notebook format (major number) before converting the notebook between versions.
//     /// This should never be written to a file.
//     pub orig_nbformat: Option<i64>,
//     /// The title of the notebook document
//     pub title: Option<String>,
//     /// For additional properties.
//     #[serde(flatten)]
//     pub extra: BTreeMap<String, Value>,
// }

// /// Kernel information.
// #[skip_serializing_none]
// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
// pub struct LanguageInfo {
//     /// The codemirror mode to use for code in this language.
//     pub codemirror_mode: Option<Value>,
//     /// The file extension for files in this language.
//     pub file_extension: Option<String>,
//     /// The mimetype corresponding to files in this language.
//     pub mimetype: Option<String>,
//     /// The programming language which this kernel runs.
//     pub name: String,
//     /// The pygments lexer to use for code in this language.
//     pub pygments_lexer: Option<String>,
//     /// For additional properties.
//     #[serde(flatten)]
//     pub extra: BTreeMap<String, Value>,
// }

/// mimetype output (e.g. text/plain), represented as either an array of strings or a
/// string.
///
/// Contents of the cell, represented as an array of lines.
///
/// The stream's text output, represented as an array of strings.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SourceValue {
    String(String),
    StringArray(Vec<String>),
}

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
        self.execution_count.is_none()
    }
    pub fn is_clear_id(&self) -> bool {
        self.id.is_none()
    }
    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }
    pub fn clear_counts(&mut self) {
        self.execution_count = None;
    }
    pub fn should_clear_output(&self, drop_output: bool) -> bool {
        // drop_output
        if drop_output {
            let keep_output_metadata = self
                .metadata
                .as_object()
                .is_some_and(|x| x.contains_key("keep_output"));
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
