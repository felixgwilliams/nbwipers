use anyhow::{anyhow, bail, Error};
use gix_attributes::{parse::Kind, AssignmentRef, StateRef};

use std::{
    fmt::Write as _,
    fs,
    io::{BufRead, BufReader, BufWriter},
    path::Path,
    path::PathBuf,
};

use bstr::BStr;
use gix_config::parse::section::Key;
use gix_config::Source;
use std::{collections::BTreeMap, io::Write};

use itertools::Itertools;

use crate::cli::GitConfigType;

impl From<GitConfigType> for Source {
    fn from(value: GitConfigType) -> Self {
        match value {
            GitConfigType::Global => Source::User,
            GitConfigType::System => Source::System,
            GitConfigType::Local => Source::Local,
        }
    }
}

pub fn install_config(config_file: Option<&Path>, config_type: GitConfigType) -> Result<(), Error> {
    let cur_exe = std::env::current_exe()?;
    let source = config_type.into();
    let cur_exe_str = cur_exe
        .to_str()
        .ok_or_else(|| (anyhow!("Executable path cannot be converted to unicode")))?
        .replace('\\', "/");
    let file_path = resolve_config_file(config_file, config_type)?;

    let mut file = if file_path.is_file() {
        gix_config::File::from_path_no_includes(file_path.clone(), source)?
    } else {
        gix_config::File::new(gix_config::file::Metadata::from(source))
    };

    // fails for invalid section names. This one is ok
    #[allow(clippy::unwrap_used)]
    let mut nbwipers_section = file
        .section_mut_or_create_new("filter", Some("nbwipers".into()))
        .unwrap();
    // fails for invalid section names. This one is ok
    #[allow(clippy::unwrap_used)]
    nbwipers_section.set(
        Key::try_from("clean").unwrap(),
        BStr::new(format!("\"{}\" clean -", cur_exe_str.as_str()).as_str()),
    );
    #[allow(clippy::unwrap_used)]
    nbwipers_section.set(Key::try_from("smudge").unwrap(), BStr::new("cat"));

    // fails for invalid section names. This one is ok
    #[allow(clippy::unwrap_used)]
    let mut diff_section = file
        .section_mut_or_create_new("diff", Some("nbwipers".into()))
        .unwrap();

    // fails for invalid section names. This one is ok
    #[allow(clippy::unwrap_used)]
    diff_section.set(
        Key::try_from("textconv").unwrap(),
        BStr::new(format!("\"{}\" clean -t", cur_exe_str.as_str()).as_str()),
    );
    println!("Writing to {}", file_path.display());
    let mut writer = BufWriter::new(fs::File::create(file_path)?);
    file.write_to(&mut writer)?;

    Ok(())
}

fn resolve_config_file(
    config_file: Option<&Path>,
    config_type: GitConfigType,
) -> Result<PathBuf, Error> {
    if let Some(config_file) = config_file {
        Ok(config_file.to_path_buf())
    } else {
        let source: Source = config_type.into();
        let cur_dir = std::env::current_dir()?;
        #[allow(clippy::unwrap_used)]
        let file_path = match config_type {
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
        Ok(file_path)
    }
}

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

pub fn uninstall_config(
    config_file: Option<&Path>,
    config_type: GitConfigType,
) -> Result<(), Error> {
    let source = config_type.into();
    let file_path = resolve_config_file(config_file, config_type)?;
    let mut file = if file_path.exists() {
        gix_config::File::from_path_no_includes(file_path.clone(), source)?
    } else {
        println!("Config file does not exist. nothing to do.");
        return Ok(());
    };

    let filter_removed = file
        .remove_section("filter", Some("nbwipers".into()))
        .is_some();
    let diff_removed = file
        .remove_section("diff", Some("nbwipers".into()))
        .is_some();
    if filter_removed || diff_removed {
        println!("Writing to {}", file_path.display());
        let mut writer = BufWriter::new(fs::File::create(file_path)?);
        file.write_to(&mut writer)?;
    }

    Ok(())
}
#[cfg(test)]
mod test {}
