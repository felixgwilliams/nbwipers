#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::{
    collections::BTreeMap,
    fs,
    io::BufWriter,
    path::{Path, PathBuf},
};

use crate::{cli::GitConfigType, settings::Settings};
use anyhow::Error;
use bstr::BStr;
use clap::Parser;
use cli::{CheckAllCommand, CheckCommand, CleanCommand, Commands, CommonArgs, InstallCommand};
use files::find_notebooks;
use gix_config::parse::section::Key;
use gix_config::Source;
use rayon::prelude::*;

mod check;
mod cli;
mod config;
mod files;
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

fn check(file: &Path, cli: CommonArgs) -> Result<(), Error> {
    let (args, overrides) = cli.partition();

    let settings = Settings::construct(args.config.as_deref(), &overrides);
    let nb = types::read_nb(file)?;
    let check_results = BTreeMap::from_iter([(file, check::check_nb(&nb, &settings))]);
    println!("{check_results:?}");

    Ok(())
}

fn strip(files: &[PathBuf], cli: CommonArgs) -> Result<(), Error> {
    let (args, overrides) = cli.partition();
    let nbs = find_notebooks(files)?;

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

fn install(cmd: &InstallCommand) -> Result<(), Error> {
    let cur_exe = std::env::current_exe().unwrap();
    let cur_dir = std::env::current_dir().unwrap();
    let cur_exe_str = cur_exe.to_str().unwrap().replace('\\', "/");
    let source = match cmd.config_type {
        GitConfigType::Global => Source::User,
        GitConfigType::System => Source::System,
        GitConfigType::Local => Source::Local,
    };

    let file_path: PathBuf = match cmd.config_type {
        GitConfigType::Global | GitConfigType::System => source
            .storage_location(&mut gix_path::env::var)
            .as_deref()
            .unwrap()
            .to_owned(),
        GitConfigType::Local => {
            let dotgit = gix_discover::upwards(&cur_dir)?
                .0
                .into_repository_and_work_tree_directories()
                .0;
            dotgit.join(source.storage_location(&mut gix_path::env::var).unwrap())
        }
    };
    let mut file = gix_config::File::from_path_no_includes(file_path.clone(), source).unwrap();

    let mut wipers_section = file
        .section_mut_or_create_new("filter", Some("wipers".into()))
        .unwrap();

    wipers_section.set(
        Key::try_from("clean").unwrap(),
        BStr::new(format!("\"{}\" clean", cur_exe_str.as_str()).as_str()),
    );
    wipers_section.set(Key::try_from("smudge").unwrap(), BStr::new("cat"));

    let mut diff_section = file
        .section_mut_or_create_new("diff", Some("wipers".into()))
        .unwrap();

    diff_section.set(
        Key::try_from("textconv").unwrap(),
        BStr::new(format!("\"{}\" clean -t", cur_exe_str.as_str()).as_str()),
    );
    println!("Writing to {}", file_path.display());
    let mut stdout = BufWriter::new(fs::File::create(file_path)?);
    file.write_to(&mut stdout).unwrap();

    Ok(())
}

fn main() -> Result<(), Error> {
    let cli = cli::Cli::parse();
    match cli.command {
        Commands::Check(CheckCommand { ref file, common }) => check(file, common),
        Commands::Clean(CleanCommand { ref files, common }) => strip(files, common),
        Commands::CheckAll(CheckAllCommand { ref files, common }) => check_all(files, common),
        Commands::Install(ref cmd) => install(cmd),
    }
}

#[cfg(test)]
mod test {

    use bstr::BStr;
    use gix_config::parse::section::Key;

    #[test]
    fn test_git_config() {
        let cur_exe = std::env::current_exe().unwrap();
        let cur_dir = std::env::current_dir().unwrap();
        let cur_exe_str = cur_exe.to_str().unwrap().replace('\\', "/");

        println!("{cur_exe_str}");
        println!("{cur_dir:?}");
        // let global = gix_config::File::from_path_no_includes(
        //     gix_config::Source::User
        //         .storage_location(&mut gix_path::env::var)
        //         .unwrap()
        //         .into(),
        //     gix_config::Source::User,
        // )
        // .unwrap();
        // println!("{global:?}");
        // println!("{local:?}");
        let Some((dotgit_dir, _)) = gix_discover::upwards(&cur_dir)
            .ok()
            .map(|(g, _)| g.into_repository_and_work_tree_directories())
        else {
            return;
        };
        println!("{dotgit_dir:?}",);
        // let mut local = gix_config::File::from_git_dir(dotgit_dir).unwrap();
        let local_source = gix_config::Source::Local;
        let mut local = gix_config::File::from_path_no_includes(
            dotgit_dir.join(
                local_source
                    .storage_location(&mut gix_path::env::var)
                    .unwrap(),
            ),
            local_source,
        )
        .unwrap();

        let mut wipers_section = local
            .section_mut_or_create_new("filter", Some("wipers".into()))
            .unwrap();

        wipers_section.set(
            Key::try_from("clean").unwrap(),
            BStr::new(format!("\"{}\" clean", cur_exe_str.as_str()).as_str()),
        );
        wipers_section.set(Key::try_from("smudge").unwrap(), BStr::new("cat"));

        println!("{wipers_section:?}");
        let mut diff_section = local
            .section_mut_or_create_new("diff", Some("wipers".into()))
            .unwrap();

        diff_section.set(
            Key::try_from("textconv").unwrap(),
            BStr::new(format!("\"{}\" clean -t", cur_exe_str.as_str()).as_str()),
        );

        let mut stdout = std::io::stdout();
        local.write_to(&mut stdout).unwrap();
    }
}
