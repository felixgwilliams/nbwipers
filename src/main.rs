#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use crate::settings::Settings;
use clap::Parser;
use cli::{CheckCommand, CleanCommand, Commands, CommonArgs};
use files::find_notebooks;
use rayon::prelude::*;

mod check;
mod cli;
mod config;
mod files;
mod settings;
mod strip;
mod types;

fn check(cli: CommonArgs) -> Result<(), String> {
    let (args, overrides) = cli.partition();
    let nbs = find_notebooks(&args.files).ok_or("Please specify a path")?;

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    nbs.par_iter().for_each(|nb_path| {
        println!("{nb_path:?}");
        let Ok(nb) = types::read_nb(nb_path) else {
            return;
        };
        let check_results = check::check_nb(&nb, &settings);
        println!("{check_results:?}");
    });
    Ok(())
}

fn strip(cli: CommonArgs) -> Result<(), String> {
    let (args, overrides) = cli.partition();
    let nbs = find_notebooks(&args.files).ok_or("Please specify a path")?;

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
        Commands::Clean(CleanCommand { common, .. }) => strip(common),
        Commands::Check(CheckCommand { common, .. }) => check(common),
    }
}
