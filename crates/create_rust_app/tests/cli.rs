//! CLI integration tests: drive the built binary end to end.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_create-rust-app"))
}

fn fixture_catalog() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/catalog/templates.json")
        .to_string_lossy()
        .to_string()
}

#[test]
fn version_flag_reports_package_version() {
    let output = Command::new(binary())
        .arg("--version")
        .output()
        .expect("run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("create-rust-app"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn lists_templates_from_local_catalog() {
    let output = Command::new(binary())
        .args(["--list-templates", "--catalog-path"])
        .arg(fixture_catalog())
        .output()
        .expect("run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("web-server"));
    assert!(stdout.contains("cli"));
}

#[test]
fn lists_templates_as_json() {
    let output = Command::new(binary())
        .args(["--list-templates", "--json", "--catalog-path"])
        .arg(fixture_catalog())
        .output()
        .expect("run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Vec<String> = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(parsed, vec!["web-server".to_string(), "cli".to_string()]);
}

#[test]
fn filters_templates_by_category() {
    let output = Command::new(binary())
        .args(["--list-templates", "--category", "web", "--catalog-path"])
        .arg(fixture_catalog())
        .output()
        .expect("run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("web-server"));
    assert!(!stdout.contains("- cli"));
}

#[test]
fn rejects_unknown_category() {
    let output = Command::new(binary())
        .args(["--list-templates", "--category", "nope", "--catalog-path"])
        .arg(fixture_catalog())
        .output()
        .expect("run binary");
    assert!(!output.status.success());
}

#[test]
fn prints_bash_completion() {
    let output = Command::new(binary())
        .args(["--add-completion", "bash"])
        .output()
        .expect("run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("create-rust-app"));
}

#[test]
fn rejects_unknown_completion_shell() {
    let output = Command::new(binary())
        .args(["--add-completion", "powershell-legacy"])
        .output()
        .expect("run binary");
    assert!(!output.status.success());
}

#[test]
fn info_flag_reports_environment() {
    let output = Command::new(binary())
        .arg("--info")
        .output()
        .expect("run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("create-rust-app"));
}

#[test]
fn scaffolds_project_headless() {
    let dir = tempfile::tempdir().expect("tempdir");
    let project = dir.path().join("my-api");
    let output = Command::new(binary())
        .current_dir(dir.path())
        .args([
            "my-api",
            "--template",
            "web-server",
            "--addons",
            "github-setup",
            "--no-interactive",
            "--no-install",
            "--catalog-path",
        ])
        .arg(fixture_catalog())
        .output()
        .expect("run binary");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(project.join("Cargo.toml").exists());
    assert!(project.join("src/main.rs").exists());
    assert!(project.join("cra.config.json").exists());
}

#[test]
fn rejects_invalid_set_override() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = Command::new(binary())
        .current_dir(dir.path())
        .args([
            "my-api",
            "--template",
            "web-server",
            "--set",
            "novalue",
            "--no-interactive",
            "--no-install",
            "--catalog-path",
        ])
        .arg(fixture_catalog())
        .output()
        .expect("run binary");
    assert!(!output.status.success());
}

#[test]
fn cache_status_reports_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = Command::new(binary())
        .env("CRA_CACHE_DIR", dir.path().join("cache"))
        .args(["cache", "status"])
        .output()
        .expect("run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cache"));
}

#[test]
fn cache_clean_removes_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cache = dir.path().join("cache");
    fs::create_dir_all(&cache).expect("mkdir");
    let output = Command::new(binary())
        .env("CRA_CACHE_DIR", &cache)
        .args(["cache", "clean"])
        .output()
        .expect("run binary");
    assert!(output.status.success());
    assert!(!cache.exists());
}
