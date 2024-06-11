mod attributes;
mod gitconfig;
use anyhow::{bail, Error};

use std::{
    env,
    ops::{BitAnd, BitOrAssign},
    path::PathBuf,
};

use gix_config::Source;

use crate::cli::GitConfigType;
use attributes::check_install_attr_files;
pub use attributes::{install_attributes, uninstall_attributes};
use gitconfig::{check_install_config_file, resolve_config_file};
pub use gitconfig::{install_config, uninstall_config};
use gix_config::File;

impl From<GitConfigType> for Source {
    fn from(value: GitConfigType) -> Self {
        match value {
            GitConfigType::Global => Self::User,
            GitConfigType::System => Self::System,
            GitConfigType::Local => Self::Local,
        }
    }
}

fn get_git_repo_and_work_tree() -> Result<(PathBuf, Option<PathBuf>), Error> {
    let cur_dir = std::env::current_dir()?;
    let (git_dir, _) = gix_discover::upwards(&cur_dir)?;
    Ok(git_dir.into_repository_and_work_tree_directories())
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct InstallToolStatus {
    pub diff: bool,
    pub filter: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct InstallStatus {
    pub nbstripout: InstallToolStatus,
    pub nbwipers: InstallToolStatus,
}

impl InstallToolStatus {
    const fn is_installed(&self) -> bool {
        self.diff && self.filter
    }
}
// impl From<InstallToolStatus> for bool {
//     fn from(value: InstallToolStatus) -> Self {
//         value.diff & value.filter
//     }
// }
// impl From<InstallStatus> for bool {
//     fn from(value: InstallStatus) -> Self {
//         (value.nbstripout | value.nbwipers).into()
//     }
// }

impl BitAnd for InstallToolStatus {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            diff: self.diff & rhs.diff,
            filter: self.filter & rhs.filter,
        }
    }
}

// impl BitOr for InstallToolStatus {
//     type Output = InstallToolStatus;
//     fn bitor(self, rhs: Self) -> Self::Output {
//         InstallToolStatus {
//             diff: self.diff | rhs.diff,
//             filter: self.filter | rhs.filter,
//         }
//     }
// }

// impl BitOr for InstallStatus {
//     type Output = InstallStatus;
//     fn bitor(self, rhs: Self) -> Self::Output {
//         InstallStatus {
//             nbstripout: self.nbstripout | rhs.nbstripout,
//             nbwipers: self.nbwipers | rhs.nbwipers,
//         }
//     }
// }
impl BitAnd for InstallStatus {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            nbstripout: self.nbstripout & rhs.nbstripout,
            nbwipers: self.nbwipers & rhs.nbwipers,
        }
    }
}

impl BitOrAssign for InstallToolStatus {
    fn bitor_assign(&mut self, rhs: Self) {
        self.diff |= rhs.diff;
        self.filter |= rhs.filter;
    }
}
impl BitOrAssign for InstallStatus {
    fn bitor_assign(&mut self, rhs: Self) {
        self.nbstripout |= rhs.nbstripout;
        self.nbwipers |= rhs.nbwipers;
    }
}

fn combine_install_status(
    attr_install_status: InstallStatus,
    config_install_status: InstallStatus,
) -> Result<(), Error> {
    let overall_status = attr_install_status & config_install_status;
    let mut installed = false;
    if overall_status.nbstripout.is_installed() {
        installed = true;
        println!("nbstripout is installed");
    }
    if overall_status.nbwipers.is_installed() {
        installed = true;
        println!("nbwipers is installed");
    }
    if installed {
        Ok(())
    } else {
        bail!("Neither nbstripout nor nbwipers are installed.")
    }
}
pub fn check_install_some_type(config_type: GitConfigType) -> Result<(), Error> {
    let attr_install_status = check_install_attr_files(&[config_type])?;

    let file_path = resolve_config_file(None, config_type)?;
    let config_file = File::from_path_no_includes(file_path, config_type.into())?;
    let config_install_status = check_install_config_file(&config_file);

    combine_install_status(attr_install_status, config_install_status)
}
pub fn check_install_none_type() -> Result<(), Error> {
    let config_types = vec![
        GitConfigType::Local,
        GitConfigType::Global,
        GitConfigType::System,
    ];

    let attr_install_status = check_install_attr_files(&config_types)?;
    let config_file = File::from_git_dir(get_git_repo_and_work_tree()?.0)?;
    let config_install_status = check_install_config_file(&config_file);

    combine_install_status(attr_install_status, config_install_status)
}

pub fn check_should_exit_zero(exit_zero: bool) -> bool {
    if exit_zero {
        exit_zero
    } else {
        env::var("NBWIPERS_CHECK_INSTALL_EXIT_ZERO").is_ok()
    }
}

#[cfg(test)]
mod test {
    use crate::test_helpers::with_dir;

    use super::*;
    use std::{fs::create_dir_all, process::Command};

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_git_discovery() {
        let temp_dir = tempfile::tempdir().unwrap();

        let git_init_out = Command::new("git")
            .current_dir(&temp_dir)
            .args(["init"])
            .output()
            .expect("git init failed");
        assert!(git_init_out.status.success());
        let subdir = temp_dir.path().join("subdir/");
        create_dir_all(&subdir).unwrap();
        with_dir(&subdir, || {
            let res = get_git_repo_and_work_tree();
            assert_eq!(res.unwrap().1.unwrap(), temp_dir.path());
        });
    }
}
