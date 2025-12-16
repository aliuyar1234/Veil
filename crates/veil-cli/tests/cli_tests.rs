//! CLI integration tests for the Veil tool.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Get the CLI binary.
fn veil_cmd() -> Command {
    Command::cargo_bin("veil").unwrap()
}

// ============== Help & Version Tests ==============

#[test]
fn test_help() {
    veil_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Detect and redact personally identifiable information"))
        .stdout(predicate::str::contains("scan"))
        .stdout(predicate::str::contains("protect"))
        .stdout(predicate::str::contains("batch"));
}

#[test]
fn test_version() {
    veil_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("veil"));
}

#[test]
fn test_scan_help() {
    veil_cmd()
        .args(["scan", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Scan files for PII"));
}

#[test]
fn test_protect_help() {
    veil_cmd()
        .args(["protect", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Protect files by redacting PII"));
}

#[test]
fn test_batch_help() {
    veil_cmd()
        .args(["batch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Batch process multiple files"));
}

// ============== Scan Command Tests ==============

#[test]
fn test_scan_text_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Contact: test@example.com").unwrap();

    veil_cmd()
        .args(["scan", file_path.to_str().unwrap()])
        .assert()
        .success()
        // Should find at least one finding
        .stdout(predicate::str::contains("finding"));
}

#[test]
fn test_scan_file_with_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Call me at 555-123-4567").unwrap();

    veil_cmd()
        .args(["--json", "scan", file_path.to_str().unwrap()])
        .assert()
        .success()
        // JSON output should be parseable
        .stdout(predicate::str::contains("{"));
}

#[test]
fn test_scan_file_no_pii() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Hello world! This has no PII.").unwrap();

    veil_cmd()
        .args(["scan", file_path.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_scan_nonexistent_file() {
    // CLI gracefully handles nonexistent files by reporting 0 findings
    veil_cmd()
        .args(["scan", "/nonexistent/file.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 findings"));
}

#[test]
fn test_scan_csv_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("data.csv");
    fs::write(&file_path, "name,email\nJohn,john@example.com\n").unwrap();

    veil_cmd()
        .args(["scan", file_path.to_str().unwrap()])
        .assert()
        .success()
        // Should find at least one finding in the CSV
        .stdout(predicate::str::contains("finding"));
}

#[test]
fn test_scan_json_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("data.json");
    fs::write(&file_path, r#"{"user": {"email": "test@example.com"}}"#).unwrap();

    veil_cmd()
        .args(["scan", file_path.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_scan_multiple_files() {
    let temp_dir = TempDir::new().unwrap();

    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    fs::write(&file1, "Email: user1@example.com").unwrap();
    fs::write(&file2, "Email: user2@example.com").unwrap();

    veil_cmd()
        .args([
            "scan",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[test]
fn test_scan_quiet_mode() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Email: test@example.com").unwrap();

    veil_cmd()
        .args(["--quiet", "scan", file_path.to_str().unwrap()])
        .assert()
        .success();
}

// ============== Protect Command Tests ==============

#[test]
fn test_protect_text_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.txt");
    let output_path = temp_dir.path().join("output.txt");

    fs::write(&input_path, "Contact: test@example.com").unwrap();

    veil_cmd()
        .args([
            "protect",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Should be redacted
    assert!(!output.contains("test@example.com"));
}

#[test]
fn test_protect_with_label_style() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.txt");
    let output_path = temp_dir.path().join("output.txt");

    fs::write(&input_path, "Contact: test@example.com for info").unwrap();

    veil_cmd()
        .args([
            "protect",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
            "--style",
            "label",
        ])
        .assert()
        .success();

    // Output file should exist
    assert!(output_path.exists());
    let output = fs::read_to_string(&output_path).unwrap();
    // Email should be redacted
    assert!(!output.contains("test@example.com"));
}

#[test]
fn test_protect_nonexistent_file() {
    veil_cmd()
        .args(["protect", "/nonexistent/file.txt"])
        .assert()
        .failure();
}

// ============== Crypto Command Tests ==============

#[test]
fn test_crypto_keygen() {
    let temp_dir = TempDir::new().unwrap();
    let key_path = temp_dir.path().join("test.key");

    veil_cmd()
        .args(["crypto", "keygen", "-o", key_path.to_str().unwrap()])
        .assert()
        .success();

    // Key file should exist and have content
    assert!(key_path.exists());
    let key_content = fs::read(&key_path).unwrap();
    assert!(!key_content.is_empty());
}

#[test]
fn test_crypto_encrypt_decrypt() {
    let temp_dir = TempDir::new().unwrap();
    let key_path = temp_dir.path().join("test.key");
    let encrypted_path = temp_dir.path().join("encrypted.txt");
    let decrypted_path = temp_dir.path().join("decrypted.txt");
    let input_path = temp_dir.path().join("input.txt");

    // Generate key
    veil_cmd()
        .args(["crypto", "keygen", "-o", key_path.to_str().unwrap()])
        .assert()
        .success();

    // Write test input
    fs::write(&input_path, "Secret message").unwrap();

    // Encrypt
    veil_cmd()
        .args([
            "crypto",
            "encrypt",
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            encrypted_path.to_str().unwrap(),
            "-k",
            key_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Encrypted file should exist and be different from input
    assert!(encrypted_path.exists());
    let encrypted = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted.trim(), "Secret message");

    // Decrypt
    veil_cmd()
        .args([
            "crypto",
            "decrypt",
            "-i",
            encrypted_path.to_str().unwrap(),
            "-o",
            decrypted_path.to_str().unwrap(),
            "-k",
            key_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Decrypted should match original
    let decrypted = fs::read_to_string(&decrypted_path).unwrap();
    assert_eq!(decrypted.trim(), "Secret message");
}

#[test]
fn test_crypto_hash() {
    veil_cmd()
        .args(["crypto", "hash", "test-input"])
        .assert()
        .success()
        .stdout(predicate::str::is_match("[a-f0-9]{64}").unwrap());
}

// ============== Policy Command Tests ==============

#[test]
fn test_policy_init() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yaml");

    veil_cmd()
        .args(["policy", "init", "-o", policy_path.to_str().unwrap()])
        .assert()
        .success();

    // Policy file should exist and contain expected content
    assert!(policy_path.exists());
    let content = fs::read_to_string(&policy_path).unwrap();
    assert!(content.contains("name:") || content.contains("rules:"));
}

#[test]
fn test_policy_validate_valid() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yaml");

    // Create a valid policy
    fs::write(
        &policy_path,
        r#"
name: test-policy
version: "1.0"
rules:
  - name: email
    pattern: email
    action: redact
"#,
    )
    .unwrap();

    veil_cmd()
        .args(["policy", "validate", policy_path.to_str().unwrap()])
        .assert()
        .success();
}

// ============== Stream Command Tests ==============

#[test]
fn test_stream_from_stdin() {
    veil_cmd()
        .args(["stream"])
        .write_stdin("Contact: test@example.com\n")
        .assert()
        .success();
}

// ============== Error Handling Tests ==============

#[test]
fn test_invalid_command() {
    veil_cmd()
        .arg("invalid-command")
        .assert()
        .failure();
}

#[test]
fn test_scan_missing_path() {
    veil_cmd()
        .arg("scan")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_protect_missing_input() {
    veil_cmd()
        .arg("protect")
        .assert()
        .failure();
}
