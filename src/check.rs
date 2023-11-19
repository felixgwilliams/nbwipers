use std::fmt::Display;

use crate::{
    extra_keys::partition_extra_keys, files::NBReadError, schema::RawNotebook, settings::Settings,
    utils::get_value_child,
};
use serde::Serialize;
use serde_json::Value;

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

pub fn check_nb(nb: &RawNotebook, settings: &Settings) -> Vec<CheckResult> {
    let (cell_keys, meta_keys) = partition_extra_keys(settings.extra_keys.as_slice());
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
            .filter_map(|(i, c)| c.as_codecell().map(|c| (i, c)))
            .filter(|(_i, c)| !c.is_clear_id())
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
