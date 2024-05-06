use std::str::FromStr;

use serde::{de, Deserialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExtraKey {
    CellMeta(StripKey),
    Metadata(StripKey),
}

impl ExtraKey {
    pub const fn get_parts(&self) -> &Vec<String> {
        match self {
            Self::Metadata(ref c) | Self::CellMeta(ref c) => &c.parts,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StripKey {
    pub(crate) parts: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum ExtraKeyParseError {
    #[error("Key must start with `cell.metadata` or `metadata`")]
    NotCellOrMetadata,
    #[error("Empty Subkey")]
    EmptySubKey,
    #[error("Empty Key")]
    Empty,
}

impl StripKey {
    pub fn try_from_slice(parts: &[&str]) -> Result<Self, ExtraKeyParseError> {
        if parts.is_empty() || parts == [""] {
            Err(ExtraKeyParseError::EmptySubKey)
        } else {
            Ok(Self {
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
            Some((&"", [])) => Err(ExtraKeyParseError::Empty),
            Some(_) => Err(ExtraKeyParseError::NotCellOrMetadata),
            None => unreachable!(),
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

pub fn partition_extra_keys<'a, I: IntoIterator<Item = &'a ExtraKey>>(
    extra_keys: I,
) -> (Vec<&'a ExtraKey>, Vec<&'a ExtraKey>) {
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
        extra_keys::ExtraKeyParseError,
        schema::Cell,
        utils::{pop_cell_key, pop_value_child},
    };

    use super::ExtraKey;
    use crate::config::EXTRA_KEYS;
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
    #[test]
    fn test_key_roundtrip() {
        for key in EXTRA_KEYS {
            let parsed_key = ExtraKey::from_str(key).unwrap();
            let key2 = parsed_key.to_string();
            assert!(key == &key2);
        }
    }

    #[test]
    fn test_key_parse_errors() {
        assert!(matches!(
            ExtraKey::from_str("hello.world"),
            Err(ExtraKeyParseError::NotCellOrMetadata)
        ));
        assert!(matches!(
            ExtraKey::from_str("metadata."),
            Err(ExtraKeyParseError::EmptySubKey)
        ));
        assert!(matches!(
            ExtraKey::from_str("metadata"),
            Err(ExtraKeyParseError::EmptySubKey)
        ));
        assert!(matches!(
            ExtraKey::from_str(""),
            Err(ExtraKeyParseError::Empty)
        ));
        assert!(matches!(
            ExtraKey::from_str(".world"),
            Err(ExtraKeyParseError::NotCellOrMetadata)
        ));
        assert!(matches!(
            ExtraKey::from_str("cell.interlinked.world"),
            Err(ExtraKeyParseError::NotCellOrMetadata)
        ));
    }

    #[test]
    fn test_deserialize() {
        let valid_key: serde_json::Result<ExtraKey> =
            serde_json::from_str("\"metadata.hello.world\"");
        let invalid_key: serde_json::Result<ExtraKey> = serde_json::from_str("\"hello.world\"");

        assert!(valid_key.is_ok());
        assert!(invalid_key.is_err());
    }
}
