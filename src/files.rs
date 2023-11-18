use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Error};
use ignore::WalkBuilder;
use itertools::Itertools;
use path_absolutize::Absolutize;
/// Convert any path to an absolute path (based on the current working
/// directory).
fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if let Ok(path) = path.absolutize() {
        return path.to_path_buf();
    }
    path.to_path_buf()
}

pub fn find_notebooks(paths: &[PathBuf]) -> Result<Vec<PathBuf>, Error> {
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

    Ok(files.into_inner()?)
}
