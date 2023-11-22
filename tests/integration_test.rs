use std::{fs, path::PathBuf, process::Command};

use bstr::ByteSlice;

#[test]
fn test_invalid_format() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let output = Command::new(cur_exe)
        .args(["check", "tests/test_nbformat2.ipynb"])
        .output()
        .expect("command failed");
    assert!(&output.stdout.contains_str(b"Invalid notebook:"))
}

fn test_expected(path: &str, expected: &str, extra_args: &[&str]) {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let output = Command::new(cur_exe)
        .args(["clean", "-t", path])
        .args(extra_args)
        .output()
        .expect("command failed");

    let expected_content = fs::read_to_string(expected).expect("could not read expected");
    assert_eq!(output.stdout.to_str().unwrap(), expected_content);
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

#[test]
fn test_drop_tagged_cells_dontdrop() {
    test_expected(
        "tests/e2e_notebooks/test_drop_tagged_cells.ipynb",
        "tests/e2e_notebooks/test_drop_tagged_cells_dontdrop.ipynb.expected",
        &[],
    );
}

#[test]
fn test_drop_tagged_cells() {
    test_expected(
        "tests/e2e_notebooks/test_drop_tagged_cells.ipynb",
        "tests/e2e_notebooks/test_drop_tagged_cells.ipynb.expected",
        &["--drop-tagged-cells=test"],
    );
}
#[test]
fn test_execution_timing() {
    test_expected(
        "tests/e2e_notebooks/test_execution_timing.ipynb",
        "tests/e2e_notebooks/test_execution_timing.ipynb.expected",
        &["--drop-tagged-cells=test"],
    );
}
#[test]
fn test_metadata() {
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata.ipynb.expected",
        &[],
    );
}
#[test]
fn test_metadata_extra_keys() {
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_extra_keys.ipynb.expected",
        &["--extra-keys", "metadata.kernelspec,metadata.language_info"],
    );
}

#[test]
fn test_metadata_keep_count() {
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_keep_count.ipynb.expected",
        &["--keep-count"],
    );
}
#[test]
fn test_metadata_keep_output() {
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_keep_output.ipynb.expected",
        &["--keep-output"],
    );
}
#[test]
fn test_metadata_keep_output_keep_count() {
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_keep_output_keep_count.ipynb.expected",
        &["--keep-output", "--keep-count"],
    );
}
#[test]
fn test_metadata_notebook() {
    test_expected(
        "tests/e2e_notebooks/test_metadata_notebook.ipynb",
        "tests/e2e_notebooks/test_metadata_notebook.ipynb.expected",
        &[],
    );
}

#[test]
fn test_keep_metadata_keys() {
    test_expected(
        "tests/e2e_notebooks/test_keep_metadata_keys.ipynb",
        "tests/e2e_notebooks/test_keep_metadata_keys.ipynb.expected",
        &[
            "--keep-keys",
            "cell.metadata.scrolled,cell.metadata.collapsed,metadata.a",
        ],
    );
}
#[test]
fn test_metadata_period() {
    test_expected(
        "tests/e2e_notebooks/test_metadata_period.ipynb",
        "tests/e2e_notebooks/test_metadata_period.ipynb.expected",
        &["--extra-keys", "cell.metadata.application/vnd.databricks.v1+cell,metadata.application/vnd.databricks.v1+notebook"],
    );
}
#[test]
fn test_strip_init_cells() {
    test_expected(
        "tests/e2e_notebooks/test_strip_init_cells.ipynb",
        "tests/e2e_notebooks/test_strip_init_cells.ipynb.expected",
        &["--strip-init-cells"],
    );
}
#[test]
fn test_nbformat45() {
    test_expected(
        "tests/e2e_notebooks/test_nbformat45.ipynb",
        "tests/e2e_notebooks/test_nbformat45.ipynb.expected",
        &["--keep-id"],
    );
}
#[test]
fn test_nbformat45_expected_sequential_id() {
    test_expected(
        "tests/e2e_notebooks/test_nbformat45.ipynb",
        "tests/e2e_notebooks/test_nbformat45.ipynb.expected_sequential_id",
        &["--drop-id"],
    );
}
#[test]
fn test_unicode() {
    test_expected(
        "tests/e2e_notebooks/test_unicode.ipynb",
        "tests/e2e_notebooks/test_unicode.ipynb.expected",
        &[],
    );
}
#[test]
fn test_widgets() {
    test_expected(
        "tests/e2e_notebooks/test_widgets.ipynb",
        "tests/e2e_notebooks/test_widgets.ipynb.expected",
        &[],
    );
}
