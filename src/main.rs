#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![warn(clippy::unwrap_used)]

use std::{
    collections::BTreeMap,
    fs,
    io::BufWriter,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{settings::Settings, strip::strip_nb};
use anyhow::Error;
use clap::Parser;
use cli::{CheckAllCommand, CleanAllCommand, CleanCommand, Commands, CommonArgs, InstallCommand};
use files::find_notebooks;
use rayon::prelude::*;
use serde::Serialize;

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

fn strip_all(files: &[PathBuf], textconv: bool, cli: CommonArgs) -> Result<(), Error> {
    let (args, overrides) = cli.partition();
    let nbs = find_notebooks(files)?;

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    nbs.par_iter()
        .map(|nb_path| {
            println!("{nb_path:?}");
            let nb = types::read_nb(nb_path)?;

            let (strip_nb, stripped) = strip::strip_nb(nb, &settings);
            if textconv {
                let stdout = std::io::stdout();
                write_nb(stdout, &strip_nb)?;
            } else if stripped {
                let writer = BufWriter::new(fs::File::create(nb_path)?);
                write_nb(writer, &strip_nb)?;
            }
            Ok(())
        })
        .collect()
}

fn write_nb<W, T>(mut writer: W, value: &T) -> anyhow::Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
{
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut ser = serde_json::Serializer::with_formatter(&mut writer, formatter);
    value.serialize(&mut ser)?;
    writeln!(writer)?;
    Ok(())
}

fn strip(file: &Path, textconv: bool, cli: CommonArgs) -> Result<(), Error> {
    let (args, overrides) = cli.partition();

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    let nb = types::read_nb(file)?;
    let (strip_nb, stripped) = strip_nb(nb, &settings);

    if textconv {
        let stdout = std::io::stdout();
        write_nb(stdout, &strip_nb)?;
    } else if stripped {
        let writer = BufWriter::new(fs::File::create(file)?);
        write_nb(writer, &strip_nb)?;
    }

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
        Commands::CleanAll(CleanAllCommand {
            ref files,
            textconv,
            common,
        }) => strip_all(files, textconv, common),
        Commands::Check(CheckAllCommand { ref files, common }) => check_all(files, common),
        Commands::Install(ref cmd) => install(cmd),
    }
}

#[cfg(test)]
mod test {}
