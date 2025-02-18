use bstr::ByteSlice;
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

fn test_expected(path: &str, expected: &str, extra_args: &[&str], snapshot_name: &str) {
    // clean and compare with expected content
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let output = Command::new(&cur_exe)
        .args(["clean", "-t", path])
        .args(extra_args)
        .output()
        .expect("command failed");

    let expected_content = fs::read_to_string(expected).expect("could not read expected");

    assert_eq!(output.stdout.to_str().unwrap(), expected_content);
    let mut extra_args_check = extra_args.to_owned();
    extra_args_check.retain(|v| *v != "--respect-exclusions");
    // check no errors after cleaning
    let mut check_output_cmd = Command::new(&cur_exe)
        .args(["check", "-"])
        .args(&extra_args_check)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("command failed");
    {
        let mut check_in = check_output_cmd.stdin.take().expect("Failed to open stdin");
        // write!(check_in, "{expected_content}").expect("Failed to write to stdin");
        check_in
            .write_all(expected_content.as_bytes())
            .expect("Failed to write to stdin");
    }
    let check_output = check_output_cmd.wait_with_output().expect("Command failed");

    dbg!("{}", check_output.stdout.to_str().unwrap());
    dbg!("{}", check_output.stderr.to_str().unwrap());

    assert!(check_output.status.success());

    // snapshot test for check output
    let output = Command::new(&cur_exe)
        .args(["check", path, "-o", "json"])
        .args(&extra_args_check)
        .output()
        .expect("command failed");
    insta::assert_snapshot!(
        format!("{snapshot_name}_json"),
        output.stdout.to_str().unwrap()
    );
    let output = Command::new(&cur_exe)
        .args(["check", path, "-o", "text"])
        .args(&extra_args_check)
        .output()
        .expect("command failed");
    insta::assert_snapshot!(
        format!("{snapshot_name}_text"),
        output.stdout.to_str().unwrap()
    );
}

fn test_config_match(config_file: &str, extra_args: &[&str]) {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let output = Command::new(&cur_exe)
        .args(["show-config", "-c", config_file])
        .output()
        .expect("command failed");
    let output_args = Command::new(&cur_exe)
        .args(["show-config", "--isolated"])
        .args(extra_args)
        .output()
        .expect("command failed");
    assert_eq!(
        output.stdout.to_str().unwrap(),
        output_args.stdout.to_str().unwrap()
    );

    let output = Command::new(&cur_exe)
        .args(["show-config", "--show-all", "-c", config_file])
        .output()
        .expect("command failed");
    let output_args = Command::new(&cur_exe)
        .args(["show-config", "--show-all", "--isolated"])
        .args(extra_args)
        .output()
        .expect("command failed");
    assert_eq!(
        output.stdout.to_str().unwrap(),
        output_args.stdout.to_str().unwrap()
    );
}

fn test_config_args_match(extra_args_left: &[&str], extra_args_right: &[&str]) {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let output_left = Command::new(&cur_exe)
        .args(["show-config", "--isolated"])
        .args(extra_args_left)
        .output()
        .expect("command failed");
    let output_right = Command::new(&cur_exe)
        .args(["show-config", "--isolated"])
        .args(extra_args_right)
        .output()
        .expect("command failed");
    assert_eq!(
        output_left.stdout.to_str().unwrap(),
        output_right.stdout.to_str().unwrap()
    );

    let output_left = Command::new(&cur_exe)
        .args(["show-config", "--show-all", "--isolated"])
        .args(extra_args_left)
        .output()
        .expect("command failed");
    let output_right = Command::new(&cur_exe)
        .args(["show-config", "--show-all", "--isolated"])
        .args(extra_args_right)
        .output()
        .expect("command failed");
    assert_eq!(
        output_left.stdout.to_str().unwrap(),
        output_right.stdout.to_str().unwrap()
    );
}

#[test]
fn test_drop_empty_cells_dontdrop() {
    test_expected(
        "tests/e2e_notebooks/test_drop_empty_cells.ipynb",
        "tests/e2e_notebooks/test_drop_empty_cells_dontdrop.ipynb.expected",
        &[],
        "test_drop_empty_cells_dontdrop",
    );
}

#[test]
fn test_drop_empty_cells() {
    test_expected(
        "tests/e2e_notebooks/test_drop_empty_cells.ipynb",
        "tests/e2e_notebooks/test_drop_empty_cells.ipynb.expected",
        &["--drop-empty-cells"],
        "test_drop_empty_cells_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_drop_empty_cells.ipynb",
        "tests/e2e_notebooks/test_drop_empty_cells.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_drop_empty_cells.toml"],
        "test_drop_empty_cells_cfg",
    );
    test_config_match(
        "tests/e2e_notebooks/test_drop_empty_cells.toml",
        &["--drop-empty-cells"],
    );
}

