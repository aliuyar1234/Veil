//! Protect command implementation.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use miette::{miette, IntoDiagnostic, Result};

use veil_detect::DetectorRegistry;
use veil_parsers::{parse_file, FileFormat, ParseOptions, ParseResult, Position};
use veil_policy::{apply_policy_to_findings, default_policy, load_policy};
use veil_redact::{RedactionConfig, RedactionEngine, RedactionStyle};

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
    // Parse file to get segments
    let parse_result = parse_file(path, &ParseOptions::default()).into_diagnostic()?;

    // Detect PII in segments
    let registry = DetectorRegistry::default();
    let findings = registry.detect_all(&parse_result.segments);
    let filtered = apply_policy_to_findings(policy, findings);

    // Get redaction config - CLI style overrides policy styles
    let config = RedactionConfig::with_style(default_style);

    match parse_result.metadata.format {
        FileFormat::Text | FileFormat::Html => {
            let content = fs::read_to_string(path).into_diagnostic()?;
            protect_text(&content, &parse_result, filtered, &config)
        }
        FileFormat::Csv => {
            let content = fs::read_to_string(path).into_diagnostic()?;
            protect_csv(&content, &parse_result, filtered, &config)
        }
        FileFormat::Json => {
            let content = fs::read_to_string(path).into_diagnostic()?;
            protect_json(&content, &parse_result, filtered, &config)
        }
        other => Err(miette!(
            "Protect does not support format {:?}. Supported formats: text, html, csv, json.",
            other
        )),
    }
}

fn protect_text(
    content: &str,
    parse_result: &ParseResult,
    findings: Vec<veil_detect::Finding>,
    config: &RedactionConfig,
) -> Result<ProtectResult> {
    let absolute_findings: Vec<_> = findings
        .into_iter()
        .filter_map(|f| {
            let segment = parse_result.segments.get(f.segment_index)?;
            match &segment.position {
                Position::Text { byte_offset, .. } | Position::Html { byte_offset, .. } => {
                    Some(veil_detect::Finding {
                        start: *byte_offset + f.start,
                        end: *byte_offset + f.end,
                        ..f
                    })
                }
                _ => None,
            }
        })
        .collect();

    let engine = RedactionEngine::new(config.clone());
    let result = engine.redact(content, &absolute_findings);

    Ok(ProtectResult {
        redacted_text: result.text,
        redaction_count: result.redactions.len(),
    })
}

fn protect_csv(
    content: &str,
    parse_result: &ParseResult,
    findings: Vec<veil_detect::Finding>,
    config: &RedactionConfig,
) -> Result<ProtectResult> {
    let mut segment_positions = vec![None; parse_result.segments.len()];
    for (idx, segment) in parse_result.segments.iter().enumerate() {
        if let Position::Csv { row, column, .. } = &segment.position {
            segment_positions[idx] = Some((*row, *column));
        }
    }

    let mut findings_by_cell: HashMap<(usize, usize), Vec<veil_detect::Finding>> = HashMap::new();
    for finding in findings {
        let position = segment_positions
            .get(finding.segment_index)
            .and_then(|pos| *pos)
            .ok_or_else(|| {
                miette!(
                    "Missing CSV position for segment index {}",
                    finding.segment_index
                )
            })?;
        findings_by_cell.entry(position).or_default().push(finding);
    }

    let options = ParseOptions::default();
    let delimiter = options.csv_delimiter.unwrap_or(b',');
    let has_headers = options.csv_has_headers.unwrap_or(true);

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(has_headers)
        .flexible(true)
        .from_reader(content.as_bytes());

    let headers = if has_headers {
        Some(
            reader
                .headers()
                .map_err(|e| miette!("CSV header read error: {}", e))?
                .clone(),
        )
    } else {
        None
    };

    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .has_headers(has_headers)
        .from_writer(vec![]);

    if let Some(headers) = headers {
        writer
            .write_record(&headers)
            .map_err(|e| miette!("CSV header write error: {}", e))?;
    }

    let mut redaction_count = 0;
    let engine = RedactionEngine::new(config.clone());

    for (row_idx, result) in reader.records().enumerate() {
        let record =
            result.map_err(|e| miette!("CSV record read error at row {}: {}", row_idx + 1, e))?;
        let row_num = row_idx + if has_headers { 2 } else { 1 };

        let mut fields: Vec<String> = record.iter().map(|value| value.to_string()).collect();
        for col_idx in 0..record.len() {
            if let Some(cell_findings) = findings_by_cell.get(&(row_num, col_idx)) {
                let value = record.get(col_idx).unwrap_or("");
                let redacted = engine.redact(value, cell_findings);
                redaction_count += redacted.redactions.len();
                if let Some(field) = fields.get_mut(col_idx) {
                    *field = redacted.text;
                }
            }
        }

        writer
            .write_record(&fields)
            .map_err(|e| miette!("CSV record write error at row {}: {}", row_num, e))?;
    }

    let bytes = writer
        .into_inner()
        .map_err(|e| miette!("CSV writer flush error: {}", e))?;
    let redacted_text =
        String::from_utf8(bytes).map_err(|e| miette!("CSV output is not UTF-8: {}", e))?;

    Ok(ProtectResult {
        redacted_text,
        redaction_count,
    })
}

