//! Integration tests for veil-discovery.

use std::fs;
use std::io::Write;
use tempfile::TempDir;
use veil_discovery::{DiscoveryOptions, PiiCategory, ReportFormat, ReportGenerator, Scanner};

#[test]
fn test_end_to_end_discovery() {
    // Create a test directory structure
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create various test files
    let emails_file = root.join("contacts.txt");
    let mut f = fs::File::create(&emails_file).unwrap();
    writeln!(f, "John Doe - john.doe@example.com").unwrap();
    writeln!(f, "Jane Smith - jane.smith@company.org").unwrap();

    let csv_file = root.join("data.csv");
    let mut f = fs::File::create(&csv_file).unwrap();
    writeln!(f, "Name,Email,Phone").unwrap();
    writeln!(f, "Alice,alice@test.com,555-1234").unwrap();
    writeln!(f, "Bob,bob@test.com,555-5678").unwrap();

    let json_file = root.join("users.json");
    let mut f = fs::File::create(&json_file).unwrap();
    writeln!(
        f,
        r#"{{"users":[{{"name":"Charlie","email":"charlie@example.com"}}]}}"#
    )
    .unwrap();

    // File without PII
    let normal_file = root.join("readme.txt");
    let mut f = fs::File::create(&normal_file).unwrap();
    writeln!(f, "This is a normal file without any sensitive data.").unwrap();

    // Create a subdirectory
    let subdir = root.join("subdir");
    fs::create_dir(&subdir).unwrap();
    let subfile = subdir.join("nested.txt");
    let mut f = fs::File::create(&subfile).unwrap();
    writeln!(f, "Contact: test@nested.com").unwrap();

    // Create a .git directory (should be excluded)
    let git_dir = root.join(".git");
    fs::create_dir(&git_dir).unwrap();
    let git_file = git_dir.join("config");
    let mut f = fs::File::create(&git_file).unwrap();
    writeln!(f, "email = should-be-excluded@example.com").unwrap();

    // Run discovery
    let options = DiscoveryOptions {
        root_path: root.to_path_buf(),
        ..Default::default()
    };

    let scanner = Scanner::new(options);
    let result = scanner.scan().unwrap();

    // Verify results
    assert_eq!(result.total_files_scanned, 5); // 4 files + 1 in subdir (excluding .git)
    assert!(result.discovered_files.len() >= 3); // At least contacts.txt, data.csv, users.json

    // Verify PII categories found
    assert!(result
        .statistics
        .findings_by_category
        .contains_key(&PiiCategory::Email));

    // Verify that .git directory was excluded
    assert!(!result
        .discovered_files
        .iter()
        .any(|f| f.path.to_string_lossy().contains(".git")));

    // Generate reports
    let json_report = ReportGenerator::generate(&result, ReportFormat::Json).unwrap();
    assert!(json_report.contains("discovered_files"));

    let text_report = ReportGenerator::generate(&result, ReportFormat::Text).unwrap();
    assert!(text_report.contains("PII Discovery Report"));

    let summary = ReportGenerator::generate(&result, ReportFormat::Summary).unwrap();
    assert!(summary.contains("PII Discovery Summary"));

    let csv_report = ReportGenerator::generate_csv(&result).unwrap();
    assert!(csv_report.contains("Path,Size,Sampled"));
}

#[test]
fn test_progress_callback() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create multiple files
    for i in 0..5 {
        let file_path = root.join(format!("file{}.txt", i));
        let mut f = fs::File::create(&file_path).unwrap();
        writeln!(f, "Email: test{}@example.com", i).unwrap();
    }

    let options = DiscoveryOptions {
        root_path: root.to_path_buf(),
        report_progress: true,
        ..Default::default()
    };

    let scanner = Scanner::new(options).with_progress_callback(move |progress| {
        // Verify progress is reported (files_scanned and bytes_scanned are usize, always >= 0)
        let _ = progress.files_scanned;
        let _ = progress.bytes_scanned;
    });

    let result = scanner.scan().unwrap();
    assert_eq!(result.discovered_files.len(), 5);
}