#[test]
fn test_drop_tagged_cells_dontdrop() {
    test_expected(
        "tests/e2e_notebooks/test_drop_tagged_cells.ipynb",
        "tests/e2e_notebooks/test_drop_tagged_cells_dontdrop.ipynb.expected",
        &[],
        "test_drop_tagged_cells_dontdrop",
    );
}

#[test]
fn test_drop_tagged_cells() {
    test_expected(
        "tests/e2e_notebooks/test_drop_tagged_cells.ipynb",
        "tests/e2e_notebooks/test_drop_tagged_cells.ipynb.expected",
        &["--drop-tagged-cells=test"],
        "test_drop_tagged_cells_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_drop_tagged_cells.ipynb",
        "tests/e2e_notebooks/test_drop_tagged_cells.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_drop_tagged_cells.toml"],
        "test_drop_tagged_cells_cfg",
    );
    test_config_match(
        "tests/e2e_notebooks/test_drop_tagged_cells.toml",
        &["--drop-tagged-cells=test"],
    );
}
#[test]
fn test_execution_timing() {
    test_expected(
        "tests/e2e_notebooks/test_execution_timing.ipynb",
        "tests/e2e_notebooks/test_execution_timing.ipynb.expected",
        &["--drop-tagged-cells=test"],
        "test_execution_timing_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_execution_timing.ipynb",
        "tests/e2e_notebooks/test_execution_timing.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_drop_tagged_cells.toml"],
        "test_execution_timing_cfg",
    );
    test_config_match(
        "tests/e2e_notebooks/test_drop_tagged_cells.toml",
        &["--drop-tagged-cells=test"],
    );
}
#[test]
fn test_metadata() {
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata.ipynb.expected",
        &[],
        "test_metadata",
    );
}
#[test]
fn test_metadata_extra_keys() {
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_extra_keys.ipynb.expected",
        &["--extra-keys", "metadata.kernelspec,metadata.language_info"],
        "test_metadata_extra_keys_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_extra_keys.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_metadata_extra_keys.toml"],
        "test_metadata_extra_keys_cfg",
    );

    test_config_match(
        "tests/e2e_notebooks/test_metadata_extra_keys.toml",
        &["--extra-keys", "metadata.kernelspec,metadata.language_info"],
    );
}

#[test]
fn test_strip_kernel_info() {
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_strip_kernelinfo.ipynb.expected",
        &[
            "--extra-keys",
            "metadata.kernelspec,metadata.language_info.version",
        ],
        "test_metadata_strip_kernelinfo_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_strip_kernelinfo.ipynb.expected",
        &[
            "-c",
            "tests/e2e_notebooks/test_metadata_strip_kernelinfo.toml",
        ],
        "test_metadata_strip_kernelinfo_cfg",
    );
}