fn protect_json(
    content: &str,
    parse_result: &ParseResult,
    findings: Vec<veil_detect::Finding>,
    config: &RedactionConfig,
) -> Result<ProtectResult> {
    let mut segment_paths = vec![None; parse_result.segments.len()];
    for (idx, segment) in parse_result.segments.iter().enumerate() {
        if let Position::Json { path } = &segment.position {
            segment_paths[idx] = Some(path.clone());
        }
    }

    let mut findings_by_path: HashMap<String, Vec<veil_detect::Finding>> = HashMap::new();
    for finding in findings {
        let path = segment_paths
            .get(finding.segment_index)
            .and_then(|pos| pos.clone())
            .ok_or_else(|| {
                miette!(
                    "Missing JSON path for segment index {}",
                    finding.segment_index
                )
            })?;
        findings_by_path.entry(path).or_default().push(finding);
    }

    let mut value: serde_json::Value =
        serde_json::from_str(content).map_err(|e| miette!("JSON parse error: {}", e))?;

    let engine = RedactionEngine::new(config.clone());
    let mut redaction_count = 0;

    for (path, path_findings) in findings_by_path {
        let segments = parse_json_path(&path)?;
        let target = get_json_value_mut(&mut value, &segments)
            .ok_or_else(|| miette!("JSON path not found for redaction: {}", path))?;

        let current = match target {
            serde_json::Value::String(s) => s.clone(),
            _ => return Err(miette!("JSON path is not a string value: {}", path)),
        };

        let redacted = engine.redact(&current, &path_findings);
        redaction_count += redacted.redactions.len();
        *target = serde_json::Value::String(redacted.text);
    }

    let redacted_text =
        serde_json::to_string_pretty(&value).map_err(|e| miette!("JSON write error: {}", e))?;

    Ok(ProtectResult {
        redacted_text,
        redaction_count,
    })
}

#[derive(Debug)]
enum JsonPathSegment {
    Key(String),
    Index(usize),
}

fn parse_json_path(path: &str) -> Result<Vec<JsonPathSegment>> {
    let mut chars = path.chars().peekable();
    match chars.next() {
        Some('$') => {}
        _ => return Err(miette!("Invalid JSON path: {}", path)),
    }

    let mut segments = Vec::new();
    while let Some(&ch) = chars.peek() {
        match ch {
            '.' => {
                chars.next();
                let mut key = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '.' || c == '[' {
                        break;
                    }
                    key.push(c);
                    chars.next();
                }
                if key.is_empty() {
                    return Err(miette!("Invalid JSON path: {}", path));
                }
                segments.push(JsonPathSegment::Key(key));
            }
            '[' => {
                chars.next();
                let mut index = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ']' {
                        break;
                    }
                    index.push(c);
                    chars.next();
                }
                if chars.next() != Some(']') {
                    return Err(miette!("Invalid JSON path: {}", path));
                }
                let idx = index
                    .parse::<usize>()
                    .map_err(|_| miette!("Invalid JSON array index in path: {}", path))?;
                segments.push(JsonPathSegment::Index(idx));
            }
            _ => return Err(miette!("Invalid JSON path: {}", path)),
        }
    }

    Ok(segments)
}

fn get_json_value_mut<'a>(
    value: &'a mut serde_json::Value,
    segments: &[JsonPathSegment],
) -> Option<&'a mut serde_json::Value> {
    let mut current = value;
    for segment in segments {
        match segment {
            JsonPathSegment::Key(key) => {
                current = current.get_mut(key)?;
            }
            JsonPathSegment::Index(index) => {
                current = current.get_mut(*index)?;
            }
        }
    }
    Some(current)
}
