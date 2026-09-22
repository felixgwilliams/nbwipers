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

/// Moves the nbwipers filter/diff config out of the shared `.git/config`,
/// writing it with `git config <location...>` run in `dir` instead.
fn move_config(repo: &Path, dir: &Path, location: &[&str]) {
    for key in [
        "filter.nbwipers.clean",
        "filter.nbwipers.smudge",
        "diff.nbwipers.textconv",
    ] {
        let output = git(repo, &["config", "--get", key]);
        assert!(output.status.success());
        let value = String::from_utf8(output.stdout).unwrap();
        let args = [&["config"], location, &[key, value.trim_end()]].concat();
        assert!(git(dir, &args).status.success());
        assert!(git(repo, &["config", "--unset", key]).status.success());
    }
}

/// Enables `extensions.worktreeConfig` and moves the nbwipers config into the
/// `config.worktree` of the worktree at `dir`, so only that worktree sees it.
fn move_config_to_worktree_config(repo: &Path, dir: &Path) {
    assert!(
        git(repo, &["config", "extensions.worktreeConfig", "true"])
            .status
            .success()
    );
    move_config(repo, dir, &["--worktree"]);
}

#[test]
fn test_check_install_linked_worktree_config() {
    // with extensions.worktreeConfig, config installed in the linked
    // worktree's `.git/worktrees/wt/config.worktree` applies only there.
    let (_temp_dir, repo, wt) = repo_with_worktree();
    assert!(nbwipers(&repo, &["install", "local"]).status.success());
    move_config_to_worktree_config(&repo, &wt);

    // sanity check: git agrees on who can see the filter
    let get_filter = ["config", "--get", "filter.nbwipers.clean"];
    assert!(git(&wt, &get_filter).status.success());
    assert!(!git(&repo, &get_filter).status.success());

    let output = nbwipers(&wt, &["check-install"]);
    dbg!(String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
    assert!(!nbwipers(&repo, &["check-install"]).status.success());
}

#[test]
fn test_check_install_main_worktree_config() {
    // the reverse: config in the main worktree's `.git/config.worktree`
    // must not be treated as installed in a linked worktree.
    let (_temp_dir, repo, wt) = repo_with_worktree();
    assert!(nbwipers(&repo, &["install", "local"]).status.success());
    move_config_to_worktree_config(&repo, &repo);

    let get_filter = ["config", "--get", "filter.nbwipers.clean"];
    assert!(git(&repo, &get_filter).status.success());
    assert!(!git(&wt, &get_filter).status.success());

    assert!(nbwipers(&repo, &["check-install"]).status.success());
    let output = nbwipers(&wt, &["check-install"]);
    dbg!(String::from_utf8_lossy(&output.stderr));
    assert!(!output.status.success());
}

#[test]
fn test_check_install_config_include() {
    // config reached through `[include]` in the shared `.git/config`
    let (temp_dir, repo, wt) = repo_with_worktree();
    assert!(nbwipers(&repo, &["install", "local"]).status.success());
    let included = temp_dir.path().join("nbwipers.gitconfig");
    let included = included.to_str().unwrap();
    move_config(&repo, &repo, &["-f", included]);
    assert!(
        git(&repo, &["config", "include.path", included])
            .status
            .success()
    );

    let get_filter = ["config", "--get", "filter.nbwipers.clean"];
    assert!(git(&repo, &get_filter).status.success());
    assert!(git(&wt, &get_filter).status.success());

    assert!(nbwipers(&repo, &["check-install"]).status.success());
    let output = nbwipers(&wt, &["check-install"]);
    dbg!(String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}

#[test]
fn test_check_install_config_include_if_gitdir() {
    // `includeIf "gitdir:..."` matches the per-worktree git dir, so a condition
    // on `.git/worktrees/wt` applies only in the linked worktree.
    let (temp_dir, repo, wt) = repo_with_worktree();
    assert!(nbwipers(&repo, &["install", "local"]).status.success());
    let included = temp_dir.path().join("nbwipers.gitconfig");
    let included = included.to_str().unwrap();
    move_config(&repo, &repo, &["-f", included]);
    let wt_git_dir = repo.join(".git/worktrees/wt");
    let key = format!("includeIf.gitdir:{}.path", wt_git_dir.to_str().unwrap());
    assert!(git(&repo, &["config", &key, included]).status.success());

    let get_filter = ["config", "--get", "filter.nbwipers.clean"];
    assert!(!git(&repo, &get_filter).status.success());
    assert!(git(&wt, &get_filter).status.success());

    assert!(!nbwipers(&repo, &["check-install"]).status.success());
    let output = nbwipers(&wt, &["check-install"]);
    dbg!(String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}

#[test]
fn test_check_install_worktree_config_without_extension() {
    // without extensions.worktreeConfig, git ignores `config.worktree`, so
    // config found there must not count as installed.
    let (_temp_dir, repo, wt) = repo_with_worktree();
    assert!(nbwipers(&repo, &["install", "local"]).status.success());
    let wt_config = repo.join(".git/worktrees/wt/config.worktree");
    move_config(&repo, &repo, &["-f", wt_config.to_str().unwrap()]);
    assert!(wt_config.is_file());

    let get_filter = ["config", "--get", "filter.nbwipers.clean"];
    assert!(!git(&wt, &get_filter).status.success());

    let output = nbwipers(&wt, &["check-install"]);
    dbg!(String::from_utf8_lossy(&output.stdout));
    assert!(!output.status.success());
}
