#![warn(clippy::all, clippy::pedantic)]

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::settings::Settings;
use crate::wipers::partition_extra_keys;
use clap::Parser;
use ignore::WalkBuilder;
use itertools::Itertools;
use path_absolutize::Absolutize;
use rayon::prelude::*;
use serde::Serialize;
use wipers::RawNotebook;

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
/// Convert any path to an absolute path (based on the current working
/// directory).
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if let Ok(path) = path.absolutize() {
        return path.to_path_buf();
    }
    path.to_path_buf()
}
fn find_notebooks(paths: &[PathBuf]) -> Option<Vec<PathBuf>> {
    let paths: Vec<PathBuf> = paths.iter().map(normalize_path).unique().collect();
    let (first_path, rest_paths) = paths.split_first()?;

    let mut builder = WalkBuilder::new(first_path);
    for path in rest_paths {
        builder.add(path);
    }
    builder.standard_filters(true);
    builder.hidden(false);

    let walker = builder.build_parallel();
    let files: std::sync::Mutex<Vec<PathBuf>> = std::sync::Mutex::new(vec![]);
    walker.run(|| {
        Box::new(|path| {
            if let Ok(entry) = path {
                let resolved = if entry.file_type().map_or(true, |ft| ft.is_dir()) {
                    None
                } else if entry.depth() == 0 {
                    Some(entry.into_path())
                } else {
                    let cur_path = entry.into_path();
                    if cur_path.extension() == Some(OsStr::new("ipynb")) {
                        Some(cur_path)
                    } else {
                        None
                    }
                };
                if let Some(resolved) = resolved {
                    files.lock().unwrap().push(resolved);
                }
            }

            ignore::WalkState::Continue
        })
    });

    Some(files.into_inner().unwrap())
}

fn strip(mut nb: RawNotebook, settings: &Settings) -> RawNotebook {
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
    nb
}

fn main() {
    let cli = cli::Cli::parse();
    let (args, overrides) = cli.partition();
    let nbs = find_notebooks(&args.files);

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    nbs.unwrap_or_default().par_iter().for_each(|nb_path| {
        println!("{nb_path:?}");
        let Ok(nb) = wipers::read_nb(nb_path) else {
            return;
        };
        let check_results = check(&nb, &settings);
        println!("{check_results:?}");
        // println!("{}", serde_json::to_string(&check_results).unwrap());
        let nb = strip(nb, &settings);

        println!("{}", serde_json::to_string_pretty(&nb).unwrap());
    });
}
