#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![warn(clippy::unwrap_used)]

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::settings::Settings;
use anyhow::Error;
use clap::Parser;
use cli::{CheckAllCommand, CleanAllCommand, CleanCommand, Commands, CommonArgs, InstallCommand};
use files::find_notebooks;
use rayon::prelude::*;
use strip::strip_single;
use types::StripResult;

mod check;
mod cli;
mod config;
mod files;
mod install;
mod settings;
mod strip;
mod types;

fn check_all(files: &[PathBuf], cli: CommonArgs) -> Result<(), Error> {
    let (args, overrides) = cli.partition();
    let nbs = find_notebooks(files)?;

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    // let check_results = BTreeMap::
    let check_results_iter: Vec<_> = nbs
        .par_iter()
        .map(|nb_path| {
            // println!("{nb_path:?}");
            match types::read_nb(nb_path) {
                Ok(nb) => (nb_path.as_path(), check::check_nb(&nb, &settings)),
                Err(e) => (nb_path.as_path(), vec![e.into()]),
            }
        })
        .collect();
    let check_results_dict = BTreeMap::from_iter(check_results_iter);

    println!("{check_results_dict:?}");

    Ok(())
}

fn strip_all(files: &[PathBuf], cli: CommonArgs) -> Result<(), Error> {
    let (args, overrides) = cli.partition();
    let nbs = find_notebooks(files)?;

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    let strip_results: Vec<StripResult> = nbs
        .par_iter()
        .map(|nb_path| strip_single(nb_path, false, &settings).into())
        .collect();

    for (nb_path, res) in nbs.iter().zip(strip_results) {
        println!("{}: {}", nb_path.display(), res);
    }
    Ok(())
}

fn strip(file: &Path, textconv: bool, cli: CommonArgs) -> Result<(), Error> {
    let (args, overrides) = cli.partition();

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    strip_single(file, textconv, &settings)?;

    Ok(())
}

fn install(cmd: &InstallCommand) -> Result<(), Error> {
    install::install_config(cmd.config_type)?;
    install::install_attributes(cmd.config_type, cmd.attribute_file.as_deref())
}

fn main() -> Result<(), Error> {
    let cli = cli::Cli::parse();
    match cli.command {
        // Commands::Check(CheckCommand { ref file, common }) => check(file, common),
        Commands::Clean(CleanCommand {
            ref file,
            textconv,
            common,
        }) => strip(file, textconv, common),
        Commands::CleanAll(CleanAllCommand { ref files, common }) => strip_all(files, common),
        Commands::Check(CheckAllCommand { ref files, common }) => check_all(files, common),
        Commands::Install(ref cmd) => install(cmd),
    }
}

#[cfg(test)]
mod test {}
