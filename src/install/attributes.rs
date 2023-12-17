use anyhow::{bail, Error};
use gix_attributes::{parse::Kind, AssignmentRef, StateRef};

use std::{
    fmt::Write as _,
    fs,
    io::{BufRead, BufReader},
    path::Path,
    path::PathBuf,
};

use std::{collections::BTreeMap, io::Write};

use super::{get_git_repo_and_work_tree, InstallStatus};
use crate::cli::GitConfigType;
use itertools::Itertools;

fn resolve_attribute_file(
    config_type: GitConfigType,
    attribute_file: Option<&Path>,
) -> Result<PathBuf, Error> {
    if let Some(path) = attribute_file {
        Ok(path.to_owned())
    } else {
        let cur_dir = std::env::current_dir()?;

        let source = match config_type {
            GitConfigType::Global => gix_attributes::Source::Git,
            GitConfigType::Local => gix_attributes::Source::Local,
            GitConfigType::System => gix_attributes::Source::System,
        };

        let file_path: PathBuf = match config_type {
            #[allow(clippy::unwrap_used)]
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
                #[allow(clippy::unwrap_used)]
                dotgit.join(source.storage_location(&mut gix_path::env::var).unwrap())
            }
        };
        Ok(file_path)
    }
}

fn get_default_attribute_file() -> Result<Option<PathBuf>, Error> {
    let Some(work_dir) = get_git_repo_and_work_tree()?.1 else {
        return Ok(None);
    };
    let attribute_file = work_dir.join(".gitattributes");
    if attribute_file.is_file() {
        Ok(Some(attribute_file))
    } else {
        Ok(None)
    }
}

const ATTRIBUTE_LINES: &[&str; 2] = &["*.ipynb filter=nbwipers", "*.ipynb diff=nbwipers"];

pub fn install_attributes(
    config_type: GitConfigType,
    attribute_file: Option<&Path>,
) -> Result<(), Error> {
    let file_path = resolve_attribute_file(config_type, attribute_file)?;
    if file_path.is_file() {
        let attribute_bytes = fs::read(&file_path)?;

        // let to_add_str = to_add_lines.join("\n").as_bytes();
        #[allow(clippy::unwrap_used)]
        let to_add_values = ATTRIBUTE_LINES
            .iter()
            .map(|x| gix_attributes::parse(x.as_bytes()).next().unwrap().unwrap())
            .flat_map(|(kind, rhs, _)| {
                rhs.filter_map(Result::ok)
                    .map(move |a| (kind.clone(), a.to_owned()))
            });

        let mut to_add: BTreeMap<_, _> = to_add_values.zip(ATTRIBUTE_LINES).collect();
        let extra = match attribute_bytes.last() {
            None | Some(&b'\n') => "",
            _ => "\n",
        };
        let lines = gix_attributes::parse(&attribute_bytes);

        for (kind, x, _) in lines.filter_map(Result::ok) {
            //
            for ass in x.filter_map(Result::ok) {
                to_add.remove(&(kind.clone(), ass.to_owned()));
            }
        }
        if !to_add.is_empty() {
            println!("Writing to {}", file_path.display());

            let mut writer = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)?;

            writeln!(writer, "{}{}", extra, to_add.values().join("\n"))?;
        }
    } else {
        println!("Writing to {}", file_path.display());
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut writer = fs::File::create(file_path)?;

        for line in ATTRIBUTE_LINES {
            writeln!(writer, "{line}")?;
        }
    }

    Ok(())
}

pub fn uninstall_attributes(
    config_type: GitConfigType,
    attribute_file: Option<&Path>,
) -> Result<(), Error> {
    let file_path = resolve_attribute_file(config_type, attribute_file)?;
    if file_path.is_file() {
        let f = BufReader::new(fs::File::open(&file_path)?);
        let mut out = String::new();
        let mut to_write = false;
        for line in f.lines() {
            let mut line = line?;
            if line.is_empty() {
                continue;
            }
            // let (kind, x, _) = gix_attributes::parse(line.as_bytes()).next().unwrap()?;
            let (kind, x, _) = match gix_attributes::parse(line.as_bytes()).next() {
                None => bail!("No pattern found in line"),
                Some(Ok(res)) => res,
                Some(res) => res?,
            };
            if let Kind::Pattern(patt) = kind {
                if patt.to_string() == "*.ipynb" {
                    let to_delete = x
                        .into_iter()
                        .map(|x| {
                            let x = x?;
                            if let StateRef::Value(s) = x.state {
                                Ok((x, s.as_bstr() == "nbwipers"))
                            } else {
                                Ok((x, false))
                            }
                        })
                        .collect::<Result<Vec<(AssignmentRef, bool)>, gix_attributes::name::Error>>(
                        )?;
                    if to_delete.iter().any(|(_, y)| *y) {
                        to_write = true;
                        let assignments = to_delete
                            .iter()
                            .filter(|(_x, y)| !y)
                            .map(|(x, _y)| x.to_string())
                            .join(" ");
                        if assignments.is_empty() {
                            line = String::new();
                        } else {
                            line = format!("{patt} {assignments}");
                        }
                    }
                }
            };
            if !line.is_empty() {
                writeln!(out, "{line}")?;
            }
        }
        if to_write {
            println!("Removing entries from {}", file_path.display());
            let mut f = fs::File::create(&file_path)?;
            write!(f, "{out}")?;
        }
    } else {
        println!("Attribute file does not exist. Nothing to do.");
    }
    Ok(())
}

fn check_attribute_file(attr_file_path: &Path) -> Result<InstallStatus, Error> {
    if attr_file_path.is_file() {
        let mut status = InstallStatus::default();

        let bytes = fs::read(attr_file_path)?;
        let lines = gix_attributes::parse(&bytes);
        for line in lines {
            let (kind, assignments, _) = line?;

            if let Kind::Pattern(patt) = kind {
                if patt.to_string() == "*.ipynb" {
                    for assignment in assignments {
                        let assignment = assignment?;
                        if let StateRef::Value(s) = assignment.state {
                            if s.as_bstr() == "nbwipers" {
                                status.nbwipers.filter |= assignment.name.as_str() == "filter";
                                status.nbwipers.diff |= assignment.name.as_str() == "diff";
                            }
                            if s.as_bstr() == "ipynb" {
                                status.nbstripout.diff |= assignment.name.as_str() == "diff";
                            }
                            if s.as_bstr() == "nbstripout" {
                                status.nbstripout.filter |= assignment.name.as_str() == "filter";
                            }
                        }
                    }
                }
            }
        }
        Ok(status)
    } else {
        Ok(InstallStatus::default())
    }
}

pub(super) fn check_install_attr_files(
    config_types: &[GitConfigType],
) -> Result<InstallStatus, Error> {
    let attribute_file = get_default_attribute_file().ok().flatten();
    let mut attr_install_status = match attribute_file {
        Some(ref attr) => check_attribute_file(attr)?,
        None => InstallStatus::default(),
    };

    for config_type in config_types {
        let attr_file_path = resolve_attribute_file(*config_type, None)?;

        attr_install_status |= check_attribute_file(&attr_file_path)?;
    }
    Ok(attr_install_status)
}
