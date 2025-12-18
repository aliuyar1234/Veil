//! Batch file processing example.
//!
//! Demonstrates how to scan multiple files in parallel.
//!
//! # Running
//!
//! ```bash
//! cargo run --example batch_processing -- /path/to/directory
//! ```

use std::env;
use std::path::PathBuf;

use veil_discovery::{DiscoveryOptions, ReportFormat, ReportGenerator, Scanner};

fn main() {
    // Get directory from command line args
    let args: Vec<String> = env::args().collect();
    let root_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        // Default to current directory
        env::current_dir().expect("Failed to get current directory")
    };

    println!("=== Veil Batch Processing Example ===\n");
    println!("Scanning directory: {}\n", root_path.display());

    // Configure discovery options
    let options = DiscoveryOptions {
        root_path: root_path.clone(),
        // Include common text files
        include_patterns: vec![
            "*.txt".to_string(),
            "*.csv".to_string(),
            "*.json".to_string(),
            "*.html".to_string(),
        ],
        // Exclude common non-content files
        exclude_patterns: vec![
            "*.exe".to_string(),
            "*.dll".to_string(),
            "*.so".to_string(),
            "node_modules/**".to_string(),
            ".git/**".to_string(),
        ],
        // Follow symlinks
        follow_symlinks: false,
        // Scan inside archives
        scan_archives: false,
        // Sample large files
        sample_large_files: true,
        max_file_size: 10 * 1024 * 1024, // 10MB
        ..Default::default()
    };

    // Create scanner
    let scanner = Scanner::new(options);

    // Run discovery
    println!("Starting scan...\n");
    let result = match scanner.scan() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Scan failed: {}", e);
            return;
        }
    };

    // Print statistics
    println!("=== Scan Statistics ===\n");
    println!("Files scanned: {}", result.statistics.files_scanned);
    println!("Files with PII: {}", result.statistics.files_with_pii);
    println!("Total findings: {}", result.statistics.total_findings);
    println!(
        "Scan duration: {:?}",
        result.statistics.end_time.map(|e| e - result.statistics.start_time)
    );
    println!();

    // Print discovered files
    if !result.discovered_files.is_empty() {
        println!("=== Files with PII ===\n");
        for file in &result.discovered_files {
            println!("File: {}", file.path.display());
            println!("  PII types found: {:?}", file.pii_categories);
            println!("  Finding count: {}", file.finding_count);
            println!();
        }
    }

    // Generate summary report
    println!("=== Summary Report ===\n");
    match ReportGenerator::generate(&result, ReportFormat::Summary) {
        Ok(report) => println!("{}", report),
        Err(e) => eprintln!("Failed to generate report: {}", e),
    }

    // Optionally generate JSON report
    if args.iter().any(|a| a == "--json") {
        println!("\n=== JSON Report ===\n");
        match ReportGenerator::generate(&result, ReportFormat::Json) {
            Ok(report) => println!("{}", report),
            Err(e) => eprintln!("Failed to generate JSON report: {}", e),
        }
    }
}
