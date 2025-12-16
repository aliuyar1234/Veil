use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use tempfile::TempDir;
use veil_batch::{BatchJob, BatchOptions, BatchProcessor, DefaultBatchProcessor};

fn create_test_files(dir: &TempDir) -> PathBuf {
    let base = dir.path().to_path_buf();

    // Create text file with PII
    let txt_path = base.join("test.txt");
    let mut file = File::create(&txt_path).unwrap();
    writeln!(file, "Contact: john.doe@example.com").unwrap();
    writeln!(file, "Phone: 555-123-4567").unwrap();

    // Create CSV file
    let csv_path = base.join("data.csv");
    let mut file = File::create(&csv_path).unwrap();
    writeln!(file, "Name,Email,Phone").unwrap();
    writeln!(file, "Alice,alice@example.com,555-111-2222").unwrap();
    writeln!(file, "Bob,bob@example.org,555-333-4444").unwrap();

    // Create subdirectory with more files
    let subdir = base.join("subdir");
    fs::create_dir(&subdir).unwrap();

    let sub_txt = subdir.join("nested.txt");
    let mut file = File::create(&sub_txt).unwrap();
    writeln!(file, "SSN: 123-45-6789").unwrap();

    base
}

#[test]
fn test_empty_batch() {
    let processor = DefaultBatchProcessor::new();
    let job = BatchJob::new(vec![], BatchOptions::default());

    let result = processor.process(&job);
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.summary.total_files, 0);
    assert_eq!(batch_result.summary.successful, 0);
    assert_eq!(batch_result.summary.failed, 0);
    assert_eq!(batch_result.summary.skipped, 0);
}

#[test]
fn test_nonexistent_path() {
    let processor = DefaultBatchProcessor::new();
    let sources = vec![std::path::PathBuf::from("nonexistent_directory_12345")];
    let job = BatchJob::new(sources, BatchOptions::default());

    let result = processor.process(&job);
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    // Should handle gracefully with no files processed
    assert_eq!(batch_result.summary.total_files, 0);
}

#[test]
fn test_batch_options_defaults() {
    let options = BatchOptions::default();
    assert!(options.follow_symlinks);
    assert!(options.recursive);
    assert_eq!(options.max_archive_depth, 5);
    assert_eq!(options.max_file_size, 100 * 1024 * 1024);
    assert!(options.include_patterns.is_empty());
    assert!(options.exclude_patterns.is_empty());
}

#[test]
fn test_batch_process_directory() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = create_test_files(&temp_dir);

    let processor = DefaultBatchProcessor::new();
    let job = BatchJob::new(vec![base_path], BatchOptions::default());

    let result = processor.process(&job);
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    // Should process at least the text and csv files
    assert!(
        batch_result.summary.successful >= 2,
        "Expected at least 2 successful files, got {}",
        batch_result.summary.successful
    );
}

#[test]
fn test_batch_recursive_processing() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = create_test_files(&temp_dir);

    let options = BatchOptions {
        recursive: true,
        ..Default::default()
    };

    let processor = DefaultBatchProcessor::new();
    let job = BatchJob::new(vec![base_path], options);

    let result = processor.process(&job);
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    // Should process files in subdirectory too
    assert!(
        batch_result.summary.successful >= 3,
        "Expected at least 3 successful files (including nested), got {}",
        batch_result.summary.successful
    );
}

#[test]
fn test_batch_non_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = create_test_files(&temp_dir);

    let options = BatchOptions {
        recursive: false,
        ..Default::default()
    };

    let processor = DefaultBatchProcessor::new();
    let job = BatchJob::new(vec![base_path], options);

    let result = processor.process(&job);
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    // Should only process files in root directory
    assert!(
        batch_result.summary.successful <= 2,
        "Expected at most 2 files when non-recursive, got {}",
        batch_result.summary.successful
    );
}

#[test]
fn test_batch_include_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = create_test_files(&temp_dir);

    let options = BatchOptions {
        include_patterns: vec!["*.txt".to_string()],
        ..Default::default()
    };

    let processor = DefaultBatchProcessor::new();
    let job = BatchJob::new(vec![base_path], options);

    let result = processor.process(&job);
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    // Should only process .txt files
    for file_result in &batch_result.results {
        let ext = file_result.path.extension().and_then(|e| e.to_str());
        assert_eq!(ext, Some("txt"), "Expected only .txt files");
    }
}

#[test]
fn test_batch_exclude_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = create_test_files(&temp_dir);

    let options = BatchOptions {
        exclude_patterns: vec!["*.csv".to_string()],
        ..Default::default()
    };

    let processor = DefaultBatchProcessor::new();
    let job = BatchJob::new(vec![base_path], options);

    let result = processor.process(&job);
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    // Should not process .csv files
    for file_result in &batch_result.results {
        let ext = file_result.path.extension().and_then(|e| e.to_str());
        assert_ne!(ext, Some("csv"), "Should not include .csv files");
    }
}

#[test]
fn test_batch_max_file_size() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();

    // Create small file
    let small_path = base_path.join("small.txt");
    let mut file = File::create(&small_path).unwrap();
    writeln!(file, "Small file content").unwrap();
    drop(file); // Ensure file is flushed

    // Create large file (over limit)
    let large_path = base_path.join("large.txt");
    let mut file = File::create(&large_path).unwrap();
    for _ in 0..1000 {
        writeln!(file, "Large file content line").unwrap();
    }
    drop(file); // Ensure file is flushed

    let options = BatchOptions {
        max_file_size: 100, // Very small limit
        ..Default::default()
    };

    let processor = DefaultBatchProcessor::new();
    let job = BatchJob::new(vec![base_path], options);

    let result = processor.process(&job);
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    // Large file should be skipped
    assert!(
        batch_result.summary.skipped >= 1,
        "Expected at least 1 skipped file due to size limit"
    );
}

#[test]
fn test_batch_extracts_segments() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = create_test_files(&temp_dir);

    let processor = DefaultBatchProcessor::new();
    let job = BatchJob::new(vec![base_path], BatchOptions::default());

    let result = processor.process(&job);
    assert!(result.is_ok());

    let batch_result = result.unwrap();

    // Check that segments were extracted
    let has_segments = batch_result
        .results
        .iter()
        .any(|r| !r.result.segments.is_empty());
    assert!(has_segments, "Expected segments to be extracted from files");

    // Check total chars
    let total_chars: usize = batch_result
        .results
        .iter()
        .map(|r| r.result.total_chars)
        .sum();
    assert!(total_chars > 0, "Expected total_chars > 0");
}

#[test]
fn test_batch_summary_stats() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = create_test_files(&temp_dir);

    let processor = DefaultBatchProcessor::new();
    let job = BatchJob::new(vec![base_path], BatchOptions::default());

    let result = processor.process(&job);
    assert!(result.is_ok());

    let batch_result = result.unwrap();

    // Verify summary is consistent
    assert_eq!(
        batch_result.summary.total_files,
        batch_result.summary.successful
            + batch_result.summary.failed
            + batch_result.summary.skipped
    );

    // Check job ID is set
    assert!(!batch_result.summary.job_id.is_nil());

    // Check duration is reasonable
    assert!(batch_result.summary.duration.as_secs() < 60);
}
