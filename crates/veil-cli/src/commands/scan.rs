//! Scan command implementation.

use std::fs::{self, File};
use std::io::{Cursor, IsTerminal, Read};
use std::path::Path;

use miette::{IntoDiagnostic, Result};

use veil_detect::DetectorRegistry;
use veil_email::{parse_email_to_result, EmailParseOptions};
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

/// Run the scan command.
pub fn run(args: ScanArgs, quiet: bool, json: bool) -> Result<()> {
    // Handle --include-values confirmation
    let include_values = if args.include_values {
        if args.yes {
            // --yes flag bypasses confirmation
            if !quiet && !json {
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

    let registry = DetectorRegistry::default();
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
            } else if !quiet && !json {
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
            category: f.category.to_string(),
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

    let format = match result.metadata.format {
        FileFormat::Eml => "eml",
        FileFormat::Msg => "msg",
        _ => "email",
    };

    Ok((result, format.to_string()))
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
    for entry in fs::read_dir(dir).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();

        if path.is_dir() {
            scan_directory(
                &path,
                registry,
                policy,
                quiet,
                json,
                include_values,
                results,
            )?;
        } else {
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
                match scan_file(&path, registry, policy, quiet, include_values) {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        if !quiet && !json {
                            eprintln!("Error scanning {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