#[test]
fn test_metadata_keep_count() {
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_keep_count.ipynb.expected",
        &["--keep-count"],
        "test_metadata_keep_count_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_keep_count.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_metadata_keep_count.toml"],
        "test_metadata_keep_count_cfg",
    );
    test_config_match(
        "tests/e2e_notebooks/test_metadata_keep_count.toml",
        &["--keep-count"],
    );
}
#[test]
fn test_metadata_keep_output() {
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_keep_output.ipynb.expected",
        &["--keep-output"],
        "test_metadata_keep_output_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_keep_output.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_metadata_keep_output.toml"],
        "test_metadata_keep_output_cfg",
    );
    test_config_match(
        "tests/e2e_notebooks/test_metadata_keep_output.toml",
        &["--keep-output"],
    );
}
#[test]
fn test_metadata_keep_output_keep_count() {
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_keep_output_keep_count.ipynb.expected",
        &["--keep-output", "--keep-count"],
        "test_metadata_keep_output_keep_count_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_metadata.ipynb",
        "tests/e2e_notebooks/test_metadata_keep_output_keep_count.ipynb.expected",
        &[
            "-c",
            "tests/e2e_notebooks/test_metadata_keep_output_keep_count.toml",
        ],
        "test_metadata_keep_output_keep_count_cfg",
    );

    test_config_match(
        "tests/e2e_notebooks/test_metadata_keep_output_keep_count.toml",
        &["--keep-output", "--keep-count"],
    );
}
#[test]
fn test_metadata_notebook() {
    test_expected(
        "tests/e2e_notebooks/test_metadata_notebook.ipynb",
        "tests/e2e_notebooks/test_metadata_notebook.ipynb.expected",
        &[],
        "test_metadata_notebook",
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
        "test_keep_metadata_keys_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_keep_metadata_keys.ipynb",
        "tests/e2e_notebooks/test_keep_metadata_keys.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_keep_metadata_keys.toml"],
        "test_keep_metadata_keys_cfg",
    );

    test_config_match(
        "tests/e2e_notebooks/test_keep_metadata_keys.toml",
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
        "test_metadata_period_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_metadata_period.ipynb",
        "tests/e2e_notebooks/test_metadata_period.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_metadata_period.toml"],
        "test_metadata_period_cfg",
    );

    test_config_match(
        "tests/e2e_notebooks/test_metadata_period.toml",
        &["--extra-keys", "cell.metadata.application/vnd.databricks.v1+cell,metadata.application/vnd.databricks.v1+notebook"],
    );
}
#[test]
fn test_strip_init_cells() {
    test_expected(
        "tests/e2e_notebooks/test_strip_init_cells.ipynb",
        "tests/e2e_notebooks/test_strip_init_cells.ipynb.expected",
        &["--strip-init-cell"],
        "test_strip_init_cells_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_strip_init_cells.ipynb",
        "tests/e2e_notebooks/test_strip_init_cells.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_strip_init_cells.toml"],
        "test_strip_init_cells_cfg",
    );

    test_config_match(
        "tests/e2e_notebooks/test_strip_init_cells.toml",
        &["--strip-init-cell"],
    );
}
#[test]
fn test_nbformat45() {
    test_expected(
        "tests/e2e_notebooks/test_nbformat45.ipynb",
        "tests/e2e_notebooks/test_nbformat45.ipynb.expected",
        &["--keep-id"],
        "test_nbformat45_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_nbformat45.ipynb",
        "tests/e2e_notebooks/test_nbformat45.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_nbformat45.toml"],
        "test_nbformat45_cfg",
    );

    test_config_match("tests/e2e_notebooks/test_nbformat45.toml", &["--keep-id"]);
}
#[test]
fn test_strip_exclude() {
    // if we clean a single file, our exclusion gets ignored
    test_expected(
        "tests/e2e_notebooks/test_nbformat45.ipynb",
        "tests/e2e_notebooks/test_nbformat45.ipynb.expected",
        &["--keep-id", "--exclude", "test_nbformat45.ipynb"],
        "test_strip_exclude_cli",
    );
    // if we add respect exclusions, the file does not get touched
    test_expected(
        "tests/e2e_notebooks/test_nbformat45.ipynb",
        "tests/e2e_notebooks/test_nbformat45.ipynb",
        &[
            "--keep-id",
            "--exclude",
            "test_nbformat45.ipynb",
            "--respect-exclusions",
            "--stdin-file-name",
            "tests/e2e_notebooks/test_nbformat45.ipynb",
        ],
        "test_strip_exclude_respect_cli",
    );
}
#[test]
fn test_nbformat45_expected_sequential_id() {
    test_expected(
        "tests/e2e_notebooks/test_nbformat45.ipynb",
        "tests/e2e_notebooks/test_nbformat45.sequential_id.ipynb.expected",
        &["--sequential-id"],
        "test_nbformat45_expected_sequential_id_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_nbformat45.ipynb",
        "tests/e2e_notebooks/test_nbformat45.sequential_id.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_nbformat45_sequential.toml"],
        "test_nbformat45_expected_sequential_id_cfg",
    );
    test_config_match(
        "tests/e2e_notebooks/test_nbformat45_sequential.toml",
        &["--sequential-id"],
    );
}
#[test]
fn test_nbformat45_expected_drop_id() {
    test_expected(
        "tests/e2e_notebooks/test_nbformat45.ipynb",
        "tests/e2e_notebooks/test_nbformat45.drop_id.ipynb.expected",
        &["--drop-id"],
        "test_nbformat45_expected_drop_id_cli",
    );
    test_expected(
        "tests/e2e_notebooks/test_nbformat45.ipynb",
        "tests/e2e_notebooks/test_nbformat45.drop_id.ipynb.expected",
        &["-c", "tests/e2e_notebooks/test_nbformat45_drop.toml"],
        "test_nbformat45_expected_drop_id_cfg",
    );
    test_config_match(
        "tests/e2e_notebooks/test_nbformat45_drop.toml",
        &["--drop-id"],
    );
}
#[test]
fn test_unicode() {
    test_expected(
        "tests/e2e_notebooks/test_unicode.ipynb",
        "tests/e2e_notebooks/test_unicode.ipynb.expected",
        &[],
        "test_unicode",
    );
}
#[test]
fn test_widgets() {
    test_expected(
        "tests/e2e_notebooks/test_widgets.ipynb",
        "tests/e2e_notebooks/test_widgets.ipynb.expected",
        &[],
        "test_widgets",
    );
}

