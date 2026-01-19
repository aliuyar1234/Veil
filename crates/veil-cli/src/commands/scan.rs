//! Scan command implementation.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Cursor, IsTerminal, Read};
use std::path::Path;

use miette::{IntoDiagnostic, Result};

use veil_core::DEFAULT_MAX_FILE_SIZE;
use veil_detect::DetectorRegistry;
use veil_email::{parse_email_to_result, EmailParseOptions};
use veil_fs::{walk_files, WalkFilesOptions};
use veil_office::{parse_docx, parse_pptx, parse_xlsx};
use veil_parsers::{parse_file, FileFormat, ParseOptions, ParseResult};
use veil_policy::{apply_policy_to_findings, default_policy, load_policy};

use crate::cli::ScanArgs;
use crate::output;

/// Supported file extensions for scanning.
const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "csv", "json", "html", "htm", "log", "md", "xml", "yaml", "yml", "toml", "ini", "cfg",
];
const OFFICE_EXTENSIONS: &[&str] = &["docx", "xlsx", "pptx"];
const EMAIL_EXTENSIONS: &[&str] = &["eml", "msg"];
const PDF_EXTENSIONS: &[&str] = &["pdf"];

fn should_print_human_stderr(quiet: bool, json: bool) -> bool {
    !quiet && !json
}

fn apply_detect_filter(
    registry: &mut DetectorRegistry,
    detect: &Option<Vec<String>>,
) -> Result<()> {
    let Some(selected) = detect else {
        return Ok(());
    };

    let selected: HashSet<String> = selected.iter().map(|s| s.to_ascii_lowercase()).collect();
    let detector_names: Vec<String> = registry
        .detector_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let available: HashSet<String> = detector_names
        .iter()
        .map(|s| s.to_ascii_lowercase())
        .collect();

    let mut unknown: Vec<String> = selected.difference(&available).cloned().collect();
    unknown.sort();
    if !unknown.is_empty() {
        let mut available: Vec<String> = available.into_iter().collect();
        available.sort();
        return Err(miette::miette!(
            "Unknown detector(s): {}. Available: {}",
            unknown.join(", "),
            available.join(", ")
        ));
    }

    for name in detector_names {
        if !selected.contains(&name.to_ascii_lowercase()) {
            registry.disable(&name);
        }
    }

    Ok(())
}

