use std::{
    fmt::Debug,
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use crate::{
    cli::RecordCommand,
    files::{find_notebooks, get_cwd, read_nb, relativize_path, FoundNotebooks},
    schema::RawNotebook,
    settings::Settings,
};
use anyhow::Error;
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum RecordError {
    #[error("No .git dir")]
    NoGitDir,
    #[error("Invalid git repo")]
    InvalidGitRepo(#[from] gix_discover::is_git::Error),
    #[error("Not a git worktree")]
    NotAGitWorktree,
    #[error("Failed to create nbwipers dir")]
    FailedCreateNbwipersDir(std::io::Error),
    #[error("Failed to read existing kernelspec file")]
    FailedReadKernelspecFile(std::io::Error),
    #[error("Invalid Kernelspec file")]
    InvalidKernelspecFile(serde_json::Error),
    #[error("Serde Write Error")]
    SerdeWriteError(serde_json::Error),
}

pub fn get_kernelspec_file<P: AsRef<Path>>(path: P) -> Result<PathBuf, RecordError> {
    let git_dir = path.as_ref().join(gix_discover::DOT_GIT_DIR);
    if !git_dir.is_dir() {
        return Err(RecordError::NoGitDir);
    }
    let git_type = gix_discover::is_git(&git_dir)?;
    // I don't know how to test this
    #[cfg(not(tarpaulin_include))]
    if !matches!(git_type, gix_discover::repository::Kind::WorkTree { .. }) {
        return Err(RecordError::NotAGitWorktree);
    }
    let nbwipers_dir = git_dir.join("x-nbwipers");
    fs::create_dir_all(&nbwipers_dir).map_err(RecordError::FailedCreateNbwipersDir)?;
    Ok(nbwipers_dir.join("kernelspec_store.json"))
}
pub fn read_kernelspec_file<P: AsRef<Path>>(
    path: P,
) -> Result<Option<IndexMap<String, KernelSpecInfo>>, RecordError> {
    if path.as_ref().exists() {
        let file = File::open(path).map_err(RecordError::FailedReadKernelspecFile)?;
        let buf = BufReader::new(file);
        Ok(Some(
            serde_json::from_reader(buf).map_err(RecordError::InvalidKernelspecFile)?,
        ))
    } else {
        Ok(None)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct KernelSpecInfo {
    pub kernelspec: Value,
    pub python_version: Option<String>,
}

pub fn record(cmd: RecordCommand) -> Result<(), Error> {
    let path = cmd.path.unwrap_or_else(get_cwd);
    let cli = cmd.common;
    let (args, overrides) = cli.partition();
    let settings = Settings::construct(args.config.as_deref(), args.isolated, &overrides)?;

    let kernelspec_file = get_kernelspec_file(&path)?;
    let mut kernelspec_records = if cmd.sync || cmd.clear {
        IndexMap::new()
    } else {
        read_kernelspec_file(&kernelspec_file)?.unwrap_or_default()
    };
    if !cmd.clear {
        let FoundNotebooks::Files(files) = find_notebooks(&[&path], &settings)? else {
            return Ok(());
        };
        let kernelspecs = get_kernelspecs(&files);
        for (nb, kernel) in kernelspecs {
            kernelspec_records.insert(nb, kernel);
        }
    }
    let out_file = File::create(kernelspec_file)?;
    let mut buf = BufWriter::new(out_file);
    Ok(serde_json::to_writer(&mut buf, &kernelspec_records)
        .map_err(RecordError::SerdeWriteError)?)
}
fn extract_kernel_info(nb: &RawNotebook) -> Option<KernelSpecInfo> {
    let kernelspec = nb.metadata.get("kernelspec");
    let python_version = nb
        .metadata
        .get("language_info")
        .and_then(|li| li.get("version"));
    if kernelspec.is_none() && python_version.is_none() {
        return None;
    }
    Some(KernelSpecInfo {
        kernelspec: kernelspec.cloned().unwrap_or(Value::Null),
        python_version: python_version.and_then(Value::as_str).map(str::to_string),
    })
}

fn get_kernelspecs<P: AsRef<Path> + Sync + Debug>(nbs: &[P]) -> IndexMap<String, KernelSpecInfo> {
    nbs.par_iter()
        .map(|nb| (nb, read_nb(nb)))
        .filter_map(|(path, nb_res)| match nb_res {
            Ok(nb) => Some((path.as_ref(), nb)),
            Err(_) => None,
        })
        .filter_map(|(path, nb_res)| {
            extract_kernel_info(&nb_res).map(|k| (relativize_path(path), k))
        })
        .collect()
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::extract_kernel_info;
    use crate::schema::RawNotebook;

    #[test]
    fn test_blank_not_recorded() {
        let blank_notebook = RawNotebook::new();
        assert!(extract_kernel_info(&blank_notebook).is_none());
    }

    #[test]
    fn test_kernelspec_extracted() {
        let python_version = "3.12.4".to_string();
        let kernelspec = json!({
            "name": "python3",
            "display_name": "Python 3"
        });
        let notebook = RawNotebook {
            cells: vec![],
            metadata: json!(
                {
                    "kernelspec": kernelspec,
                    "language_info": {
                        "name": "python",
                         "version": python_version
                    }
                }
            ),
            nbformat: 4,
            nbformat_minor: 5,
        };
        let extracted = extract_kernel_info(&notebook);
        assert!(extracted.is_some());
        let extracted = extracted.unwrap();
        assert_eq!(extracted.python_version, Some(python_version));
        assert_eq!(extracted.kernelspec, kernelspec);
    }
}
