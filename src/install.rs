use anyhow::Error;

use std::{fs, io::BufWriter, path::Path, path::PathBuf};

use bstr::BStr;
use gix_config::parse::section::Key;
use gix_config::Source;
use std::{collections::BTreeMap, io::Write};

use itertools::Itertools;

use crate::cli::GitConfigType;

pub fn install_config(config_type: GitConfigType) -> Result<(), Error> {
    let cur_exe = std::env::current_exe().unwrap();
    let cur_dir = std::env::current_dir().unwrap();
    let cur_exe_str = cur_exe.to_str().unwrap().replace('\\', "/");
    let source = match config_type {
        GitConfigType::Global => Source::User,
        GitConfigType::System => Source::System,
        GitConfigType::Local => Source::Local,
    };

    let file_path: PathBuf = match config_type {
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

fn resolve_attribute_file(
    config_type: GitConfigType,
    attribute_file: Option<&Path>,
) -> Result<PathBuf, Error> {
    if let Some(path) = attribute_file {
        Ok(path.to_owned())
    } else {
        let cur_dir = std::env::current_dir().unwrap();

        let source = match config_type {
            GitConfigType::Global => gix_attributes::Source::Git,
            GitConfigType::Local => gix_attributes::Source::Local,
            GitConfigType::System => gix_attributes::Source::System,
        };

        let file_path: PathBuf = match config_type {
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

pub fn install_attributes(
    config_type: GitConfigType,
    attribute_file: Option<&Path>,
) -> Result<(), Error> {
    let file_path = resolve_attribute_file(config_type, attribute_file)?;
    let attribute_bytes = fs::read(&file_path)?;

    let to_add_lines = &[
        "*.ipynb filter=wipers",
        "*.zpln filter=wipers",
        "*.ipynb diff=wipers",
    ];
    // let to_add_str = to_add_lines.join("\n").as_bytes();

    let to_add_values = to_add_lines
        .iter()
        .map(|x| gix_attributes::parse(x.as_bytes()).next().unwrap().unwrap())
        .flat_map(|(kind, rhs, _)| {
            rhs.filter_map(Result::ok)
                .map(move |a| (kind.clone(), a.to_owned()))
        });

    let mut to_add: BTreeMap<_, _> = to_add_values.zip(to_add_lines).collect();
    let extra_newline = attribute_bytes.last() == Some(&b'\n');

    let mut lines = gix_attributes::parse(&attribute_bytes);

    for (kind, x, _) in lines.by_ref().filter_map(Result::ok) {
        //
        for ass in x.filter_map(Result::ok) {
            to_add.remove(&(kind.clone(), ass.to_owned()));
        }
    }
    println!("{to_add:?}");
    if !to_add.is_empty() {
        let mut writer = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .unwrap();
        let extra = if extra_newline { "" } else { "\n" };
        writeln!(writer, "{}{}", extra, to_add.values().join("\n"))?;
    }

    Ok(())
}

#[cfg(test)]
mod test {}
