//! Protect command implementation.

use std::fs;
use std::io::Write;
use std::path::Path;

use miette::{IntoDiagnostic, Result};

use veil_detect::DetectorRegistry;
use veil_parsers::{parse_file, ParseOptions};
use veil_policy::{apply_policy_to_findings, default_policy, load_policy};
use veil_redact::{RedactionEngine, RedactionStyle};

use crate::cli::ProtectArgs;
use crate::output;

/// Run the protect command.
pub fn run(args: ProtectArgs, quiet: bool, json: bool) -> Result<()> {
    // Load policy
    let policy = match &args.policy {
        Some(path) => load_policy(path).into_diagnostic()?,
        None => default_policy(),
    };

    // Parse style argument
    let style = match args.style.as_str() {
        "label" => RedactionStyle::Label,
        "bar" | "black_bar" => RedactionStyle::black_bar(),
        "mask" => RedactionStyle::mask(Default::default()),
        _ => RedactionStyle::Label,
    };

    let result = protect_file(&args.input, &policy, style, quiet)?;

    // Output
    if let Some(output_path) = &args.output {
        fs::write(output_path, &result.redacted_text).into_diagnostic()?;
        if !quiet {
            eprintln!(
                "Protected {} -> {} ({} redactions)",
                args.input.display(),
                output_path.display(),
                result.redaction_count
            );
        }
    } else {
        // Write to stdout
        std::io::stdout()
            .write_all(result.redacted_text.as_bytes())
            .into_diagnostic()?;
    }

    if json {
        output::print_json(&ProtectOutput {
            input: args.input.display().to_string(),
            output: args.output.map(|p| p.display().to_string()),
            redaction_count: result.redaction_count,
        })?;
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct ProtectOutput {
    input: String,
    output: Option<String>,
    redaction_count: usize,
}

struct ProtectResult {
    redacted_text: String,
    redaction_count: usize,
}

fn protect_file(
    path: &Path,
    policy: &veil_policy::Policy,
    default_style: RedactionStyle,
    _quiet: bool,
) -> Result<ProtectResult> {
    // Read file content
    let content = fs::read_to_string(path).into_diagnostic()?;

    // Parse file to get segments
    let parse_result = parse_file(path, &ParseOptions::default()).into_diagnostic()?;

    // Detect PII in segments
    let registry = DetectorRegistry::default();
    let findings = registry.detect_all(&parse_result.segments);
    let filtered = apply_policy_to_findings(policy, findings);

    // Convert segment-relative offsets to absolute file offsets
    let absolute_findings: Vec<_> = filtered
        .into_iter()
        .filter_map(|f| {
            let segment = parse_result.segments.get(f.segment_index)?;
            match &segment.position {
                veil_parsers::Position::Text { byte_offset, .. } => {
                    // Add segment's byte offset to finding's offset
                    Some(veil_detect::Finding {
                        start: byte_offset + f.start,
                        end: byte_offset + f.end,
                        ..f
                    })
                }
                veil_parsers::Position::Html { byte_offset, .. } => {
                    // HTML segments - use byte offset
                    Some(veil_detect::Finding {
                        start: byte_offset + f.start,
                        end: byte_offset + f.end,
                        ..f
                    })
                }
                _ => {
                    // For CSV/JSON, search for the matched text in content
                    // This is a fallback that may find the wrong occurrence
                    content
                        .find(&f.matched_text)
                        .map(|pos| veil_detect::Finding {
                            start: pos,
                            end: pos + f.matched_text.len(),
                            ..f
                        })
                }
            }
        })
        .collect();

    // Get redaction config - CLI style overrides policy styles
    let config = veil_redact::RedactionConfig::with_style(default_style);

    // Redact
    let engine = RedactionEngine::new(config);
    let result = engine.redact(&content, &absolute_findings);

    Ok(ProtectResult {
        redacted_text: result.text,
        redaction_count: result.redactions.len(),
    })
}
