#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::settings::Settings;
use clap::Parser;
use cli::{CheckAllCommand, CheckCommand, CleanCommand, Commands, CommonArgs};
use files::find_notebooks;
use rayon::prelude::*;

mod check;
mod cli;
mod config;
mod files;
mod settings;
mod strip;
mod types;

fn check_all(files: &[PathBuf], cli: CommonArgs) -> Result<(), String> {
    let (args, overrides) = cli.partition();
    let nbs = find_notebooks(files).ok_or("Please specify a path")?;

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    // let check_results = BTreeMap::
    let check_results_iter: Vec<_> = nbs
        .par_iter()
        .filter_map(|nb_path| {
            // println!("{nb_path:?}");
            let Ok(nb) = types::read_nb(nb_path) else {
                return None;
            };
            let check_results = check::check_nb(&nb, &settings);
            Some((nb_path.as_path(), check_results))
        })
        .collect();
    let check_results_dict = BTreeMap::from_iter(check_results_iter);

    println!("{check_results_dict:?}");

    Ok(())
}

fn check(file: &Path, cli: CommonArgs) -> Result<(), String> {
    let (args, overrides) = cli.partition();

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    let nb = types::read_nb(file).map_err(|e| e.to_string())?;
    let check_results = BTreeMap::from_iter([(file, check::check_nb(&nb, &settings))]);
    println!("{check_results:?}");

    Ok(())
}

fn strip(files: &[PathBuf], cli: CommonArgs) -> Result<(), String> {
    let (args, overrides) = cli.partition();
    let nbs = find_notebooks(files).ok_or("Please specify a path")?;

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    nbs.par_iter().for_each(|nb_path| {
        println!("{nb_path:?}");
        let Ok(nb) = types::read_nb(nb_path) else {
            return;
        };

        let nb = strip::strip_nb(nb, &settings);

        println!("{}", serde_json::to_string_pretty(&nb).unwrap());
    });
    Ok(())
}

fn main() -> Result<(), String> {
    let cli = cli::Cli::parse();
    match cli.command {
        Commands::Check(CheckCommand { ref file, common }) => check(file, common),
        Commands::Clean(CleanCommand { ref files, common }) => strip(files, common),
        Commands::CheckAll(CheckAllCommand { ref files, common }) => check_all(files, common),
    }
}
