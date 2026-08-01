use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;

use crate::cli::{CheckLargeFilesCommand, ConfigOverrides, HookCommands};
use crate::files::read_nb;
use crate::settings::Settings;
use crate::strip::{strip_nb, write_nb};
use anyhow::{Context, Error, anyhow, bail};
use itertools::Itertools;

use rayon::prelude::*;
use rustc_hash::FxHashSet;

/// # Errors
///
/// Returns an error if the underlying hook command fails.
pub fn hooks(cmd: &HookCommands) -> Result<(), Error> {
    match cmd {
        HookCommands::CheckLargeFiles(inner_cmd) => check_large_files(inner_cmd),
    }
}
const DEFAULT_MAX_SIZE_KB: u64 = 500; // 500 KB
fn check_normal_filesize<P: AsRef<Path>>(path: P) -> Result<u64, Error> {
    Ok(std::fs::metadata(path)?.len())
}
fn stripped_size(path: &Path, settings: &Settings) -> Result<u64, Error> {
    let file_name = path.file_name().ok_or_else(|| anyhow!("Invalid file"))?;
    if settings.exclude_.is_match(path)
        || settings.exclude_.is_match(file_name)
        || settings.extend_exclude_.is_match(path)
        || settings.extend_exclude_.is_match(file_name)
    {
        return check_normal_filesize(path);
    }
    let nb = read_nb(path)?;
    let (stripped_nb, _) = strip_nb(nb, settings);
    let mut out = Vec::new();
    write_nb(&mut out, &stripped_nb)?;
    Ok(out.len().try_into()?)
}
fn check_large_files(cmd: &CheckLargeFilesCommand) -> Result<(), Error> {
    let max_size_kb = cmd.maxkb.unwrap_or(DEFAULT_MAX_SIZE_KB);
    let mut files: FxHashSet<PathBuf> = cmd.filenames.iter().map(PathBuf::to_owned).collect();
    filter_lfs(&mut files)?;
    if !cmd.enforce_all {
        let added = get_added_files()?;
        files = &files & &added;
    }
    let settings = Settings::construct(
        cmd.config.as_deref(),
        cmd.isolated,
        &ConfigOverrides::default(),
    );

    let out: Vec<(&Path, u64)> = files
        .par_iter()
        .map(|f| {
            match (f.extension().and_then(OsStr::to_str), &settings) {
                (Some("ipynb"), Ok(settings)) => stripped_size(f, settings).map_or_else(
                    |_| {
                        eprintln!(
                            "Could not parse nb file {}. Using on-disk size",
                            f.to_string_lossy()
                        );
                        Ok((f.as_path(), check_normal_filesize(f)?))
                    },
                    |size| Ok((f.as_path(), size)),
                ),
                (Some("ipynb"), Err(_)) => {
                    eprintln!("Could not parse settings. Using on-disk size");
                    Ok((f.as_path(), check_normal_filesize(f)?))
                }
                _ => Ok((f.as_path(), check_normal_filesize(f)?)),
            }
            // don't worry about
        })
        .map(|x| x)
        .collect::<Result<Vec<(&Path, u64)>, Error>>()?;
    let mut status = false;
    for (file, size) in out {
        let size_kb = size.div_ceil(1024);
        if size_kb > max_size_kb {
            println!(
                "{} ({} KB) exceeds {} KB",
                file.to_string_lossy(),
                size_kb,
                max_size_kb
            );
            status = true;
        }
    }
    if status {
        bail!("Some files exceed the limit")
    }
    Ok(())
}

fn get_added_files() -> Result<FxHashSet<PathBuf>, Error> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "--staged", "--diff-filter=A"])
        .output()?;
    if !output.status.success() {
        bail!("Git diff failed");
    }
    let stdout = String::from_utf8(output.stdout)?;
    stdout
        .lines()
        .map(PathBuf::from_str)
        .collect::<Result<FxHashSet<PathBuf>, _>>()
        .map_err(Error::from)
}

fn filter_lfs(files: &mut FxHashSet<PathBuf>) -> Result<(), Error> {
    let file_list = Itertools::intersperse(
        files.iter().filter_map(|p| p.to_str().map(str::as_bytes)),
        b"\0",
    )
    .collect::<Vec<&[u8]>>()
    .concat();

    let mut check_attr = Command::new("git")
        .args(["check-attr", "filter", "-z", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    {
        let mut stdin = check_attr.stdin.take().context("Could not open stdin")?;
        // git may exit without reading stdin (e.g. outside a repository);
        // ignore the broken pipe so the exit-status check reports the real error
        match stdin.write_all(&file_list) {
            Err(e) if e.kind() != std::io::ErrorKind::BrokenPipe => return Err(e.into()),
            _ => {}
        }
    }
    let check_output = check_attr.wait_with_output()?;
    if !check_output.status.success() {
        bail!("Git check-attr failed");
    }
    let stdout = String::from_utf8(check_output.stdout)?;

    let parts = stdout.trim_matches('\0').split('\0');

    for chunk in &parts.chunks(3) {
        let mut chunk = chunk;
        // 1st element index 0
        let fname = chunk.next();
        // 3rd element index 2, but we already consumed one...
        let info = chunk.nth(1);
        if let (Some(fname), Some(info)) = (fname.map(PathBuf::from), info)
            && info == "lfs"
        {
            files.remove(&fname);
        }
    }

    Ok(())
}
