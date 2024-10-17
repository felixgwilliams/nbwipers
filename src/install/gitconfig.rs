use anyhow::{anyhow, Error};

use std::{fs, io::BufWriter, path::Path, path::PathBuf};

use gix_config::{parse::section::ValueName, Source};

use super::{get_git_repo_and_work_tree, InstallStatus, InstallToolStatus};
use bstr::BStr;

use crate::cli::GitConfigType;

pub fn install_config(config_file: Option<&Path>, config_type: GitConfigType) -> Result<(), Error> {
    let cur_exe = std::env::current_exe()?;
    let source = config_type.into();
    let cur_exe_str = cur_exe
        .to_str()
        .map_or_else(
            || Err(anyhow!("Executable path cannot be converted to unicode")),
            Ok,
        )?
        .replace('\\', "/");
    let file_path = resolve_config_file(config_file, config_type)?;

    let mut file = if file_path.is_file() {
        gix_config::File::from_path_no_includes(file_path.clone(), source)?
    } else {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
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
        ValueName::try_from("clean").unwrap(),
        BStr::new(format!("\"{}\" clean -", cur_exe_str.as_str()).as_str()),
    );

    #[allow(clippy::unwrap_used)]
    nbwipers_section.set(
        ValueName::try_from("smudge").unwrap(),
        BStr::new(format!("\"{}\" smudge %f", cur_exe_str.as_str()).as_str()),
    );

    // fails for invalid section names. This one is ok
    #[allow(clippy::unwrap_used)]
    let mut diff_section = file
        .section_mut_or_create_new("diff", Some("nbwipers".into()))
        .unwrap();

    // fails for invalid section names. This one is ok
    #[allow(clippy::unwrap_used)]
    diff_section.set(
        ValueName::try_from("textconv").unwrap(),
        BStr::new(format!("\"{}\" clean -t", cur_exe_str.as_str()).as_str()),
    );
    println!("Writing to {}", file_path.display());
    {
        let mut writer = BufWriter::new(fs::File::create(file_path)?);
        file.write_to(&mut writer)?;
    }
    Ok(())
}

pub(super) fn resolve_config_file(
    config_file: Option<&Path>,
    config_type: GitConfigType,
) -> Result<PathBuf, Error> {
    if let Some(config_file) = config_file {
        Ok(config_file.to_path_buf())
    } else {
        let source: Source = config_type.into();
        #[allow(clippy::unwrap_used)]
        let file_path = match config_type {
            GitConfigType::Global | GitConfigType::System => source
                .storage_location(&mut gix_path::env::var)
                .as_deref()
                .unwrap()
                .to_owned(),
            GitConfigType::Local => {
                let dotgit = get_git_repo_and_work_tree()?.0;
                dotgit.join(source.storage_location(&mut gix_path::env::var).unwrap())
            }
        };
        Ok(file_path)
    }
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

fn check_config_sections(
    filter_section: &Result<&gix_config::file::Section, gix_config::lookup::existing::Error>,
    diff_section: &Result<&gix_config::file::Section, gix_config::lookup::existing::Error>,
) -> InstallToolStatus {
    InstallToolStatus {
        diff: diff_section
            .as_ref()
            .is_ok_and(|x| x.contains_value_name("textconv")),
        filter: filter_section
            .as_ref()
            .is_ok_and(|x| x.contains_value_name("clean") && x.contains_value_name("smudge")),
    }
}

pub(super) fn check_install_config_file(config_file: &gix_config::File) -> InstallStatus {
    let filter_section = config_file.section("filter", Some("nbwipers".into()));
    let diff_section = config_file.section("diff", Some("nbwipers".into()));
    let filter_section_nbstripout = config_file.section("filter", Some("nbstripout".into()));
    let diff_section_nbstripout = config_file.section("diff", Some("ipynb".into()));

    InstallStatus {
        nbstripout: check_config_sections(&filter_section_nbstripout, &diff_section_nbstripout),
        nbwipers: check_config_sections(&filter_section, &diff_section),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve() {
        assert!(resolve_config_file(None, GitConfigType::Global).is_ok());
        assert!(resolve_config_file(None, GitConfigType::System).is_ok());
    }
}
