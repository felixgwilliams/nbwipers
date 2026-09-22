mod attributes;
mod gitconfig;
use anyhow::{Error, bail};

use std::{
    borrow::Cow,
    env,
    ops::{BitAnd, BitOrAssign},
    path::{Path, PathBuf},
};

use gix_config::{
    Source,
    file::{includes, init},
    path::interpolate,
};

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
/// Repository config as git sees it from the current worktree:
/// globals, `<common_dir>/config`, `<git_dir>/config.worktree`, env overrides.
fn repo_config(common_dir: &Path, git_dir: &Path) -> Result<File<'static>, Error> {
    let load = |path: PathBuf, source| -> Result<Option<File<'static>>, Error> {
        if path.is_file() {
            Ok(Some(File::from_path_no_includes(path, source)?))
        } else {
            Ok(None)
        }
    };
    let local = load(common_dir.join("config"), Source::Local)?;
    let worktree_enabled = local
        .as_ref()
        .and_then(|f| f.boolean("extensions.worktreeConfig"))
        .transpose()?
        .unwrap_or(false);
    let worktree = if worktree_enabled {
        load(git_dir.join("config.worktree"), Source::Worktree)?
    } else {
        None
    };

    // same include options as `File::from_git_dir`; `includeIf "gitdir:..."`
    // is matched against the per-worktree git dir, as git does
    let home = gix_path::env::home_dir();
    let options = init::Options {
        includes: includes::Options::follow(
            interpolate::Context {
                home_dir: home.as_deref(),
                ..Default::default()
            },
            includes::conditional::Context {
                git_dir: Some(git_dir),
                branch_name: None,
            },
        ),
        ..Default::default()
    };

    let mut config = File::from_globals()?;
    config.resolve_includes(options)?;
    for mut file in [local, worktree].into_iter().flatten() {
        file.resolve_includes(options)?;
        config.append(file);
    }
    config.append(File::from_environment_overrides()?);
    Ok(config)
}
pub(crate) struct GitDirs {
    pub common: PathBuf,
    pub git: PathBuf,
    pub work: Option<PathBuf>,
}
pub(crate) fn get_git_dirs(path: Option<&Path>) -> Result<GitDirs, Error> {
    let cur_dir = match path {
        Some(p) => Cow::Borrowed(p),
        None => Cow::Owned(std::env::current_dir()?),
    };
    let (git_dir, work_dir) = gix_discover::upwards(&cur_dir)?
        .0
        .into_repository_and_work_tree_directories();
    match gix_discover::path::from_plain_file(&git_dir.join("commondir")) {
        Some(common) => {
            let common = common?;
            Ok(GitDirs {
                common: git_dir.join(&common),
                git: git_dir,
                work: work_dir,
            })
        }
        None => Ok(GitDirs {
            common: git_dir.clone(),
            git: git_dir,
            work: work_dir,
        }),
    }
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
/// # Errors
///
/// Returns an error if the attribute or config files cannot be checked, or if neither
/// `nbstripout` nor `nbwipers` is installed.
pub fn check_install_some_type(config_type: GitConfigType) -> Result<(), Error> {
    let attr_install_status = check_install_attr_files(&[config_type])?;

    let file_path = resolve_config_file(None, config_type)?;
    let config_file = File::from_path_no_includes(file_path, config_type.into())?;
    let config_install_status = check_install_config_file(&config_file);

    combine_install_status(attr_install_status, config_install_status)
}
/// # Errors
///
/// Returns an error if the attribute or config files cannot be checked, or if neither
/// `nbstripout` nor `nbwipers` is installed.
pub fn check_install_none_type() -> Result<(), Error> {
    let git_dirs = get_git_dirs(None)?;
    let config_file = repo_config(&git_dirs.common, &git_dirs.git)?;
    let config_types = vec![
        GitConfigType::Local,
        GitConfigType::Global,
        GitConfigType::System,
    ];

    let attr_install_status = check_install_attr_files(&config_types)?;
    let config_install_status = check_install_config_file(&config_file);

    combine_install_status(attr_install_status, config_install_status)
}

#[must_use]
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
            let res = get_git_dirs(None);
            // canonicalize both sides: on macOS `TMPDIR` lives under `/var/...`,
            // a symlink to `/private/var/...`, and `current_dir()` (used inside
            // `get_git_repo_and_work_tree`) returns the resolved form.
            assert_eq!(
                res.unwrap().work.unwrap().canonicalize().unwrap(),
                temp_dir.path().canonicalize().unwrap()
            );
        });
    }
}
