#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![warn(clippy::unwrap_used)]
#![warn(missing_docs)]

/*! nbwipers is a command line tool to wipe clean Jupyter Notebooks
 *
 *
 */

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::settings::Settings;
use anyhow::{anyhow, Error};
use clap::Parser;
use cli::{CheckCommand, CleanAllCommand, CleanCommand, Commands, CommonArgs, InstallCommand};
use colored::Colorize;
use files::{find_notebooks, read_nb, relativize_path};
use rayon::prelude::*;
use strip::{strip_single, StripResult};

mod cell_impl;
mod check;
mod cli;
mod config;
mod extra_keys;
mod files;
mod install;
mod schema;
mod settings;
mod strip;
mod utils;

fn check_all(files: &[PathBuf], cli: CommonArgs) -> Result<(), Error> {
    let (args, overrides) = cli.partition();
    let nbs = find_notebooks(files)?;

    let settings = Settings::construct(args.config.as_deref(), &overrides)?;
    // let check_results = BTreeMap::
    let check_results_iter: Vec<_> = nbs
        .par_iter()
        .map(|nb_path| {
            // println!("{nb_path:?}");
            match read_nb(nb_path) {
                Ok(nb) => (nb_path.as_path(), check::check_nb(&nb, &settings)),
                Err(e) => (nb_path.as_path(), vec![e.into()]),
            }
        })
        .collect();

    let check_results_dict = BTreeMap::from_iter(check_results_iter);
    for (path, res) in &check_results_dict {
        let rel_path = relativize_path(path).bold();
        for item in res {
            println!("{rel_path}:{item}");
        }
    }
    if check_results_dict.is_empty() {
        Ok(())
    } else {
        let n_checks = check_results_dict.values().flatten().count();
        Err(anyhow!("Found {n_checks} items to strip"))
    }
}

fn strip_all(files: &[PathBuf], dry_run: bool, yes: bool, cli: CommonArgs) -> Result<(), Error> {
    let (args, overrides) = cli.partition();
    let nbs = find_notebooks(files)?;
    if !yes {
        let ans = inquire::Confirm::new("Continue?")
            .with_default(false)
            .prompt()?;
        if !ans {
            return Ok(());
        }
    }

    let settings = Settings::construct(args.config.as_deref(), &overrides)?;
    let strip_results: Vec<StripResult> = nbs
        .par_iter()
        .map(|nb_path| strip_single(nb_path, dry_run, &settings).into())
        .collect();

    for (nb_path, res) in nbs.iter().zip(strip_results) {
        let rel_path = relativize_path(nb_path).bold();

        println!("{rel_path}: {res}");
    }
    Ok(())
}

fn strip(file: &Path, textconv: bool, cli: CommonArgs) -> Result<(), Error> {
    let (args, overrides) = cli.partition();

    let settings = Settings::construct(args.config.as_deref(), &overrides)?;
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
        Commands::Clean(CleanCommand {
            ref file,
            textconv,
            common,
        }) => strip(file, textconv, common),
        Commands::CleanAll(CleanAllCommand {
            ref files,
            dry_run,
            yes,
            common,
        }) => strip_all(files, dry_run, yes, common),
        Commands::Check(CheckCommand { ref files, common }) => check_all(files, common),
        Commands::Install(ref cmd) => install(cmd),
    }
}

#[cfg(test)]
mod test {}
