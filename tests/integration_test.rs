use std::{env, fs, io::BufWriter, io::Write, path::PathBuf, process::Command, process::Stdio};

use bstr::ByteSlice;
use nbwipers::{
    schema::{Cell, CodeCell, RawNotebook, SourceValue},
    strip::write_nb,
};
use serde_json::{Value, json};

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
        assert!(
            output
                .stderr
                .to_str()
                .unwrap()
                .contains("Error: Could not find any notebooks in path(s)")
        );
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
fn test_uninstall_preserves_comments() {
    // regression test: uninstall used to bail with "No pattern found in line"
    // on comment lines, leaving the nbwipers entries in place
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let config_file = temp_dir.path().join("gitconfig");
    let attr_file = temp_dir.path().join("attributes");

    fs::write(
        &config_file,
        "[filter \"nbwipers\"]\n\tclean = nbwipers clean -\n\tsmudge = cat\n",
    )
    .unwrap();
    fs::write(
        &attr_file,
        r"# strip notebook outputs
*.ipynb filter=nbwipers
*.ipynb diff=nbwipers
[attr]myattr text
*.py text
",
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
    dbg!(output.stderr.as_bstr());
    assert!(output.status.success());

    let attr_file_contents = fs::read_to_string(&attr_file).unwrap();
    assert!(attr_file_contents.contains("# strip notebook outputs"));
    assert!(attr_file_contents.contains("[attr]myattr text"));
    assert!(attr_file_contents.contains("*.py text"));
    assert!(!attr_file_contents.contains("nbwipers"));
}

#[test]
fn test_uninstall_invalid_attribute_line() {
    // git itself rejects negative patterns in .gitattributes, so uninstall
    // should surface the parse error rather than silently rewriting the file
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let config_file = temp_dir.path().join("gitconfig");
    let attr_file = temp_dir.path().join("attributes");

    fs::write(&config_file, "[filter \"nbwipers\"]\n\tclean = x\n").unwrap();
    let attr_contents = "!*.ipynb filter=nbwipers\n*.ipynb filter=nbwipers\n";
    fs::write(&attr_file, attr_contents).unwrap();

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
    assert!(!output.status.success());
    assert!(
        output
            .stderr
            .to_str()
            .unwrap()
            .contains("has a negative pattern")
    );
    // the attribute file must be left untouched on error
    assert_eq!(fs::read_to_string(&attr_file).unwrap(), attr_contents);
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
        .args(["install", "local", "-a", ".gitattributes"])
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
    unsafe {
        env::set_var("NBWIPERS_CHECK_INSTALL_EXIT_ZERO", "1");
    }
    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["check-install"])
        .output()
        .expect("command failed");
    assert!(output.status.success());

    let attr_file = temp_dir.path().join(".gitattributes");
    let config_file = temp_dir.path().join(".git/config");
    fs::write(
        &attr_file,
        r#"*.ipynb filter=nbstripout
*.zpln filter=nbstripout
*.ipynb diff=ipynb"#,
    )
    .unwrap();
    fs::write(
        &config_file,
        r#"[filter "nbstripout"]
        clean = \"python3.11\" -m nbstripout
        smudge = cat
[diff "ipynb"]
        textconv = \"python3.11\" -m nbstripout -t
"#,
    )
    .unwrap();
    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["check-install", "local"])
        .output()
        .expect("command failed");
    assert!(output.status.success());

    fs::write(&attr_file, "").unwrap();
    fs::write(&config_file, "").unwrap();
}

#[test]
fn test_check_install_fresh_repo() {
    // a git repo where nothing has ever been installed: the default
    // `.gitattributes` and git config don't exist yet, so nbwipers should
    // report cleanly that it isn't installed rather than erroring.
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
fn test_check_install_bare_repo() {
    // a bare repo has no work tree, so there's nowhere for a default
    // `.gitattributes` to live; nbwipers should treat that as "not installed"
    // rather than erroring.
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let git_init_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["init", "--bare"])
        .output()
        .expect("git init failed");
    assert!(git_init_out.status.success());

    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["check-install", "local"])
        .output()
        .expect("command failed");
    assert!(!output.status.success());
}

#[test]
fn test_uninstall_fresh_repo_defaults() {
    // uninstall using the default (non-explicit) git config file and
    // `.gitattributes` resolution, in a repo where nothing was ever
    // installed. Both files need their parent directories/resolution paths
    // handled even though neither file exists yet.
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
        .args(["uninstall", "local"])
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
fn test_clean_all_stdin_not_supported() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let output = Command::new(cur_exe)
        .args(["clean-all", "-"])
        .output()
        .expect("command failed");
    assert!(!output.status.success());
    assert!(
        output
            .stderr
            .to_str()
            .unwrap()
            .contains("`strip-all` does not support stdin")
    );
}

