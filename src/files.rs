use std::{
    ffi::OsStr,
    fs::File,
    io::{stdin, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Error};
use ignore::WalkBuilder;
use itertools::Itertools;
use path_absolutize::Absolutize;
use thiserror::Error;

use crate::schema::RawNotebook;

// normalize_path and relative_path are from Ruff, used under the MIT license

fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if let Ok(path) = path.absolutize() {
        return path.to_path_buf();
    }
    path.to_path_buf()
}
pub fn relativize_path<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();

    let cwd = path_absolutize::path_dedot::CWD.as_path();

    if let Ok(path) = path.strip_prefix(cwd) {
        return format!("{}", path.display());
    }
    format!("{}", path.display())
}

pub enum FoundNotebooks {
    Stdin,
    Files(Vec<PathBuf>),
}

pub fn find_notebooks(paths: &[PathBuf]) -> Result<FoundNotebooks, Error> {
    if paths == [Path::new("-")] {
        return Ok(FoundNotebooks::Stdin);
    }
    let paths: Vec<PathBuf> = paths.iter().map(normalize_path).unique().collect();
    let (first_path, rest_paths) = paths
        .split_first()
        .ok_or(anyhow!("Please provide at least one path"))?;

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
            if let Ok(entry) = path {
                let resolved = if entry.file_type().map_or(true, |ft| ft.is_dir()) {
                    None
                } else if entry.depth() == 0 {
                    Some(entry.into_path())
                } else {
                    let cur_path = entry.into_path();
                    if cur_path.extension() == Some(OsStr::new("ipynb")) {
                        Some(cur_path)
                    } else {
                        None
                    }
                };
                if let Some(resolved) = resolved {
                    #[allow(clippy::unwrap_used)]
                    files.lock().unwrap().push(resolved);
                }
            }

            ignore::WalkState::Continue
        })
    });
    let out = files.into_inner()?;
    if out.is_empty() {
        Err(anyhow!("Could not find any notebooks in path(s)"))
    } else {
        Ok(FoundNotebooks::Files(out))
    }
}

pub fn read_nb(path: &Path) -> Result<RawNotebook, NBReadError> {
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
    #[error("JSON read error")]
    Serde(#[from] serde_json::Error),
}

pub fn read_nb_stdin() -> Result<RawNotebook, NBReadError> {
    let handle = stdin().lock();
    let out = serde_json::from_reader(handle)?;
    Ok(out)
}
