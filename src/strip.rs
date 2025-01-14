use std::{
    fmt::Display,
    fs,
    io::{BufWriter, Write},
    path::Path,
};

use serde::Serialize;
use thiserror::Error;

/// Maximum nbformat_minor version for which cell ids are optional.
use crate::{
    config::IdAction,
    extra_keys::partition_extra_keys,
    files::{read_nb, read_nb_stdin, NBReadError, NBWriteError},
    schema::{RawNotebook, ID_OPTIONAL_MAX_VERSION},
    settings::Settings,
    utils::{get_value_child, pop_cell_key, pop_meta_key},
};
use serde_json::Value;

pub fn strip_nb(mut nb: RawNotebook, settings: &Settings) -> (RawNotebook, bool) {
    let (cell_keys, meta_keys) = partition_extra_keys(&settings.extra_keys);
    let nb_keep_output = get_value_child(&nb.metadata, &["keep_output"])
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let drop_output = settings.drop_output && !nb_keep_output;

    let mut stripped = false;
    for meta_key in meta_keys {
        stripped |= pop_meta_key(&mut nb, meta_key).is_some();
    }

    let drop_cells: Vec<_> = nb
        .cells
        .iter()
        .map(|c| c.should_drop(settings.drop_empty_cells, &settings.drop_tagged_cells))
        .collect();
    if drop_cells.iter().any(|b| *b) {
        stripped = true;
        let mut retained_cells = vec![];
        for (cell, to_drop) in nb.cells.into_iter().zip(drop_cells.iter()) {
            if !to_drop {
                retained_cells.push(cell);
            }
        }
        nb.cells = retained_cells;
    }
    let mut downgrade_nbversion_minor = false;

    for (i, cell) in nb.cells.iter_mut().enumerate() {
        if let Some(codecell) = cell.as_codecell_mut() {
            if codecell.should_clear_output(drop_output, settings.strip_init_cell)
                && !codecell.is_clear_outputs()
            {
                stripped = true;

                codecell.clear_outputs();
            }
            if settings.drop_count && !codecell.is_clear_exec_count() {
                stripped = true;

                codecell.clear_counts();
            }
        }
        match settings.id_action {
            IdAction::Sequential => {
                if !cell.is_clear_id(i) {
                    stripped = true;
                    cell.set_id(Some(format!("{i}")));
                }
            }
            IdAction::Drop => {
                if cell.get_id().is_some() {
                    stripped = true;
                    downgrade_nbversion_minor = true;
                    cell.set_id(None);
                }
            }
            IdAction::Keep => {}
        }

        for cell_key in &cell_keys {
            stripped |= pop_cell_key(cell, cell_key).is_some();
        }
    }
    if downgrade_nbversion_minor && nb.nbformat_minor > ID_OPTIONAL_MAX_VERSION {
        nb.nbformat_minor = ID_OPTIONAL_MAX_VERSION;
        stripped = true;
    }
    (nb, stripped)
}
pub fn strip_single(
    nb_path: &Path,
    textconv: bool,
    settings: &Settings,
) -> Result<StripSuccess, StripError> {
    let (nb, to_stdout) = match nb_path.to_str() {
        Some("-") => (read_nb_stdin()?, true),
        _ => (read_nb(nb_path)?, textconv),
    };

    let (strip_nb, stripped) = strip_nb(nb, settings);
    match (to_stdout, stripped) {
        (true, _) => {
            let stdout = std::io::stdout();
            match write_nb(stdout, &strip_nb) {
                Ok(()) => Ok(StripSuccess::from_stripped(stripped)),
                #[cfg(not(tarpaulin_include))]
                Err(e) => Err(e.into()),
            }
        }
        (false, false) => Ok(StripSuccess::NoChange),
        (false, true) => {
            let f = fs::File::create(nb_path).map_err(NBWriteError::from)?;
            let writer = BufWriter::new(f);
            match write_nb(writer, &strip_nb) {
                Ok(()) => Ok(StripSuccess::Stripped),
                #[cfg(not(tarpaulin_include))]
                Err(e) => Err(e.into()),
            }
        }
    }
}
pub fn write_nb<W, T>(mut writer: W, value: &T) -> Result<(), NBWriteError>
where
    W: Write,
    T: ?Sized + Serialize,
{
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut ser = serde_json::Serializer::with_formatter(&mut writer, formatter);
    value.serialize(&mut ser)?;
    writeln!(writer)?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum StripError {
    #[error("File read Error")]
    ReadError(#[from] NBReadError),
    #[error("File write Error")]
    WriteError(#[from] NBWriteError),
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

impl StripSuccess {
    pub const fn from_stripped(stripped: bool) -> Self {
        if stripped {
            Self::Stripped
        } else {
            Self::NoChange
        }
    }
}

impl StripResult {
    pub const fn is_err(&self) -> bool {
        matches!(self, Self::ReadError(_) | Self::WriteError(_))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_error_to_strip_result() {
        let read_error = StripError::ReadError(NBReadError::IO(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        )));
        let write_error = StripError::WriteError(NBWriteError::IO(std::io::Error::from(
            std::io::ErrorKind::PermissionDenied,
        )));
        let read_error_res: StripResult = read_error.into();
        let write_error_res: StripResult = write_error.into();
        assert!(matches!(read_error_res, StripResult::ReadError(..)));
        assert!(matches!(write_error_res, StripResult::WriteError(..)));

        assert!(write_error_res.to_string().starts_with("Write error:"));
    }
}