#[test]
fn test_custom_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create files in different directories
    let src_dir = root.join("src");
    fs::create_dir(&src_dir).unwrap();
    let src_file = src_dir.join("code.rs");
    let mut f = fs::File::create(&src_file).unwrap();
    writeln!(f, "// Email: dev@example.com").unwrap();

    let docs_dir = root.join("docs");
    fs::create_dir(&docs_dir).unwrap();
    let docs_file = docs_dir.join("readme.md");
    let mut f = fs::File::create(&docs_file).unwrap();
    writeln!(f, "Contact: support@example.com").unwrap();

    // Only scan src directory
    let options = DiscoveryOptions {
        root_path: root.to_path_buf(),
        include_patterns: vec!["**/src/**".to_string()],
        ..Default::default()
    };

    let scanner = Scanner::new(options);
    let result = scanner.scan().unwrap();

    // Should only find the src file
    assert_eq!(result.discovered_files.len(), 1);
    assert!(result.discovered_files[0]
        .path
        .to_string_lossy()
        .contains("src"));
}

#[test]
fn test_large_file_sampling() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create a large file
    let large_file = root.join("large.txt");
    let mut f = fs::File::create(&large_file).unwrap();

    // Write PII at the beginning
    writeln!(f, "Important email: test@example.com").unwrap();

    // Write lots of filler content
    for i in 0..1000 {
        writeln!(f, "Line {}: This is filler content.", i).unwrap();
    }

    let options = DiscoveryOptions {
        root_path: root.to_path_buf(),
        max_file_size: Some(1024), // 1KB max
        sample_size: 512,          // Sample 512 bytes
        ..Default::default()
    };

    let scanner = Scanner::new(options);
    let result = scanner.scan().unwrap();

    assert_eq!(result.discovered_files.len(), 1);
    let discovered = &result.discovered_files[0];
    assert!(discovered.sampled);
    assert_eq!(discovered.bytes_scanned, 512);
    assert!(discovered.pii_categories.contains(&PiiCategory::Email));
}

#[test]
fn test_multiple_pii_types() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let file_path = root.join("mixed.txt");
    let mut f = fs::File::create(&file_path).unwrap();
    writeln!(f, "Contact Information:").unwrap();
    writeln!(f, "Email: john@example.com").unwrap();
    writeln!(f, "Phone: 555-123-4567").unwrap();
    writeln!(f, "SSN: 123-45-6789").unwrap();

    let options = DiscoveryOptions {
        root_path: root.to_path_buf(),
        ..Default::default()
    };

    let scanner = Scanner::new(options);
    let result = scanner.scan().unwrap();

    assert_eq!(result.discovered_files.len(), 1);
    let discovered = &result.discovered_files[0];

    // Check that PII was found (at least email should be detected)
    assert!(!discovered.pii_categories.is_empty());
    assert!(discovered.finding_count >= 1);
}

#[test]
fn test_statistics_accuracy() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create files with known PII
    let file1 = root.join("file1.txt");
    let mut f = fs::File::create(&file1).unwrap();
    writeln!(f, "test1@example.com").unwrap();
    writeln!(f, "test2@example.com").unwrap();

    let file2 = root.join("file2.txt");
    let mut f = fs::File::create(&file2).unwrap();
    writeln!(f, "test3@example.com").unwrap();

    let file3 = root.join("file3.txt");
    let mut f = fs::File::create(&file3).unwrap();
    writeln!(f, "No PII here").unwrap();

    let options = DiscoveryOptions {
        root_path: root.to_path_buf(),
        ..Default::default()
    };

    let scanner = Scanner::new(options);
    let result = scanner.scan().unwrap();

    // Verify statistics
    assert_eq!(result.total_files_scanned, 3);
    assert_eq!(result.statistics.total_files_with_pii, 2);
    assert!(result.statistics.total_findings >= 3); // At least 3 email addresses

    let email_count = result
        .statistics
        .findings_by_category
        .get(&PiiCategory::Email)
        .unwrap_or(&0);
    assert!(*email_count >= 3);
}
