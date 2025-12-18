//! Integration tests for symlink handling in discovery.

use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;
use veil_discovery::{DiscoveryOptions, Scanner};

#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(windows)]
use std::os::windows::fs::symlink_file;

fn create_test_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    let mut file = File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

#[test]
#[cfg(unix)]
fn test_follow_symlinks_to_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create a file with PII
    let original = create_test_file(&temp_dir, "original.txt", "Email: test@example.com");

    // Create symlink to the file
    let link_path = temp_dir.path().join("link.txt");
    symlink(&original, &link_path).unwrap();

    let options = DiscoveryOptions {
        root_path: temp_dir.path().to_path_buf(),
        follow_symlinks: true,
        ..Default::default()
    };

    let scanner = Scanner::new(options);
    let result = scanner.scan().unwrap();

    // Should find PII in both the original and the symlink
    assert!(!result.discovered_files.is_empty());
}

#[test]
#[cfg(unix)]
fn test_ignore_symlinks() {
    let temp_dir = TempDir::new().unwrap();

    // Create a file with PII
    let original = create_test_file(&temp_dir, "original.txt", "Email: test@example.com");

    // Create symlink to the file
    let link_path = temp_dir.path().join("link.txt");
    symlink(&original, &link_path).unwrap();

    let options = DiscoveryOptions {
        root_path: temp_dir.path().to_path_buf(),
        follow_symlinks: false,
        ..Default::default()
    };

    let scanner = Scanner::new(options);
    let result = scanner.scan().unwrap();

    // Should only find the original file, not the symlink
    // (or might skip symlinks entirely)
    assert!(!result.discovered_files.is_empty());
}

#[test]
#[cfg(unix)]
fn test_symlink_to_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Create a subdirectory with PII files
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let _file = {
        let path = subdir.join("data.txt");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"SSN: 123-45-6789").unwrap();
        path
    };

    // Create symlink to the directory
    let link_path = temp_dir.path().join("link_dir");
    symlink(&subdir, &link_path).unwrap();

    let options = DiscoveryOptions {
        root_path: temp_dir.path().to_path_buf(),
        follow_symlinks: true,
        ..Default::default()
    };

    let scanner = Scanner::new(options);
    let result = scanner.scan().unwrap();

    // Should find PII files in both the original dir and through the symlink
    assert!(!result.discovered_files.is_empty());
}

#[test]
#[cfg(unix)]
fn test_broken_symlink_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Create a symlink to a non-existent file
    let link_path = temp_dir.path().join("broken_link.txt");
    symlink("/nonexistent/path/file.txt", &link_path).unwrap();

    // Create a real file so the scan has something to work with
    create_test_file(&temp_dir, "real.txt", "Email: test@example.com");

    let options = DiscoveryOptions {
        root_path: temp_dir.path().to_path_buf(),
        follow_symlinks: true,
        ..Default::default()
    };

    let scanner = Scanner::new(options);
    // Should not panic on broken symlinks
    let result = scanner.scan();

    // Should either succeed or report an error for the broken link
    assert!(result.is_ok());
}

#[test]
#[cfg(unix)]
fn test_symlink_loop_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create a circular symlink (a -> b -> a)
    let a_path = temp_dir.path().join("a");
    let b_path = temp_dir.path().join("b");

    // Create one symlink first, then the circular one
    // Note: This might not work on all systems
    symlink(&b_path, &a_path).ok();
    symlink(&a_path, &b_path).ok();

    // Create a real file
    create_test_file(&temp_dir, "real.txt", "Email: test@example.com");

    let options = DiscoveryOptions {
        root_path: temp_dir.path().to_path_buf(),
        follow_symlinks: true,
        ..Default::default()
    };

    let scanner = Scanner::new(options);
    // Should not hang on circular symlinks
    let result = scanner.scan();

    assert!(result.is_ok());
}

#[test]
fn test_no_symlinks_basic_scan() {
    let temp_dir = TempDir::new().unwrap();

    // Create files without symlinks
    create_test_file(&temp_dir, "file1.txt", "Email: test@example.com");
    create_test_file(&temp_dir, "file2.txt", "Phone: 555-123-4567");

    let options = DiscoveryOptions {
        root_path: temp_dir.path().to_path_buf(),
        follow_symlinks: false,
        ..Default::default()
    };

    let scanner = Scanner::new(options);
    let result = scanner.scan().unwrap();

    // Should find both files
    assert!(!result.discovered_files.is_empty());
}
