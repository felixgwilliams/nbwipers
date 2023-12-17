use std::{fs, path::PathBuf, process::Command};

use bstr::ByteSlice;
#[test]
fn test_install() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    {
        let config_file = temp_dir.path().join("gitconfig");
        let attr_file = temp_dir.path().join("attributes");
        let output = Command::new(&cur_exe)
            .args([
                "install",
                "local",
                "-g",
                config_file.to_str().unwrap(),
                "-a",
                attr_file.to_str().unwrap(),
            ])
            .output()
            .expect("command failed");
        assert!(output.status.success());
        let config_file_contents = fs::read_to_string(&config_file).unwrap();
        let attr_file_contents = fs::read_to_string(&attr_file).unwrap();

        assert!(config_file_contents.contains("nbwipers"));
        assert!(attr_file_contents.contains("nbwipers"));

        let output = Command::new(&cur_exe)
            .args([
                "uninstall",
                "local",
                "-g",
                config_file.to_str().unwrap(),
                "-a",
                attr_file.to_str().unwrap(),
            ])
            .output()
            .expect("command failed");
        assert!(output.status.success());

        let config_file_contents = fs::read_to_string(&config_file).unwrap();
        let attr_file_contents = fs::read_to_string(&attr_file).unwrap();

        assert!(!config_file_contents.contains("nbwipers"));
        assert!(!attr_file_contents.contains("nbwipers"));
    }
}

#[test]
fn test_check_install() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let git_init_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["init"])
        .output()
        .expect("git init failed");
    assert!(git_init_out.status.success());
    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["install", "local"])
        .output()
        .expect("command failed");
    assert!(output.status.success());
    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["check-install"])
        .output()
        .expect("command failed");
    assert!(output.status.success());
    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["check-install", "local"])
        .output()
        .expect("command failed");
    assert!(output.status.success());

    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["uninstall", "local"])
        .output()
        .expect("command failed");
    assert!(output.status.success());

    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["check-install"])
        .output()
        .expect("command failed");
    assert!(!output.status.success());
    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["check-install", "local"])
        .output()
        .expect("command failed");
    assert!(!output.status.success());
}

#[test]
fn test_invalid_format() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let output = Command::new(cur_exe)
        .args(["check", "tests/test_nbformat2.ipynb"])
        .output()
        .expect("command failed");
    assert!(&output.stdout.contains_str(b"Invalid notebook:"))
}

#[test]
fn test_file_not_found() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let output = Command::new(cur_exe)
        .args(["check", "tests/test_nbformat2.ipynb"])
        .args(["-c", "bananas.toml"])
        .output()
        .expect("command failed");
    assert!(!output.status.success());
    assert!(&output.stderr.contains_str(b"Pyproject IO Error"))
}

#[test]
fn test_strip_all() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let dest_file = temp_dir.path().join("test_nbformat45.ipynb");
    fs::copy("tests/e2e_notebooks/test_nbformat45.ipynb", dest_file).unwrap();
    dbg!(temp_dir.path());
    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args(["clean-all", ".", "-y"])
        .output()
        .expect("command failed");

    let stdout = output.stdout.to_str_lossy();
    assert!(output.status.success());
    assert!(stdout.ends_with("Stripped\n"));

    let output = Command::new(cur_exe)
        .current_dir(temp_dir.path())
        .args(["clean-all", "-y", "."])
        .output()
        .expect("command failed");

    let stdout = output.stdout.to_str_lossy();
    assert!(output.status.success());
    assert!(stdout.ends_with("No Change\n"));
}

#[test]
fn test_strip_all_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let dest_file = temp_dir.path().join("test_nbformat2.ipynb");
    fs::copy("tests/test_nbformat2.ipynb", dest_file).unwrap();
    let output = Command::new(cur_exe)
        .current_dir(temp_dir.path())
        .args(["clean-all", "-y", "."])
        .output()
        .expect("command failed");

    let stdout = output.stdout.to_str_lossy();
    dbg!(&stdout);
    assert!(!output.status.success());
    assert!(stdout.contains("Read error"));
}
