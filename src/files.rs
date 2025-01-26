use std::{
    ffi::OsStr,
    fs::File,
    io::{stdin, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Error};
use globset::Candidate;
use ignore::{WalkBuilder, WalkState};
use itertools::Itertools;
use path_absolutize::Absolutize;
use thiserror::Error;

use crate::{schema::RawNotebook, settings::Settings};

#[inline]
#[cfg(not(test))]
pub fn get_cwd() -> PathBuf {
    path_absolutize::path_dedot::CWD.to_owned()
    // current_dir().unwrap().absolutize().unwrap().into_owned()
}
#[allow(clippy::unwrap_used)]
#[cfg(test)]
pub fn get_cwd() -> PathBuf {
    use std::env::current_dir;

    // path_absolutize::path_dedot::CWD.to_owned()
    current_dir().unwrap().absolutize().unwrap().into_owned()
}

// normalize_path, normalize_path_to relative_path are from Ruff, used under the MIT license

pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    path.absolutize()
        .map_or_else(|_| path.to_path_buf(), |path| path.to_path_buf())
}
pub fn relativize_path<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();

    let cwd = get_cwd();

    path.strip_prefix(cwd).map_or_else(
        |_| format!("{}", path.display()),
        |path| format!("{}", path.display()),
    )
}

pub fn normalize_path_to<P: AsRef<Path>, R: AsRef<Path>>(path: P, root_path: R) -> PathBuf {
    let path = path.as_ref();
    if let Ok(path) = path.absolutize_from(root_path.as_ref()) {
        return path.to_path_buf();
    }
    path.to_path_buf()
}

pub enum FoundNotebooks {
    Stdin,
    NoFiles,
    Files(Vec<PathBuf>),
}

pub fn find_notebooks_or_stdin(
    paths: &[PathBuf],
    settings: &Settings,
) -> Result<FoundNotebooks, Error> {
    if paths == [Path::new("-")] {
        return Ok(FoundNotebooks::Stdin);
    }
    find_notebooks(paths, settings)
}
pub fn check_exclusions(path: &Path, settings: &Settings) -> bool {
    if let Some(file_name) = path.file_name() {
        let fname_candidate = Candidate::new(file_name);
        let path_candidate = Candidate::new(path);
        if !settings.exclude_.is_empty()
            && (settings.exclude_.is_match_candidate(&fname_candidate)
                || settings.exclude_.is_match_candidate(&path_candidate))
        {
            return true;
        }
        if !settings.extend_exclude_.is_empty()
            && (settings
                .extend_exclude_
                .is_match_candidate(&fname_candidate)
                || settings.extend_exclude_.is_match_candidate(&path_candidate))
        {
            return true;
        }
    }
    false
}

pub fn find_notebooks<P: AsRef<Path>>(
    paths: &[P],
    settings: &Settings,
) -> Result<FoundNotebooks, Error> {
    let paths: Vec<PathBuf> = paths.iter().map(normalize_path).unique().collect();
    let (first_path, rest_paths) = paths
        .split_first()
        .ok_or_else(|| anyhow!("Please provide at least one path"))?;

    let mut builder = WalkBuilder::new(first_path);
    for path in rest_paths {
        builder.add(path);
    }
    builder.standard_filters(true);
    builder.hidden(false);

    let walker = builder.build_parallel();
    let files: std::sync::Mutex<Vec<PathBuf>> = std::sync::Mutex::new(vec![]);
    walker.run(|| {
        Box::new(|path| {
            if let Ok(entry) = &path {
                if entry.depth() > 0 {
                    let path = entry.path();
                    if check_exclusions(path, settings) {
                        return WalkState::Skip;
                    }
                }

                let resolved = if entry.file_type().map_or(true, |ft| ft.is_dir()) {
                    None
                } else if entry.depth() == 0 {
                    Some(entry.path())
                } else {
                    let cur_path = entry.path();
                    if cur_path.extension() == Some(OsStr::new("ipynb")) {
                        Some(cur_path)
                    } else {
                        None
                    }
                };
                if let Some(resolved) = resolved {
                    #[allow(clippy::unwrap_used)]
                    files.lock().unwrap().push(resolved.to_owned());
                }
            }

            ignore::WalkState::Continue
        })
    });
    let out = files.into_inner()?;
    if out.is_empty() {
        Ok(FoundNotebooks::NoFiles)
    } else {
        Ok(FoundNotebooks::Files(out))
    }
}

pub fn read_nb<P: AsRef<Path>>(path: P) -> Result<RawNotebook, NBReadError> {
    let f = File::open(path)?;
    let rdr = BufReader::new(f);

    let out = serde_json::from_reader(rdr)?;
    Ok(out)
}

#[derive(Error, Debug)]
pub enum NBReadError {
    #[error("File IO error")]
    IO(#[from] std::io::Error),
    #[error("JSON read error")]
    Serde(#[from] serde_json::Error),
}
#[derive(Debug, Error)]
pub enum NBWriteError {
    #[error("File IO error")]
    IO(#[from] std::io::Error),
    #[error("JSON write error")]
    Serde(#[from] serde_json::Error),
}

pub fn read_nb_stdin() -> Result<RawNotebook, NBReadError> {
    let out = serde_json::from_reader(stdin().lock())?;
    Ok(out)
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {

    use crate::test_helpers::CWD_MUTEX;

    use super::get_cwd;
    use super::normalize_path;
    use super::relativize_path;

    #[test]
    fn test_normalize() {
        let _lock = CWD_MUTEX.lock();
        let cur_dir = get_cwd();
        dbg!(&cur_dir);
        let rel_dir = "subdir";
        let subdir = cur_dir.join(rel_dir);
        assert_eq!(normalize_path(format!("./{rel_dir}")), subdir);
        dbg!(&subdir);
        dbg!(relativize_path(&subdir));
        assert_eq!(rel_dir, relativize_path(&subdir));
        if let Some(parent) = cur_dir.parent() {
            assert_eq!(parent.to_str().unwrap(), relativize_path(parent));
        }
    }
}
