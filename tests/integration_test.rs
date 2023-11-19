use std::{fs, path::PathBuf, process::Command};

use bstr::ByteSlice;

#[test]
fn test_invalid_format() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_wipers"));

    let output = Command::new(cur_exe)
        .args(["check", "tests/test_nbformat2.ipynb"])
        .output()
        .expect("command failed");
    assert!(&output.stdout.contains_str(b"Invalid notebook:"))
}

fn test_expected(path: &str, expected: &str, extra_args: &[&str]) {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_wipers"));
    let output = Command::new(cur_exe)
        .args(["clean", "-t", path])
        .args(extra_args)
        .output()
        .expect("command failed");

    let expected_content = fs::read(expected).expect("could not read expected");
    assert_eq!(output.stdout, expected_content);
}

#[test]
fn test_drop_empty_cells_dontdrop() {
    test_expected(
        "tests/e2e_notebooks/test_drop_empty_cells.ipynb",
        "tests/e2e_notebooks/test_drop_empty_cells_dontdrop.ipynb.expected",
        &[],
    );
}

#[test]
fn test_drop_empty_cells() {
    test_expected(
        "tests/e2e_notebooks/test_drop_empty_cells.ipynb",
        "tests/e2e_notebooks/test_drop_empty_cells.ipynb.expected",
        &["--drop-empty-cells"],
    );
}
