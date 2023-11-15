use crate::{
    settings::Settings,
    types::{get_value_child, partition_extra_keys, CheckResult, RawNotebook},
};

pub fn check_nb(nb: &RawNotebook, settings: &Settings) -> Vec<CheckResult> {
    let (cell_keys, meta_keys) = partition_extra_keys(settings.extra_keys.as_slice());
    let mut out = vec![];

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

    if settings.drop_output {
        nb.cells
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.as_codecell().map(|c| (i, c)))
            .filter(|(_i, c)| !c.is_clear_outputs() && c.should_clear_output(settings.drop_output))
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
