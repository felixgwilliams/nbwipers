use clap::Parser;

use crate::wipers::ExtraKey;

mod cli;
mod options;
mod wipers;

fn main() {
    let cli = cli::Cli::parse();

    let Ok(mut nb) = wipers::read_nb(&cli.notebook) else {
        return;
    };
    let extra_keys = cli.extra_keys.unwrap_or_default();
    let mut meta_keys = vec![];
    let mut cell_keys = vec![];
    for extra_key in extra_keys {
        match extra_key {
            ExtraKey::CellMeta(ref _cell_key) => cell_keys.push(extra_key),
            ExtraKey::Metadata(ref _meta_key) => meta_keys.push(extra_key),
        };
    }
    // TODO: notebook meta keys
    for cell in nb.cells.iter_mut().filter_map(|x| x.as_codecell_mut()) {
        cell.clear_counts();
        cell.clear_outputs();
        assert!(cell.is_clear_outputs());
        assert!(cell.is_clear_exec_count());
        wipers::pop_value_child(&mut cell.metadata, &["collapsed"]);
        for cell_key in cell_keys.iter() {
            wipers::pop_cell_key(cell, cell_key);
        }
    }
    println!("{}", serde_json::to_string_pretty(&nb).unwrap());
}
