use std::{
    fmt::Display,
    fs,
    io::{BufWriter, Write},
    path::Path,
};

use serde::Serialize;
use thiserror::Error;

use crate::{
    extra_keys::partition_extra_keys,
    files::{read_nb, NBReadError, NBWriteError},
    schema::RawNotebook,
    settings::Settings,
    utils::{get_value_child, pop_cell_key, pop_meta_key, pop_value_child},
};
use serde_json::Value;

pub fn strip_nb(mut nb: RawNotebook, settings: &Settings) -> (RawNotebook, bool) {
    let (cell_keys, meta_keys) = partition_extra_keys(settings.extra_keys.as_slice());
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

    for cell in &mut nb.cells {
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

        stripped |= pop_value_child(cell.get_metadata_mut(), &["collapsed"]).is_some();
        for cell_key in &cell_keys {
            stripped |= pop_cell_key(cell, cell_key).is_some();
        }
    }
    (nb, stripped)
}
pub fn strip_single(
    nb_path: &Path,
    textconv: bool,
    settings: &Settings,
) -> Result<StripSuccess, StripError> {
    let nb = read_nb(nb_path)?;

    let (strip_nb, stripped) = strip_nb(nb, settings);
    match (textconv, stripped) {
        (true, _) => {
            let stdout = std::io::stdout();
            match write_nb(stdout, &strip_nb) {
                Ok(()) => Ok(StripSuccess::from_stripped(stripped)),
                Err(e) => Err(e.into()),
            }
        }
        (false, false) => Ok(StripSuccess::NoChange),
        (false, true) => {
            let f = fs::File::create(nb_path).map_err(NBWriteError::from)?;
            let writer = BufWriter::new(f);
            match write_nb(writer, &strip_nb) {
                Ok(()) => Ok(StripSuccess::Stripped),
                Err(e) => Err(e.into()),
            }
        }
    }
}
fn write_nb<W, T>(mut writer: W, value: &T) -> Result<(), NBWriteError>
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
    pub fn from_stripped(stripped: bool) -> Self {
        if stripped {
            Self::Stripped
        } else {
            Self::NoChange
        }
    }
}
