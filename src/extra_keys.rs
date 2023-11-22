use std::str::FromStr;

use serde::{de, Deserialize};
use thiserror::Error;

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

    use crate::{
        schema::Cell,
        utils::{pop_cell_key, pop_value_child},
    };

    use super::ExtraKey;
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
        pop_cell_key(&mut cell, &extra_key);
        println!("{cell:?}");
    }
}
