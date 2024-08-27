#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![warn(clippy::unwrap_used)]
#![warn(missing_docs)]
#![allow(clippy::multiple_crate_versions)] // can't do anything about these

/*! nbwipers is a command line tool to wipe clean Jupyter Notebooks
 *
 *
 */

use std::{
    // fmt::write,
    path::{Path, PathBuf},
};

use crate::settings::Settings;
use anyhow::{anyhow, bail, Error};
use check::PathCheckResult;
use clap::Parser;
use cli::{
    resolve_bool_arg, CheckCommand, CheckInstallCommand, CleanAllCommand, CleanCommand, Commands,
    CommonArgs, InstallCommand, OutputFormat, ShowConfigCommand, UninstallCommand,
};
use colored::Colorize;
use config::resolve;
use files::{find_notebooks, read_nb, read_nb_stdin, relativize_path, FoundNotebooks};
use hooks::hooks;
use rayon::prelude::*;
use std::io::Write;
use strip::{strip_single, StripResult};

mod cell_impl;
mod check;
mod cli;
mod config;
mod extra_keys;
mod files;
mod hooks;
mod install;
mod schema;
mod settings;
mod strip;
mod utils;

fn check_all(
    files: &[PathBuf],
    output_format: Option<OutputFormat>,
    cli: CommonArgs,
) -> Result<(), Error> {
    let output_format = output_format.unwrap_or_default();
    let (args, overrides) = cli.partition();
    let settings = Settings::construct(args.config.as_deref(), &overrides)?;
    let nbs = find_notebooks(files, &settings)?;
    let check_results_by_file = match nbs {
        FoundNotebooks::Stdin => match read_nb_stdin() {
            Ok(nb) => vec![(Path::new("-"), check::check_nb(&nb, &settings))],
            Err(e) => vec![(Path::new("-"), vec![e.into()])],
        },
        FoundNotebooks::NoFiles => {
            if args.allow_no_notebooks {
                return Ok(());
            }
            bail!("Could not find any notebooks in path(s)")
        }
        FoundNotebooks::Files(ref nbs) => {
            nbs.par_iter()
                .map(|nb_path| {
                    // println!("{nb_path:?}");
                    match read_nb(nb_path) {
                        Ok(nb) => (nb_path.as_path(), check::check_nb(&nb, &settings)),
                        Err(e) => (nb_path.as_path(), vec![e.into()]),
                    }
                })
                .collect()
        }
    };
    let mut check_results = Vec::new();

    for (path, res) in &check_results_by_file {
        check_results.extend(res.iter().map(|result| PathCheckResult { path, result }));
    }

    match output_format {
        OutputFormat::Text => {
            for PathCheckResult { path, result } in &check_results {
                let rel_path = relativize_path(path).bold();
                println!("{rel_path}:{result}");
            }
        }
        OutputFormat::Json => print!("{}", serde_json::to_string_pretty(&check_results)?),
    }

    if check_results.is_empty() {
        Ok(())
    } else {
        let n_checks = check_results.len();

        Err(anyhow!("Found {n_checks} items to strip"))
    }
}

fn strip_all(files: &[PathBuf], dry_run: bool, yes: bool, cli: CommonArgs) -> Result<(), Error> {
    let (args, overrides) = cli.partition();
    let settings = Settings::construct(args.config.as_deref(), &overrides)?;
    let FoundNotebooks::Files(nbs) = find_notebooks(files, &settings)? else {
        bail!("`strip-all` does not support stdin");
    };
    if !yes {
        let ans = inquire::Confirm::new("Continue?")
            .with_default(false)
            .prompt()?;
        if !ans {
            return Ok(());
        }
    }

    let strip_results: Vec<StripResult> = nbs
        .par_iter()
        .map(|nb_path| strip_single(nb_path, dry_run, &settings).into())
        .collect();

    let any_errors = strip_results.iter().any(StripResult::is_err);

    for (nb_path, res) in nbs.iter().zip(strip_results) {
        let rel_path = relativize_path(nb_path).bold();

        println!("{rel_path}: {res}");
    }
    if any_errors {
        bail!("IO Errors found")
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
    install::install_config(cmd.git_config_file.as_deref(), cmd.config_type)?;
    install::install_attributes(cmd.config_type, cmd.attribute_file.as_deref())
}

fn uninstall(cmd: &UninstallCommand) -> Result<(), Error> {
    install::uninstall_config(cmd.git_config_file.as_deref(), cmd.config_type)?;
    install::uninstall_attributes(cmd.config_type, cmd.attribute_file.as_deref())
}

fn check_install(cmd: &CheckInstallCommand) -> Result<(), Error> {
    let check_result = cmd
        .config_type
        .map_or_else(install::check_install_none_type, |config_type| {
            install::check_install_some_type(config_type)
        });
    if install::check_should_exit_zero(cmd.exit_zero) {
        Ok(())
    } else {
        check_result
    }
}

fn show_config(common: CommonArgs, show_all: bool) -> Result<(), Error> {
    let (args, overrides) = common.partition();
    let settings_str = if show_all {
        let settings = Settings::construct(args.config.as_deref(), &overrides)?;
        toml::to_string(&settings)?
    } else {
        let (config_sec, config_path) = resolve(args.config.as_deref())?;
        let mut config = config_sec.make_configuration(config_path.as_deref());
        config = overrides.override_config(config);
        toml::to_string(&config)?
    };

    let mut stdout = std::io::stdout();
    writeln!(stdout, "{settings_str}")?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let cli = cli::Cli::parse();

    #[cfg(feature = "markdown-help")]
    if cli.markdown_help {
        clap_markdown::print_help_markdown::<cli::Cli>();
        return Ok(());
    }
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
        Commands::Check(CheckCommand {
            ref files,
            output_format,
            common,
        }) => check_all(files, output_format, common),
        Commands::Install(ref cmd) => install(cmd),
        Commands::Uninstall(ref cmd) => uninstall(cmd),
        Commands::CheckInstall(ref cmd) => check_install(cmd),
        Commands::ShowConfig(ShowConfigCommand {
            common,
            show_all,
            no_show_defaults,
        }) => show_config(
            common,
            resolve_bool_arg(show_all, no_show_defaults).unwrap_or(false),
        ),
        Commands::Hook(ref cmd) => hooks(cmd),
    }
}

#[cfg(test)]
mod test {}
#[allow(clippy::unwrap_used)]
#[cfg(test)]
pub(crate) mod test_helpers {
    use lazy_static::lazy_static;
    use std::{env::set_current_dir, path::Path, sync::Mutex};
    lazy_static! {
        pub static ref CWD_MUTEX: Mutex<()> = Mutex::new(());
    }

    pub fn with_dir<P: AsRef<Path>, T: Sized>(dir: P, f: impl FnOnce() -> T) -> T {
        let _lock = CWD_MUTEX.lock().unwrap();
        let cur_dir = crate::files::get_cwd();
        dbg!(&cur_dir);
        set_current_dir(&dir).unwrap();
        dbg!(dir.as_ref());
        let res = f();
        dbg!(dir.as_ref());
        set_current_dir(cur_dir).unwrap();
        res
    }
}
