#![warn(clippy::all, clippy::pedantic)]

use clap::Parser;
use serde::Serialize;
use wipers::RawNotebook;

use crate::settings::Settings;
use crate::wipers::partition_extra_keys;

mod cli;
mod config;
mod settings;
mod wipers;

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

fn check(nb: &RawNotebook, settings: &Settings) -> Vec<CheckResult> {
    let (cell_keys, meta_keys) = partition_extra_keys(settings.extra_keys.as_slice());
    let mut out = vec![];

    meta_keys
        .iter()
        .filter(|k| wipers::get_value_child(&nb.metadata, k.get_parts()).is_some())
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
            .filter(|(_i, c)| !c.is_clear_outputs() || c.should_clear_output(settings.drop_output))
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
            .filter(|k| wipers::get_value_child(cell.get_metadata(), k.get_parts()).is_some())
            .for_each(|k| {
                out.push(CheckResult::CellStripMeta {
                    cell_number,
                    extra_key: k.to_string(),
                });
            });
    }

    out
}

fn main() {
    let cli = cli::Cli::parse();
    let (args, overrides) = cli.partition();

    let Ok(mut nb) = wipers::read_nb(&args.notebook) else {
        return;
    };

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    let check_results = check(&nb, &settings);
    println!("{check_results:?}");
    println!("{}", serde_json::to_string_pretty(&check_results).unwrap());

    let (cell_keys, meta_keys) = partition_extra_keys(settings.extra_keys.as_slice());

    for meta_key in meta_keys {
        wipers::pop_meta_key(&mut nb, meta_key);
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

        wipers::pop_value_child(&mut cell.metadata, &["collapsed"]);
        for cell_key in &cell_keys {
            wipers::pop_cell_key(cell, cell_key);
        }
    }
    // println!("{}", serde_json::to_string_pretty(&nb).unwrap());
}