#[test]
fn test_check_all() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let output = Command::new(cur_exe)
        .current_dir("tests/e2e_notebooks")
        .args([
            "check",
            ".",
            "../test_nbformat2.ipynb",
            "--drop-empty-cells",
            "--drop-id",
        ])
        .output()
        .expect("command failed");

    // let stdout = output.stdout.to_str_lossy();
    assert!(!output.status.success());
}
#[test]
fn test_nothing() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let output = Command::new(cur_exe)
        .args(["check", "--drop-empty-cells", "--drop-id"])
        .output()
        .expect("command failed");

    // let stdout = output.stdout.to_str_lossy();
    assert!(!output.status.success());
}

#[test]
fn test_large_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let git_init_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["init"])
        .output()
        .expect("git init failed");
    assert!(git_init_out.status.success());

    let mut nb = RawNotebook::default();

    let big_cell = json!([
     {
      "name": "stdout",
      "output_type": "stream",
      "text": [
       "a".repeat(1000*1024)
      ]
     }
    ]);

    nb.cells.push(Cell::Code(CodeCell {
        source: SourceValue::StringArray(vec!["# This is a test".to_string()]),
        metadata: Value::Null,
        execution_count: Some(1),
        outputs: vec![big_cell],
        id: None,
    }));
    let large_file_path = temp_dir.path().join("large_nb.ipynb");

    {
        let f = fs::File::create(large_file_path).unwrap();
        let writer = BufWriter::new(f);
        write_nb(writer, &nb).unwrap();
    }
    {
        fs::write(
            temp_dir.path().join(".nbwipers.toml"),
            "exclude = [\"large*.ipynb\"]\n",
        )
        .unwrap();
    }
    {
        fs::write(
            temp_dir.path().join("invalid.ipynb"),
            "a".repeat(1000 * 1024),
        )
        .unwrap();
    }
    // our large file wasn't added so isn't looked at
    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args(["hook", "check-large-files", "large_nb.ipynb"])
        .output()
        .expect("command failed");

    assert!(output.status.success());
    // the nbwipers file tells us not to strip this file, so it trips the check
    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args([
            "hook",
            "check-large-files",
            "--enforce-all",
            "large_nb.ipynb",
        ])
        .output()
        .expect("command failed");

    assert!(!output.status.success());

    // now we ignore the config file, and it gets stripped when checking the size
    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args([
            "hook",
            "check-large-files",
            "--enforce-all",
            "--isolated",
            "large_nb.ipynb",
        ])
        .output()
        .expect("command failed");

    assert!(output.status.success());

    // now we change maxkb to allow very large files and it passes again
    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args([
            "hook",
            "check-large-files",
            "--enforce-all",
            "--maxkb=2048",
            "large_nb.ipynb",
        ])
        .output()
        .expect("command failed");

    assert!(output.status.success());

    // now we try an invalid file, and it fails to strip anything
    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args([
            "hook",
            "check-large-files",
            "--enforce-all",
            "invalid.ipynb",
        ])
        .output()
        .expect("command failed");
    dbg!(output.stdout.as_bstr());
    dbg!(output.stderr.as_bstr());
    assert!(!output.status.success());
    assert!(
        output
            .stderr
            .as_bstr()
            .contains_str(b"Could not parse nb file")
    );

    let git_add_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["add", "large_nb.ipynb"])
        .output()
        .expect("git init failed");
    dbg!(git_add_out.stdout.as_bstr());
    dbg!(git_add_out.stderr.as_bstr());
    assert!(git_add_out.status.success());

    // now check we fail after the file has been git added
    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args(["hook", "check-large-files", "large_nb.ipynb"])
        .output()
        .expect("command failed");

    assert!(!output.status.success());
}

#[test]
fn test_large_files_lfs_skip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let git_init_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["init"])
        .output()
        .expect("git init failed");
    assert!(git_init_out.status.success());

    // a large, non-notebook-JSON file so its raw on-disk size is used directly
    fs::write(
        temp_dir.path().join("lfs_nb.ipynb"),
        "a".repeat(1000 * 1024),
    )
    .unwrap();

    // without any lfs attribute, the large file trips the check
    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args(["hook", "check-large-files", "--enforce-all", "lfs_nb.ipynb"])
        .output()
        .expect("command failed");
    assert!(!output.status.success());

    // mark it as lfs-tracked; the hook measures on-disk size, not the (small)
    // lfs pointer file git would actually store, so it should skip it entirely
    fs::write(
        temp_dir.path().join(".gitattributes"),
        "lfs_nb.ipynb filter=lfs\n",
    )
    .unwrap();

    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args(["hook", "check-large-files", "--enforce-all", "lfs_nb.ipynb"])
        .output()
        .expect("command failed");
    assert!(output.status.success());
}

