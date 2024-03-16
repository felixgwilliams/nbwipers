use std::{env, fs, path::PathBuf, process::Command};

use bstr::ByteSlice;

#[test]
fn test_no_notebooks() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    {
        let py_file = temp_dir.path().join("script.py");
        fs::write(py_file, "print('hello, world')").unwrap();

        let output = Command::new(&cur_exe)
            .current_dir(&temp_dir)
            .args(["check", "."])
            .output()
            .expect("command failed");
        assert!(!output.status.success());
        assert!(output
            .stderr
            .to_str()
            .unwrap()
            .contains("Error: Could not find any notebooks in path(s)"));
        let output = Command::new(&cur_exe)
            .current_dir(&temp_dir)
            .args(["check", ".", "--allow-no-notebooks"])
            .output()
            .expect("command failed");
        assert!(output.status.success());
    }
}

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

        let config_file_contents2 = fs::read_to_string(&config_file).unwrap();
        let attr_file_contents2 = fs::read_to_string(&attr_file).unwrap();
        assert!(config_file_contents == config_file_contents2);
        assert!(attr_file_contents == attr_file_contents2);

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
fn test_config_subdir() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    {
        let config_file = temp_dir.path().join("new/subdir/").join("gitconfig");
        let attr_file = temp_dir.path().join("new/subdir/").join("attributes");

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
        dbg!(output.stdout.as_bstr());
        assert!(output.status.success());
        assert!(config_file.exists());

        let config_file_contents = fs::read_to_string(&config_file).unwrap();
        let attr_file_contents = fs::read_to_string(&attr_file).unwrap();

        let _output = Command::new(&cur_exe)
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
        assert_eq!(
            config_file_contents,
            fs::read_to_string(&config_file).unwrap(),
        );
        assert_eq!(attr_file_contents, fs::read_to_string(&attr_file).unwrap());
    }
}
#[test]
fn test_uninstall_nothing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    {
        let config_file = temp_dir.path().join("bananas");
        let attributes_file = temp_dir.path().join("pineapples");
        let output = Command::new(cur_exe)
            .args([
                "uninstall",
                "local",
                "-g",
                config_file.to_str().unwrap(),
                "-a",
                attributes_file.to_str().unwrap(),
            ])
            .output()
            .expect("command failed");
        dbg!(output.stdout.as_bstr());

        assert!(output.status.success());
    }
}

#[test]
fn test_partial_install() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    {
        let config_file = temp_dir.path().join("gitconfig");
        let attr_file = temp_dir.path().join("attributes");

        fs::write(
            &config_file,
            r#"
        [filter "nbwipers"]
        clean = \"/home/felix/.local/pipx/venvs/nbwipers/bin/nbwipers\" clean -
        smudge = cat
        "#,
        )
        .unwrap();
        fs::write(
            &attr_file,
            r#"*.ipynb filter=nbwipers
*.ipynb filter=banana"#,
        )
        .unwrap();

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
    }
}

#[test]
fn test_handle_multiple_assignments() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    {
        let config_file = temp_dir.path().join("gitconfig");
        let attr_file = temp_dir.path().join("attributes");

        fs::write(
            &config_file,
            r#"
[filter "nbwipers"]
        clean = \"/home/felix/.local/pipx/venvs/nbwipers/bin/nbwipers\" clean -
        smudge = cat
"#,
        )
        .unwrap();
        fs::write(
            &attr_file,
            r#"
            *.ipynb filter=nbwipers filter=banana argh
        "#,
        )
        .unwrap();

        let output = Command::new(cur_exe)
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
        dbg!(output.stdout.as_bstr());
        dbg!(output.stderr.as_bstr());
        let attr_file_contents = fs::read_to_string(&attr_file).unwrap();
        dbg!(attr_file_contents);
        assert!(output.status.success());
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

    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["check-install", "--exit-zero"])
        .output()
        .expect("command failed");
    assert!(output.status.success());

    env::set_var("NBWIPERS_CHECK_INSTALL_EXIT_ZERO", "1");
    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["check-install"])
        .output()
        .expect("command failed");
    assert!(output.status.success());

    let attr_file = temp_dir.path().join(".gitattributes");
    let config_file = temp_dir.path().join(".git/config");
    fs::write(
        attr_file,
        r#"*.ipynb filter=nbstripout
*.zpln filter=nbstripout
*.ipynb diff=ipynb"#,
    )
    .unwrap();
    fs::write(
        config_file,
        r#"[filter "nbstripout"]
        clean = \"/home/felix/mambaforge/bin/python3.11\" -m nbstripout
        smudge = cat
[diff "ipynb"]
        textconv = \"/home/felix/mambaforge/bin/python3.11\" -m nbstripout -t
"#,
    )
    .unwrap();
    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["check-install", "local"])
        .output()
        .expect("command failed");
    assert!(output.status.success());
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

#[test]
fn test_check_all() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let output = Command::new(cur_exe)
        .current_dir("tests/e2e_notebooks")
        .args(["check", ".", "--drop-empty-cells", "--drop-id"])
        .output()
        .expect("command failed");

    // let stdout = output.stdout.to_str_lossy();
    assert!(!output.status.success());
}
