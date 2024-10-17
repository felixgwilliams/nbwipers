use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use nbwipers::{
    record::{get_kernelspec_file, read_kernelspec_file, KernelSpecInfo},
    schema::RawNotebook,
    strip::write_nb,
};
use serde_json::json;

#[test]
fn test_blank_not_recorded() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let git_init_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["init"])
        .output()
        .expect("git init failed");
    assert!(git_init_out.status.success());
    let nb = RawNotebook::new();
    let nb_path = temp_dir.path().join("notebook.ipynb");
    write_nb(File::create(nb_path).unwrap(), &nb).unwrap();

    Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["record"])
        .output()
        .expect("record failed");
    let kernelspec_info = read_kernelspec_file(get_kernelspec_file(&temp_dir).unwrap())
        .unwrap()
        .unwrap();
    assert!(kernelspec_info.is_empty())
}

#[test]
fn test_metadata_recorded() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let git_init_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["init"])
        .output()
        .expect("git init failed");
    assert!(git_init_out.status.success());
    let python_version = "3.12.4".to_string();
    let kernelspec = json!({
        "name": "python3",
        "display_name": "Python 3"
    });
    let dirty_nb = RawNotebook {
        metadata: json!({
            "kernelspec": kernelspec,
            "language_info": {
                "name": "python",
                 "version": python_version
            }
        }),
        ..Default::default()
    };
    let nb_rel_path = String::from("notebook.ipynb");
    let nb_path = temp_dir.path().join(&nb_rel_path);
    // let mut nb_bytes = Vec::new();
    // write_nb(&mut nb_bytes, &nb).unwrap();
    write_nb(File::create(&nb_path).unwrap(), &dirty_nb).unwrap();

    Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["record"])
        .output()
        .expect("record failed");
    let kernelspec_info = read_kernelspec_file(get_kernelspec_file(&temp_dir).unwrap())
        .unwrap()
        .unwrap();
    assert!(!kernelspec_info.is_empty());
    assert_eq!(
        kernelspec_info.get(&nb_rel_path),
        Some(&KernelSpecInfo {
            kernelspec,
            python_version: Some(python_version)
        })
    );

    let strip_out = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["clean", &nb_rel_path, "--strip-kernel-info"])
        .output()
        .expect("clean failed");
    dbg!(strip_out.stderr);
    assert!(strip_out.status.success());

    let mut nb_bytes = vec![];
    fs::File::open(&nb_path)
        .unwrap()
        .read_to_end(&mut nb_bytes)
        .unwrap();
    assert!(strip_out.status.success());
    let mut check_smudge_output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["smudge", &nb_rel_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("smudge failed");
    {
        let mut check_in = check_smudge_output.stdin.take().unwrap();
        check_in.write_all(&nb_bytes).unwrap()
    }
    let smudge_out = check_smudge_output.wait_with_output().unwrap();

    // dbg!(String::from_utf8(smudge_out.stderr).unwrap());
    assert!(smudge_out.status.success());
    let clean_nb = serde_json::from_slice::<RawNotebook>(&nb_bytes).unwrap();
    assert_ne!(dirty_nb, clean_nb);
    let smudged_nb = serde_json::from_slice::<RawNotebook>(&smudge_out.stdout).unwrap();

    assert_eq!(dirty_nb, smudged_nb);
}

#[test]
fn test_invalid_git_dirs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let not_git_dir_out = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["record"])
        .output()
        .expect("record failed");
    assert!(!not_git_dir_out.status.success());
    let stderr = String::from_utf8(not_git_dir_out.stderr).unwrap();
    assert!(stderr.contains("Error: No .git dir"));

    let temp_dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp_dir.path().join(".git")).unwrap();

    let not_not_workdir = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["record"])
        .output()
        .expect("record failed");

    assert!(!not_not_workdir.status.success());
    let stderr = String::from_utf8(not_not_workdir.stderr).unwrap();
    dbg!(&stderr);
    assert!(stderr.contains("Error: Invalid git repo"));
}

#[test]
fn test_smudge_nothing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let git_init_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["init"])
        .output()
        .expect("git init failed");
    assert!(git_init_out.status.success());
    let verbatim_bytes = Vec::from(b"Hello world!");

    let mut check_smudge_output = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["smudge", "bananas.ipynb"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("smudge failed");
    {
        let mut check_in = check_smudge_output.stdin.take().unwrap();
        check_in.write_all(&verbatim_bytes).unwrap()
    }
    let smudge_out = check_smudge_output.wait_with_output().unwrap();
    assert_eq!(smudge_out.stdout, verbatim_bytes);
}

#[test]
fn record_clear_sync() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let git_init_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["init"])
        .output()
        .expect("git init failed");
    assert!(git_init_out.status.success());
    let python_version = "3.12.4".to_string();
    let kernelspec = json!({
        "name": "python3",
        "display_name": "Python 3"
    });
    let dirty_nb = RawNotebook {
        metadata: json!({
            "kernelspec": kernelspec,
            "language_info": {
                "name": "python",
                 "version": python_version
            }
        }),
        ..Default::default()
    };
    let nb1_rel = "nb1.ipynb";
    let nb2_rel = "nb2.ipynb";

    write_nb(
        File::create(temp_dir.path().join(nb1_rel)).unwrap(),
        &dirty_nb,
    )
    .unwrap();
    write_nb(
        File::create(temp_dir.path().join(nb2_rel)).unwrap(),
        &dirty_nb,
    )
    .unwrap();
    Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["record"])
        .output()
        .expect("record failed");
    let first_kernel_info = read_kernelspec_file(get_kernelspec_file(temp_dir.path()).unwrap())
        .unwrap()
        .unwrap();
    assert!(first_kernel_info.contains_key(nb1_rel));
    assert!(first_kernel_info.contains_key(nb2_rel));
    fs::remove_file(temp_dir.path().join(nb2_rel)).unwrap();

    Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["record", "--sync"])
        .output()
        .expect("record failed");
    let second_kernel_info = read_kernelspec_file(get_kernelspec_file(temp_dir.path()).unwrap())
        .unwrap()
        .unwrap();
    assert!(second_kernel_info.contains_key(nb1_rel));
    assert!(!second_kernel_info.contains_key(nb2_rel));

    Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["record", "--clear"])
        .output()
        .expect("record failed");
    let third_kernel_info = read_kernelspec_file(get_kernelspec_file(temp_dir.path()).unwrap())
        .unwrap()
        .unwrap();
    assert!(!third_kernel_info.contains_key(nb1_rel));
    assert!(!third_kernel_info.contains_key(nb2_rel));
}
#[test]
fn test_record_nothing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cur_exe = PathBuf::from(env!("CARGO_BIN_EXE_nbwipers"));
    let git_init_out = Command::new("git")
        .current_dir(&temp_dir)
        .args(["init"])
        .output()
        .expect("git init failed");
    assert!(git_init_out.status.success());
    let out = Command::new(&cur_exe)
        .current_dir(&temp_dir)
        .args(["record"])
        .output()
        .expect("record failed");
    assert!(!out.status.success());
}
