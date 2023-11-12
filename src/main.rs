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

#[derive(Clone, Debug, Serialize)]
pub enum CheckResult {
    StripMeta,
    DropCells,
    ClearOutput,
    ClearCount,
    ClearId,
    CellStripMeta,
}

fn check(nb: &RawNotebook, settings: &Settings) -> Vec<CheckResult> {
    let (cell_keys, meta_keys) = partition_extra_keys(settings.extra_keys.as_slice());
    let mut out = vec![];
    let strip_meta = meta_keys
        .iter()
        .any(|k| wipers::get_value_child(&nb.metadata, k.get_parts()).is_some());
    if strip_meta {
        out.push(CheckResult::StripMeta);
    }
    let drop_cells = nb
        .cells
        .iter()
        .any(|c| c.should_drop(settings.drop_empty_cells, &settings.drop_tagged_cells));
    if drop_cells {
        out.push(CheckResult::DropCells);
    }
    let clear_output = nb
        .cells
        .iter()
        .filter_map(|c| c.as_codecell())
        .any(|c| c.is_clear_outputs() || !c.should_clear_output(settings.drop_output));
    if clear_output {
        out.push(CheckResult::ClearOutput);
    }
    let clear_count = settings.drop_count
        && nb
            .cells
            .iter()
            .filter_map(|c| c.as_codecell())
            .any(|c| !c.is_clear_exec_count());
    if clear_count {
        out.push(CheckResult::ClearCount);
    }
    let clear_id = settings.drop_id
        && nb
            .cells
            .iter()
            .filter_map(|c| c.as_codecell())
            .any(|c| !c.is_clear_id());
    if clear_id {
        out.push(CheckResult::ClearId);
    }
    let strip_cell_meta = nb.cells.iter().any(|c| {
        cell_keys
            .iter()
            .any(|k| wipers::get_value_child(c.get_metadata(), k.get_parts()).is_some())
    });
    if strip_cell_meta {
        out.push(CheckResult::CellStripMeta);
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