#[test]
fn test_large_files_non_ipynb() {
    // non-notebook files are measured by their on-disk size directly
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let git_init_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["init"])
        .output()
        .expect("git init failed");
    assert!(git_init_out.status.success());

    fs::write(temp_dir.path().join("big.py"), "a".repeat(1000 * 1024)).unwrap();

    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args(["hook", "check-large-files", "--enforce-all", "big.py"])
        .output()
        .expect("command failed");
    assert!(!output.status.success());
    assert!(output.stdout.to_str().unwrap().contains("exceeds"));

    // a higher limit lets the same file pass
    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args([
            "hook",
            "check-large-files",
            "--enforce-all",
            "--maxkb",
            "2000",
            "big.py",
        ])
        .output()
        .expect("command failed");
    assert!(output.status.success());
}

#[test]
fn test_large_files_outside_git_repo() {
    // the hook shells out to `git check-attr`, which fails outside a repo
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    fs::write(temp_dir.path().join("big.py"), "a".repeat(1000 * 1024)).unwrap();

    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args(["hook", "check-large-files", "--enforce-all", "big.py"])
        .output()
        .expect("command failed");
    assert!(!output.status.success());
    assert!(
        output
            .stderr
            .to_str()
            .unwrap()
            .contains("Git check-attr failed")
    );
}

#[test]
fn test_large_files_invalid_config() {
    // if the nbwipers settings can't be constructed, the hook warns and
    // falls back to the on-disk size instead of the stripped size
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let git_init_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["init"])
        .output()
        .expect("git init failed");
    assert!(git_init_out.status.success());

    let nb = RawNotebook::default();
    {
        let f = fs::File::create(temp_dir.path().join("nb.ipynb")).unwrap();
        write_nb(BufWriter::new(f), &nb).unwrap();
    }
    fs::write(temp_dir.path().join("bad.toml"), "not = [valid toml").unwrap();

    let output = Command::new(&cur_exe)
        .current_dir(temp_dir.path())
        .args([
            "hook",
            "check-large-files",
            "--enforce-all",
            "-c",
            "bad.toml",
            "nb.ipynb",
        ])
        .output()
        .expect("command failed");
    assert!(output.status.success());
    assert!(
        output
            .stderr
            .to_str()
            .unwrap()
            .contains("Could not parse nb file")
    );
}

#[test]
fn test_invalid_stdin() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let mut check_output_cmd = Command::new(&cur_exe)
        .args(["check", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("command failed");
    {
        let mut check_in = check_output_cmd.stdin.take().expect("Failed to open stdin");
        check_in
            .write_all(b"Invalid input")
            .expect("Failed to write to stdin");
    }
    let check_output = check_output_cmd.wait_with_output().expect("Command failed");
    assert!(!check_output.status.success())
}

#[test]
fn test_show_config_exclude_patterns() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    fs::write(
        temp_dir.path().join(".nbwipers.toml"),
        "exclude = [\"vendored*.ipynb\"]\nextend-exclude = [\"scratch/\"]\n",
    )
    .unwrap();

    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["show-config"])
        .output()
        .expect("command failed");
    assert!(output.status.success());
    let stdout = output.stdout.to_str().unwrap();
    assert!(stdout.contains("vendored*.ipynb"));
    assert!(stdout.contains("scratch/"));
}

#[test]
fn test_clean_file_not_found() {
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let output = Command::new(cur_exe)
        .args(["clean", "no_such_notebook.ipynb"])
        .output()
        .expect("command failed");
    assert!(!output.status.success());
}

#[test]
// on Windows the confirmation prompt reads from the console device rather
// than stdin, so it hangs waiting for input even with stdin closed
#[cfg(not(windows))]
fn test_clean_all_non_interactive_confirmation() {
    // without --yes, clean-all asks for confirmation; when stdin is not a
    // terminal the prompt fails, and no notebook may be modified
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));

    let nb = RawNotebook {
        cells: vec![Cell::Code(CodeCell {
            source: SourceValue::StringArray(vec!["1 + 1".to_string()]),
            metadata: Value::Null,
            execution_count: Some(1),
            outputs: vec![],
            id: None,
        })],
        ..Default::default()
    };
    let nb_path = temp_dir.path().join("nb.ipynb");
    {
        let f = fs::File::create(&nb_path).unwrap();
        write_nb(BufWriter::new(f), &nb).unwrap();
    }
    let before = fs::read_to_string(&nb_path).unwrap();

    let output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["clean-all", "."])
        .stdin(Stdio::null())
        .output()
        .expect("command failed");
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&nb_path).unwrap(), before);
}
