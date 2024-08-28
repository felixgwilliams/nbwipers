use std::{fmt::Display, path::Path};

use crate::{
    extra_keys::partition_extra_keys,
    files::{relativize_path, NBReadError},
    schema::RawNotebook,
    settings::Settings,
    utils::get_value_child,
};
use serde::{Serialize, Serializer};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum CheckResult {
    IOError {
        error: String,
    },
    InvalidNotebook {
        error: String,
    },
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
            NBReadError::IO(e) => Self::IOError {
                error: e.to_string(),
            },
            NBReadError::Serde(e) => Self::InvalidNotebook {
                error: e.to_string(),
            },
        }
    }
}

fn relativize_ser<S>(p: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&relativize_path(p))
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PathCheckResult<'a> {
    #[serde(serialize_with = "relativize_ser")]
    pub path: &'a Path,
    #[serde(flatten)]
    pub result: &'a CheckResult,
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError { error } => write!(f, "IO Error: {error}"),
            Self::InvalidNotebook { error } => write!(f, "Invalid notebook: {error}"),
            Self::DropCells { cell_number } => {
                write!(f, "cell: {cell_number}: Found cell to be dropped")
            }
            Self::StripMeta { extra_key } => {
                write!(f, "Found notebook metadata: {extra_key}")
            }

            Self::CellStripMeta {
                cell_number,
                extra_key,
            } => write!(f, "cell {cell_number}: Found cell metadata {extra_key}"),
            Self::ClearCount { cell_number } => {
                write!(f, "cell {cell_number}: Found cell with execution count")
            }
            Self::ClearId { cell_number } => {
                write!(f, "cell {cell_number}: Found cell with Id")
            }
            Self::ClearOutput { cell_number } => {
                write!(f, "cell {cell_number}: Found cell with output")
            }
        }
    }
}

pub fn check_nb(nb: &RawNotebook, settings: &Settings) -> Vec<CheckResult> {
    let (cell_keys, meta_keys) = partition_extra_keys(&settings.extra_keys);
    let mut out = vec![];
    let nb_keep_output = get_value_child(&nb.metadata, &["keep_output"])
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let drop_output = settings.drop_output && !nb_keep_output;

    meta_keys
        .iter()
        .filter(|k| get_value_child(&nb.metadata, k.get_parts()).is_some())
        .for_each(|extra_key| {
            out.push(CheckResult::StripMeta {
                extra_key: extra_key.to_string(),
            });
        });

    nb.cells
        .iter()
        .enumerate()
        .filter(|(_i, c)| c.should_drop(settings.drop_empty_cells, &settings.drop_tagged_cells))
        .for_each(|(cell_number, _c)| out.push(CheckResult::DropCells { cell_number }));

    if drop_output {
        nb.cells
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.as_codecell().map(|c| (i, c)))
            .filter(|(_i, c)| {
                !c.is_clear_outputs()
                    && c.should_clear_output(drop_output, settings.strip_init_cell)
            })
            .for_each(|(cell_number, _)| out.push(CheckResult::ClearOutput { cell_number }));
    }
    if settings.drop_count {
        nb.cells
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.as_codecell().map(|c| (i, c)))
            .filter(|(_i, c)| !c.is_clear_exec_count())
            .for_each(|(cell_number, _)| out.push(CheckResult::ClearCount { cell_number }));
    }
    if settings.drop_id {
        nb.cells
            .iter()
            .enumerate()
            .filter(|(i, c)| !c.is_clear_id(*i))
            .for_each(|(cell_number, _)| out.push(CheckResult::ClearId { cell_number }));
    }
    for (cell_number, cell) in nb.cells.iter().enumerate() {
        cell_keys
            .iter()
            .filter(|k| get_value_child(cell.get_metadata(), k.get_parts()).is_some())
            .for_each(|k| {
                out.push(CheckResult::CellStripMeta {
                    cell_number,
                    extra_key: k.to_string(),
                });
            });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_check_result_convert() {
        let ioerror = NBReadError::IO(std::io::Error::from(std::io::ErrorKind::NotFound));
        let check_result: CheckResult = ioerror.into();
        assert!(matches!(check_result, CheckResult::IOError { .. }));
    }
    #[test]
    fn test_display_ioerror() {
        let ioerror = NBReadError::IO(std::io::Error::from(std::io::ErrorKind::NotFound));
        let check_result: CheckResult = ioerror.into();
        let displayed = check_result.to_string();
        assert!(displayed.starts_with("IO Error"));
    }
}