/// Run the scan command.
pub fn run(args: ScanArgs, quiet: bool, json: bool) -> Result<()> {
    // Handle --include-values confirmation
    let include_values = if args.include_values {
        if args.yes {
            // --yes flag bypasses confirmation
            if should_print_human_stderr(quiet, json) {
                eprintln!("Warning: Including PII values in output (--yes flag used)");
            }
            true
        } else if std::io::stdin().is_terminal() {
            // Interactive mode: prompt for confirmation
            eprintln!("WARNING: Including PII values exposes sensitive data in output.");
            eprintln!("This may be captured in logs, terminal history, or other systems.");
            eprint!("Do you understand the security implications? (yes/no): ");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).into_diagnostic()?;

            if input.trim().eq_ignore_ascii_case("yes") {
                true
            } else {
                eprintln!("Aborted. Use --include-values --yes to skip this prompt.");
                return Ok(());
            }
        } else {
            // Non-interactive mode without --yes: reject
            return Err(miette::miette!(
                "The --include-values flag requires --yes in non-interactive mode"
            ));
        }
    } else {
        false
    };

    // Load policy
    let policy = match &args.policy {
        Some(path) => load_policy(path).into_diagnostic()?,
        None => default_policy(),
    };

    let mut registry = DetectorRegistry::default();
    apply_detect_filter(&mut registry, &args.detect)?;
    let mut total_findings = 0;
    let mut all_results = Vec::new();

    for path in &args.paths {
        if path.is_dir() {
            if args.recursive {
                scan_directory(
                    path,
                    &registry,
                    &policy,
                    quiet,
                    json,
                    include_values,
                    &mut all_results,
                )?;
            } else if should_print_human_stderr(quiet, json) {
                eprintln!(
                    "Skipping directory: {} (use -r for recursive)",
                    path.display()
                );
            }
        } else {
            match scan_file(path, &registry, &policy, quiet, include_values) {
                Ok(result) => {
                    total_findings += result.findings_count;
                    all_results.push(result);
                }
                Err(e) => {
                    if !quiet {
                        eprintln!("Error scanning {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    // Output results
    if json {
        output::print_json(&all_results)?;
    } else if !quiet {
        for result in &all_results {
            output::print_scan_result(result);
        }
        println!(
            "\nTotal: {} findings in {} files",
            total_findings,
            all_results.len()
        );
    }

    // Exit code based on findings
    if args.fail_on_findings && total_findings > 0 {
        std::process::exit(2);
    }

    Ok(())
}

#[derive(serde::Serialize)]
pub struct ScanResult {
    pub file: String,
    pub format: String,
    pub findings_count: usize,
    pub findings: Vec<FindingOutput>,
}

#[derive(serde::Serialize)]
pub struct FindingOutput {
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub position: String,
    pub confidence: f32,
}

fn scan_file(
    path: &Path,
    registry: &DetectorRegistry,
    policy: &veil_policy::Policy,
    quiet: bool,
    include_values: bool,
) -> Result<ScanResult> {
    if !quiet {
        eprintln!("Scanning: {}", path.display());
    }

    enforce_max_input_size(path)?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Parse based on file type
    let (parse_result, format_name) = if OFFICE_EXTENSIONS.contains(&ext.as_str()) {
        parse_office_file(path)?
    } else if EMAIL_EXTENSIONS.contains(&ext.as_str()) {
        parse_email_file(path)?
    } else if PDF_EXTENSIONS.contains(&ext.as_str()) {
        parse_pdf_file(path)?
    } else {
        // Default to text-based parsing
        let result = parse_file(path, &ParseOptions::default()).into_diagnostic()?;
        (result, "text".to_string())
    };

    // Detect PII
    let findings = registry.detect_all(&parse_result.segments);
    let filtered = apply_policy_to_findings(policy, findings);

    let finding_outputs: Vec<FindingOutput> = filtered
        .iter()
        .map(|f| FindingOutput {
            category: f.category.as_str().to_string(),
            // Only include PII text if explicitly requested
            // Use .as_str() to get the actual value (Display is intentionally redacted)
            text: if include_values {
                Some(f.matched_text.as_str().to_string())
            } else {
                None
            },
            position: format!("{}..{}", f.start, f.end),
            confidence: f.confidence,
        })
        .collect();

    Ok(ScanResult {
        file: path.display().to_string(),
        format: format_name,
        findings_count: finding_outputs.len(),
        findings: finding_outputs,
    })
}

fn enforce_max_input_size(path: &Path) -> Result<()> {
    let size = std::fs::metadata(path).into_diagnostic()?.len();
    let max = DEFAULT_MAX_FILE_SIZE as u64;

    if size > max {
        return Err(miette::miette!(
            "File too large: {} bytes (max: {} bytes)",
            size,
            max
        ));
    }

    Ok(())
}

fn parse_office_file(path: &Path) -> Result<(ParseResult, String)> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (result, format) = match ext.as_str() {
        "docx" => {
            let file = File::open(path).into_diagnostic()?;
            let parsed =
                parse_docx(file).map_err(|e| miette::miette!("DOCX parse error: {}", e))?;
            (parsed, "docx")
        }
        "xlsx" => {
            // Read file into memory for xlsx (requires Clone)
            let mut file = File::open(path).into_diagnostic()?;
            let mut data = Vec::new();
            file.read_to_end(&mut data).into_diagnostic()?;
            let cursor = Cursor::new(data);
            let parsed =
                parse_xlsx(cursor).map_err(|e| miette::miette!("XLSX parse error: {}", e))?;
            (parsed, "xlsx")
        }
        "pptx" => {
            let file = File::open(path).into_diagnostic()?;
            let parsed =
                parse_pptx(file).map_err(|e| miette::miette!("PPTX parse error: {}", e))?;
            (parsed, "pptx")
        }
        _ => return Err(miette::miette!("Unsupported office format: {}", ext)),
    };

    Ok((result, format.to_string()))
}

fn parse_email_file(path: &Path) -> Result<(ParseResult, String)> {
    let data = fs::read(path).into_diagnostic()?;
    let options = EmailParseOptions::default();

    let result = parse_email_to_result(&data, &options)
        .map_err(|e| miette::miette!("Email parse error: {}", e))?;

    let format = email_format_label(result.metadata.format);

    Ok((result, format.to_string()))
}

fn email_format_label(format: FileFormat) -> &'static str {
    match format {
        FileFormat::Eml => "eml",
        FileFormat::Msg => "msg",
        _ => "email",
    }
}

fn parse_pdf_file(path: &Path) -> Result<(ParseResult, String)> {
    // Use veil-parsers PDF support
    let result = parse_file(path, &ParseOptions::default()).into_diagnostic()?;
    Ok((result, "pdf".to_string()))
}

fn scan_directory(
    dir: &Path,
    registry: &DetectorRegistry,
    policy: &veil_policy::Policy,
    quiet: bool,
    json: bool,
    include_values: bool,
    results: &mut Vec<ScanResult>,
) -> Result<()> {
    let walk_options = WalkFilesOptions {
        follow_symlinks: false,
        max_depth: None,
    };

    for entry in walk_files(dir, walk_options) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                if should_print_human_stderr(quiet, json) {
                    eprintln!("Warning: Failed to access path: {}", e);
                }
                continue;
            }
        };

        let path = entry.path();

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check if file type is supported
        let supported = TEXT_EXTENSIONS.contains(&ext.as_str())
            || OFFICE_EXTENSIONS.contains(&ext.as_str())
            || EMAIL_EXTENSIONS.contains(&ext.as_str())
            || PDF_EXTENSIONS.contains(&ext.as_str());

        if supported {
            match scan_file(path, registry, policy, quiet, include_values) {
                Ok(result) => results.push(result),
                Err(e) => {
                    if should_print_human_stderr(quiet, json) {
                        eprintln!("Error scanning {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn should_print_human_stderr_requires_nonquiet_nonjson() {
        assert!(should_print_human_stderr(false, false));
        assert!(!should_print_human_stderr(true, false));
        assert!(!should_print_human_stderr(false, true));
        assert!(!should_print_human_stderr(true, true));
    }

    #[test]
    fn enforce_max_input_size_allows_exact_max_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("exact_max.bin");
        let file = std::fs::File::create(&file_path).unwrap();
        file.set_len(DEFAULT_MAX_FILE_SIZE as u64).unwrap();

        assert!(enforce_max_input_size(&file_path).is_ok());
    }

    #[test]
    fn enforce_max_input_size_rejects_large_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("big.bin");

        let file = std::fs::File::create(&file_path).unwrap();
        file.set_len(DEFAULT_MAX_FILE_SIZE as u64 + 1).unwrap();

        assert!(enforce_max_input_size(&file_path).is_err());
    }

    #[test]
    fn parse_office_file_routes_to_correct_parser_by_extension() {
        let temp_dir = TempDir::new().unwrap();

        let docx = temp_dir.path().join("file.docx");
        std::fs::write(&docx, []).unwrap();
        let err = parse_office_file(&docx).unwrap_err();
        assert!(format!("{err}").contains("DOCX parse error"));

        let xlsx = temp_dir.path().join("file.xlsx");
        std::fs::write(&xlsx, []).unwrap();
        let err = parse_office_file(&xlsx).unwrap_err();
        assert!(format!("{err}").contains("XLSX parse error"));

        let pptx = temp_dir.path().join("file.pptx");
        std::fs::write(&pptx, []).unwrap();
        let err = parse_office_file(&pptx).unwrap_err();
        assert!(format!("{err}").contains("PPTX parse error"));
    }

    #[test]
    fn email_format_label_is_stable() {
        assert_eq!(email_format_label(FileFormat::Eml), "eml");
        assert_eq!(email_format_label(FileFormat::Msg), "msg");
        assert_eq!(email_format_label(FileFormat::Pdf), "email");
    }

    #[test]
    fn apply_detect_filter_rejects_unknown_detectors() {
        let mut registry = DetectorRegistry::default();
        let detect = Some(vec!["definitely-not-a-detector".to_string()]);
        assert!(apply_detect_filter(&mut registry, &detect).is_err());
    }

    #[test]
    fn apply_detect_filter_disables_unselected_detectors() {
        let mut registry = DetectorRegistry::default();
        let all: Vec<String> = registry
            .detector_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(all.iter().any(|s| s == "email"));

        apply_detect_filter(&mut registry, &Some(vec!["email".to_string()])).unwrap();
        assert!(registry.is_enabled("email"));

        let other = all
            .iter()
            .find(|name| *name != "email")
            .expect("expected at least one detector besides email");
        assert!(!registry.is_enabled(other));
    }
}