#[test]
fn test_id_action_config() {
    test_config_args_match(&["--keep-id"], &["--id-action=keep"]);
    test_config_args_match(&["--drop-id"], &["--id-action=drop"]);
    test_config_args_match(&["--sequential-id"], &["--id-action=sequential"]);
    // only look at the last value
    test_config_args_match(
        &["--id-action=keep", "--sequential-id"],
        &["--id-action=sequential"],
    );
}

#[test]
fn test_exclusions() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    // the notebooks are full of issues
    let output = Command::new(&cur_exe)
        .args(["check", "tests/e2e_notebooks", "-o", "text"])
        // .args([])
        .output()
        .expect("command failed");
    assert!(!output.status.success());
    // but if we exclude them, it should pass
    let output = Command::new(&cur_exe)
        .args(["check", "tests/e2e_notebooks", "-o", "text"])
        .args(["--exclude", "tests/e2e_notebooks", "--allow-no-notebooks"])
        .output()
        .expect("command failed");
    dbg!(output.stderr.as_bstr());
    assert!(output.status.success());
    // and let's exclude them another way
    let output = Command::new(&cur_exe)
        .args(["check", "tests/e2e_notebooks", "-o", "text"])
        .args([
            "--extend-exclude",
            "tests/e2e_notebooks",
            "--allow-no-notebooks",
        ])
        .output()
        .expect("command failed");
    dbg!(output.stderr.as_bstr());
    assert!(output.status.success());
}

#[test]
fn test_strip_stdin() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let original = fs::read_to_string("tests/e2e_notebooks/test_nbformat45.ipynb")
        .expect("could not read expected");
    let cleaned = fs::read_to_string("tests/e2e_notebooks/test_nbformat45.ipynb.expected")
        .expect("could not read expected");

    // if we do not exclude anything then the file will be stripped
    let mut output = Command::new(&cur_exe)
        .args([
            "clean",
            "-t",
            "-",
            "--stdin-file-name",
            "tests/e2e_notebooks/test_nbformat45.ipynb",
            "--respect-exclusions",
        ])
        // .args(extra_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("command failed");
    {
        let mut clean_in = output.stdin.take().expect("Failed to open stdin");
        clean_in
            .write_all(original.as_bytes())
            .expect("Failed to write to stdin");
    }
    let clean_output = output.wait_with_output().expect("Command failed");
    assert_eq!(&cleaned, clean_output.stdout.to_str().unwrap());

    // if we exclude but don't give the name, it won't be stripped
    let mut output_noname = Command::new(&cur_exe)
        .args([
            "clean",
            "-t",
            "-",
            "--respect-exclusions",
            "--exclude",
            "test_nbformat45.ipynb",
        ])
        // .args(extra_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("command failed");
    {
        let mut clean_in = output_noname.stdin.take().expect("Failed to open stdin");
        clean_in
            .write_all(original.as_bytes())
            .expect("Failed to write to stdin");
    }
    let clean_output_noname = output_noname.wait_with_output().expect("Command failed");
    assert_eq!(&cleaned, clean_output_noname.stdout.to_str().unwrap());

    // if we do not respect exclusions, it will be stripped
    let mut output_no_respect = Command::new(&cur_exe)
        .args([
            "clean",
            "-t",
            "-",
            "--exclude",
            "test_nbformat45.ipynb",
            "--stdin-file-name",
            "tests/e2e_notebooks/test_nbformat45.ipynb",
        ])
        // .args(extra_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("command failed");
    {
        let mut clean_in = output_no_respect
            .stdin
            .take()
            .expect("Failed to open stdin");
        clean_in
            .write_all(original.as_bytes())
            .expect("Failed to write to stdin");
    }
    let clean_output_no_respect = output_no_respect
        .wait_with_output()
        .expect("Command failed");
    assert_eq!(&cleaned, clean_output_no_respect.stdout.to_str().unwrap());

    // if we exclude and give the file name it will not be stripped
    let mut output_exclude = Command::new(&cur_exe)
        .args([
            "clean",
            "-t",
            "-",
            "--exclude",
            "test_nbformat45.ipynb",
            "--stdin-file-name",
            "tests/e2e_notebooks/test_nbformat45.ipynb",
            "--respect-exclusions",
        ])
        // .args(extra_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("command failed");
    {
        let mut clean_in = output_exclude.stdin.take().expect("Failed to open stdin");
        clean_in
            .write_all(original.as_bytes())
            .expect("Failed to write to stdin");
    }
    let clean_output_exclude = output_exclude.wait_with_output().expect("Command failed");
    assert_eq!(&original, clean_output_exclude.stdout.to_str().unwrap());
}
