use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};

use serde::Serialize;

use crate::{
    settings::Settings,
    types::{
        partition_extra_keys, pop_cell_key, pop_meta_key, pop_value_child, read_nb, NBWriteError,
        RawNotebook, StripError, StripSuccess,
    },
};
pub fn strip_nb(mut nb: RawNotebook, settings: &Settings) -> (RawNotebook, bool) {
    let (cell_keys, meta_keys) = partition_extra_keys(settings.extra_keys.as_slice());
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

    for cell in nb.cells.iter_mut().filter_map(|x| x.as_codecell_mut()) {
        if cell.should_clear_output(settings.drop_output) && !cell.is_clear_outputs() {
            stripped = true;

            cell.clear_outputs();
        }
        if settings.drop_count && !cell.is_clear_exec_count() {
            stripped = true;

            cell.clear_counts();
        }

        pop_value_child(&mut cell.metadata, &["collapsed"]);
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
