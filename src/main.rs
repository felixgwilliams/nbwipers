#![warn(clippy::all, clippy::pedantic)]

use clap::Parser;

use crate::settings::Settings;
use crate::wipers::ExtraKey;

mod cli;
mod config;
mod settings;
mod wipers;
fn main() {
    let cli = cli::Cli::parse();
    let (args, overrides) = cli.partition();

    let Ok(mut nb) = wipers::read_nb(&args.notebook) else {
        return;
    };

    let settings = Settings::construct(args.config.as_deref(), &overrides);

    let mut meta_keys = vec![];
    let mut cell_keys = vec![];
    for extra_key in settings.extra_keys {
        match extra_key {
            ExtraKey::CellMeta(ref _cell_key) => cell_keys.push(extra_key),
            ExtraKey::Metadata(ref _meta_key) => meta_keys.push(extra_key),
        };
    }
    for meta_key in &meta_keys {
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
        cell.clear_counts();
        cell.clear_outputs();
        assert!(cell.is_clear_outputs());
        assert!(cell.is_clear_exec_count());
        wipers::pop_value_child(&mut cell.metadata, &["collapsed"]);
        for cell_key in &cell_keys {
            wipers::pop_cell_key(cell, cell_key);
        }
    }
    println!("{}", serde_json::to_string_pretty(&nb).unwrap());
}
