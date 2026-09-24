use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn binary_path() -> String {
    let output = Command::new("cargo")
        .args(["build", "--bin", "compare-json"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to build");
    assert!(output.status.success(), "Build failed: {}", String::from_utf8_lossy(&output.stderr));

    let path = format!("{}/target/debug/compare-json", env!("CARGO_MANIFEST_DIR"));
    path
}

fn run_cli(args: &[&str]) -> (String, String, bool) {
    let bin = binary_path();
    let output = Command::new(&bin)
        .args(args)
        .output()
        .expect("Failed to execute");
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    )
}

#[test]
fn test_cli_version() {
    let (stdout, _, success) = run_cli(&["--version"]);
    assert!(success);
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_cli_compare_json_strings() {
    let (stdout, _, success) = run_cli(&[r#"{"a":1}"#, r#"{"a":2}"#]);
    assert!(success);
    assert!(stdout.contains("valueChanged"));
    assert!(stdout.contains("(Base) a"));
}

#[test]
fn test_cli_compare_json_files() {
    let dir = TempDir::new().unwrap();
    let f1 = dir.path().join("base.json");
    let f2 = dir.path().join("contrast.json");
    fs::write(&f1, r#"{"a":1,"b":2}"#).unwrap();
    fs::write(&f2, r#"{"a":2,"c":3}"#).unwrap();

    let (stdout, _, success) = run_cli(&[f1.to_str().unwrap(), f2.to_str().unwrap()]);
    assert!(success);
    assert!(stdout.contains("valueChanged"));
    assert!(stdout.contains("deleted"));
    assert!(stdout.contains("added"));
}

#[test]
fn test_cli_json_export() {
    let (stdout, _, success) = run_cli(&["-j", r#"{"a":1}"#, r#"{"a":2}"#]);
    assert!(success);
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["diffType"], "valueChanged");
    assert_eq!(parsed[0]["pathBelongsTo"], "both");
}

#[test]
fn test_cli_output_to_file() {
    let dir = TempDir::new().unwrap();
    let out_file = dir.path().join("output.txt");

    let (stdout, _, success) = run_cli(&["-o", out_file.to_str().unwrap(), r#"{"a":1}"#, r#"{"a":2}"#]);
    assert!(success);
    assert!(stdout.contains("Output written to"));

    let content = fs::read_to_string(&out_file).unwrap();
    assert!(content.contains("valueChanged"));
}

#[test]
fn test_cli_array_compare_unordered() {
    let (stdout, _, success) = run_cli(&["-a", "unordered", "[1,2,3]", "[3,2,1]"]);
    assert!(success);
    assert!(stdout.contains("No differences found"));
}

#[test]
fn test_cli_key_case_insensitive() {
    let (stdout, _, success) = run_cli(&["-k", r#"{"Name":"Alice"}"#, r#"{"name":"Alice"}"#]);
    assert!(success);
    assert!(stdout.contains("No differences found"));
}

#[test]
fn test_cli_value_case_insensitive() {
    let (stdout, _, success) = run_cli(&["-v", r#"{"status":"OK"}"#, r#"{"status":"ok"}"#]);
    assert!(success);
    assert!(stdout.contains("No differences found"));
}

#[test]
fn test_cli_numeric_string_equals_number() {
    let (stdout, _, success) = run_cli(&["--numeric-string-equals-number", r#"{"count":1}"#, r#"{"count":"1"}"#]);
    assert!(success);
    assert!(stdout.contains("No differences found"));
}

#[test]
fn test_cli_invalid_json() {
    let (_, _, success) = run_cli(&["{invalid}", r#"{"a":1}"#]);
    assert!(!success);
}

#[test]
fn test_cli_root_level_difference() {
    let (stdout, _, success) = run_cli(&["1", r#""hello""#]);
    assert!(success);
    assert!(stdout.contains("(Root)"));
    assert!(stdout.contains("typeChanged"));
}

#[test]
fn test_cli_no_differences() {
    let (stdout, _, success) = run_cli(&[r#"{"a":1}"#, r#"{"a":1}"#]);
    assert!(success);
    assert!(stdout.contains("No differences found"));
}

#[test]
fn test_cli_json_export_with_output_file() {
    let dir = TempDir::new().unwrap();
    let out_file = dir.path().join("output.json");

    let (_, _, success) = run_cli(&["-j", "-o", out_file.to_str().unwrap(), r#"{"a":1,"b":2}"#, r#"{"a":2,"c":3}"#]);
    assert!(success);

    let content = fs::read_to_string(&out_file).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[0]["diffType"], "valueChanged");
    assert_eq!(parsed[1]["diffType"], "deleted");
    assert_eq!(parsed[1]["pathBelongsTo"], "base");
    assert_eq!(parsed[2]["diffType"], "added");
    assert_eq!(parsed[2]["pathBelongsTo"], "contrast");
}
