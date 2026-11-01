use std::io::Write;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::write::SimpleFileOptions;

fn unique() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

#[test]
fn test_new_scaffolds_content_mod() {
    let tmp = std::env::temp_dir().join(format!("hmt_new_{}", unique()));
    let proj = tmp.join("my_mod");
    let status = Command::new(env!("CARGO_BIN_EXE_horizon-mod-toolkit"))
        .args(["new", proj.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success(), "new should succeed");

    assert!(proj.join("horizon-mod.toml").exists());
    let content = std::fs::read_to_string(proj.join("horizon-mod.toml")).unwrap();
    assert!(content.contains("id = \"my_mod\""), "manifest should contain id");
    assert!(content.contains("capabilities = [\"registries\"]"));

    assert!(proj.join("assets").join("my_mod").is_dir());
    assert!(proj.join("data").join("my_mod").is_dir());
    assert!(proj.join("assets").join("my_mod").join(".gitkeep").exists());
    assert!(proj.join("data").join("my_mod").join(".gitkeep").exists());
    assert!(proj.join("README.md").exists());

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_new_invalid_id_errors() {
    let tmp = std::env::temp_dir().join(format!("hmt_newbad_{}", unique()));
    let proj = tmp.join("bad");
    let status = Command::new(env!("CARGO_BIN_EXE_horizon-mod-toolkit"))
        .args(["new", proj.to_str().unwrap(), "--id", "Bad-ID"])
        .status()
        .unwrap();
    assert!(!status.success());
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_doctor_runs() {
    let status = Command::new(env!("CARGO_BIN_EXE_horizon-mod-toolkit"))
        .arg("doctor")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_help_and_version() {
    let s1 = Command::new(env!("CARGO_BIN_EXE_horizon-mod-toolkit"))
        .arg("--help")
        .status()
        .unwrap();
    assert!(s1.success());
    let s2 = Command::new(env!("CARGO_BIN_EXE_horizon-mod-toolkit"))
        .arg("--version")
        .status()
        .unwrap();
    assert!(s2.success());
}

#[test]
fn test_inspect_missing_file_exit_2() {
    let status = Command::new(env!("CARGO_BIN_EXE_horizon-mod-toolkit"))
        .args(["inspect", "C:\\definitely\\not\\here.hmod"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(2));
}

#[test]
fn test_inspect_valid_content_mod_exit_0() {
    let dir = std::env::temp_dir().join(format!("hmt_inspect_{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.hmod");
    let file = std::fs::File::create(&path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default();
    let manifest = r#"
id = "ruby_gems"
name = "Ruby Gems"
version = "1.0.0"
description = "Adds rare ruby minerals"
authors = ["CGRLabs"]
api = "0.2"
capabilities = ["registries"]
"#;
    zip.start_file("horizon-mod.toml", options).unwrap();
    zip.write_all(manifest.as_bytes()).unwrap();
    zip.finish().unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_horizon-mod-toolkit"))
        .args(["inspect", path.to_str().unwrap(), "--json"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(0));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_unknown_command_exit_2() {
    let status = Command::new(env!("CARGO_BIN_EXE_horizon-mod-toolkit"))
        .arg("frobnicate")
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(2));
}
