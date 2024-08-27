use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::RwLock;

use crate::cli::{CheckLargeFilesCommand, ConfigOverrides, HookCommands};
use crate::files::read_nb;
use crate::settings::Settings;
use crate::strip::{strip_nb, write_nb};
use anyhow::{anyhow, bail, Context, Error};
use itertools::Itertools;

use rayon::prelude::*;
use rustc_hash::FxHashSet;

pub fn hooks(cmd: &HookCommands) -> Result<(), Error> {
    match cmd {
        HookCommands::CheckLargeFiles(ref inner_cmd) => check_large_files(inner_cmd),
    }
}
const DEFAULT_MAX_SIZE_KB: u64 = 500; // 500 KB
fn check_normal_filesize<P: AsRef<Path>>(path: P) -> Result<u64, Error> {
    Ok(std::fs::metadata(path)?.len())
}
fn check_large_files(cmd: &CheckLargeFilesCommand) -> Result<(), Error> {
    let max_size_kb = cmd.maxkb.unwrap_or(DEFAULT_MAX_SIZE_KB);
    let mut files: FxHashSet<PathBuf> = cmd.filenames.iter().map(PathBuf::to_owned).collect();
    filter_lfs(&mut files)?;
    if !cmd.enforce_all {
        let added = get_added_files()?;
        files = &files & &added;
    }
    let lazy_settings = SizeFinder::new();

    let out: Vec<(&Path, u64)> = files
        .par_iter()
        .map(|f| {
            match f.extension().and_then(OsStr::to_str) {
                Some("ipynb") => lazy_settings
                    .stripped_size(f, cmd.config.as_deref())
                    .map_or_else(
                        |_| {
                            eprintln!(
                                "Could not parse nb file {}. Using on-disk size",
                                f.to_string_lossy()
                            );
                            Ok((f.as_path(), check_normal_filesize(f)?))
                        },
                        |size| Ok((f.as_path(), size)),
                    ),
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

#[derive(Debug)]
struct SizeFinder {
    settings: RwLock<Option<Settings>>,
}
impl SizeFinder {
    const fn new() -> Self {
        Self {
            settings: RwLock::new(None),
        }
    }
    #[allow(clippy::unwrap_used)]
    fn load_settings(&self, config_file: Option<&Path>) -> Result<(), Error> {
        if self.settings.read().unwrap().is_none() {
            let mut s = self.settings.write().unwrap();
            *s = Some(Settings::construct(
                config_file,
                &ConfigOverrides::default(),
            )?);
        }
        Ok(())
    }
    #[allow(clippy::unwrap_used)]
    fn stripped_size(&self, path: &Path, config_file: Option<&Path>) -> Result<u64, Error> {
        self.load_settings(config_file)?;
        let binding = self.settings.read().unwrap();

        let x = binding.as_ref().expect("settings should be loaded");
        let file_name = path.file_name().ok_or_else(|| anyhow!("Invalid file"))?;

        if x.exclude_.is_match(path)
            || x.exclude_.is_match(file_name)
            || x.extend_exclude_.is_match(path)
            || x.extend_exclude_.is_match(file_name)
        {
            // we're not treating this one as a candidate for stripping
            return check_normal_filesize(path);
        }

        let nb = read_nb(path)?;

        let (stripped_nb, _) = strip_nb(nb, binding.as_ref().expect("settings should be loaded"));
        drop(binding); // release lock early at clippy's suggestion
        let mut out: Vec<u8> = Vec::new();

        write_nb(&mut out, &stripped_nb)?;
        Ok(out.len() as u64)
    }
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
        stdin.write_all(&file_list)?;
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
        if let (Some(fname), Some(info)) = (fname.map(PathBuf::from), info) {
            if info == "lfs" {
                files.remove(&fname);
            }
        }
    }

    Ok(())
}
