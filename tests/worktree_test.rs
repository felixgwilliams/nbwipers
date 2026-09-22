#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]
//! Tests for running nbwipers inside a linked git worktree (`git worktree add`)
//! and from subdirectories of a repository.
//!
//! In a linked worktree, `<worktree>/.git` is a file pointing at
//! `<repo>/.git/worktrees/<name>`, which has no `config` or `info/attributes`;
//! git finds those through `<repo>/.git/worktrees/<name>/commondir`.
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use nbwipers::{schema::RawNotebook, strip::write_nb};
use serde_json::json;
use tempfile::TempDir;

fn git<P: AsRef<Path>>(dir: P, args: &[&str]) -> Output {
    Command::new("git")
        .current_dir(dir)
        .args(["-c", "user.name=test", "-c", "user.email=test@example.com"])
        .args(args)
        .output()
        .expect("git failed")
}

fn nbwipers<P: AsRef<Path>>(dir: P, args: &[&str]) -> Output {
    Command::new(PathBuf::from(env!("CARGO_BIN_EXE_nbwipers")))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("command failed")
}

/// Creates `<temp>/repo` with one commit and a linked worktree at `<temp>/wt`.
fn repo_with_worktree() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = temp_dir.path().join("repo");
    let wt = temp_dir.path().join("wt");
    fs::create_dir(&repo).unwrap();
    assert!(git(&repo, &["init"]).status.success());
    assert!(
        git(&repo, &["commit", "--allow-empty", "-m", "init"])
            .status
            .success()
    );
    assert!(
        git(
            &repo,
            &["worktree", "add", wt.to_str().unwrap(), "-b", "wt"]
        )
        .status
        .success()
    );
    (temp_dir, repo, wt)
}

fn write_dirty_nb(path: &Path) {
    let nb = RawNotebook {
        metadata: json!({
            "kernelspec": {"name": "python3", "display_name": "Python 3"},
            "language_info": {"name": "python", "version": "3.12.4"}
        }),
        ..Default::default()
    };
    write_nb(File::create(path).unwrap(), &nb).unwrap();
}

#[test]
fn test_check_install_in_linked_worktree() {
    // issue #25: installed in the main checkout, but check-install from a
    // linked worktree reads `.git/worktrees/wt/config`, which doesn't exist.
    let (_temp_dir, repo, wt) = repo_with_worktree();
    assert!(nbwipers(&repo, &["install", "local"]).status.success());
    assert!(nbwipers(&repo, &["check-install"]).status.success());

    let output = nbwipers(&wt, &["check-install"]);
    dbg!(String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());

    let output = nbwipers(&wt, &["check-install", "local"]);
    dbg!(String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}

#[test]
fn test_install_local_from_linked_worktree_is_seen_by_git() {
    // install local from a linked worktree must write where git reads it,
    // not into the per-worktree gitdir.
    let (_temp_dir, repo, wt) = repo_with_worktree();
    assert!(nbwipers(&wt, &["install", "local"]).status.success());

    let output = git(&wt, &["config", "--get", "filter.nbwipers.clean"]);
    assert!(output.status.success(), "filter not visible to git");

    let output = git(&wt, &["check-attr", "filter", "--", "notebook.ipynb"]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("filter: nbwipers"), "{stdout}");

    // and the main checkout should see it too, since config is shared
    assert!(nbwipers(&repo, &["check-install"]).status.success());
}

#[test]
fn test_uninstall_local_from_linked_worktree() {
    let (_temp_dir, repo, wt) = repo_with_worktree();
    assert!(nbwipers(&repo, &["install", "local"]).status.success());
    assert!(nbwipers(&wt, &["uninstall", "local"]).status.success());

    let output = git(&repo, &["config", "--get", "filter.nbwipers.clean"]);
    assert!(!output.status.success(), "filter still installed");
    assert!(!nbwipers(&repo, &["check-install"]).status.success());
}

#[test]
fn test_record_and_smudge_in_linked_worktree() {
    let (_temp_dir, _repo, wt) = repo_with_worktree();
    write_dirty_nb(&wt.join("notebook.ipynb"));

    let output = nbwipers(&wt, &["record"]);
    dbg!(String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());

    let nb_bytes = fs::read(wt.join("notebook.ipynb")).unwrap();
    let mut child = Command::new(PathBuf::from(env!("CARGO_BIN_EXE_nbwipers")))
        .current_dir(&wt)
        .args(["smudge", "notebook.ipynb"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("smudge failed");
    child.stdin.take().unwrap().write_all(&nb_bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    dbg!(String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}

#[test]
fn test_record_from_subdir() {
    // not worktree specific: record only looks for `<path>/.git`, without
    // searching parent directories.
    let temp_dir = tempfile::tempdir().unwrap();
    assert!(git(&temp_dir, &["init"]).status.success());
    let sub = temp_dir.path().join("sub");
    fs::create_dir(&sub).unwrap();
    write_dirty_nb(&sub.join("notebook.ipynb"));

    let output = nbwipers(&sub, &["record"]);
    dbg!(String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());

    let output = nbwipers(&temp_dir, &["record", "sub"]);
    dbg!(String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}
