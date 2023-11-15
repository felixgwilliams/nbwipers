use crate::{
    settings::Settings,
    types::{partition_extra_keys, pop_cell_key, pop_meta_key, pop_value_child, RawNotebook},
};
pub fn strip_nb(mut nb: RawNotebook, settings: &Settings) -> RawNotebook {
    let (cell_keys, meta_keys) = partition_extra_keys(settings.extra_keys.as_slice());

    for meta_key in meta_keys {
        pop_meta_key(&mut nb, meta_key);
    }

    let drop_cells: Vec<_> = nb
        .cells
        .iter()
        .map(|c| c.should_drop(settings.drop_empty_cells, &settings.drop_tagged_cells))
        .collect();
    if drop_cells.iter().any(|b| *b) {
        let mut retained_cells = vec![];
        for (cell, to_drop) in nb.cells.into_iter().zip(drop_cells.iter()) {
            if !to_drop {
                retained_cells.push(cell);
            }
        }
        nb.cells = retained_cells;
    }

    for cell in nb.cells.iter_mut().filter_map(|x| x.as_codecell_mut()) {
        if cell.should_clear_output(settings.drop_output) {
            cell.clear_outputs();
        }
        if settings.drop_count {
            cell.clear_counts();
        }

        pop_value_child(&mut cell.metadata, &["collapsed"]);
        for cell_key in &cell_keys {
            pop_cell_key(cell, cell_key);
        }
    }
    nb
}
