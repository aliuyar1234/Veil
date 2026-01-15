# Feature Specification: CLI Scan & Protect

**Feature Branch**: `004-cli-scan-protect`
**Created**: 2025-12-08
**Status**: Draft
**Input**: Command-line interface with scan and protect commands

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scan Single File (Priority: P1)

A privacy analyst runs `veil scan document.txt` from the command line and receives a report of all detected PII findings with their locations and categories.

**Why this priority**: Single file scanning is the most basic operation and validates the end-to-end pipeline from parsing through detection.

**Independent Test**: Run CLI with a test file containing known PII, verify output lists all expected findings.

**Acceptance Scenarios**:

1. **Given** a text file with PII, **When** running `veil scan file.txt`, **Then** CLI outputs findings in human-readable format.
2. **Given** a file with no PII, **When** scanned, **Then** CLI reports "No PII detected" with exit code 0.
3. **Given** a non-existent file, **When** scanned, **Then** CLI shows error message and exits with non-zero code.

---

### User Story 2 - Scan Directory Recursively (Priority: P1)

A compliance team needs to scan an entire folder structure for PII. They run `veil scan ./documents/ --recursive` and receive aggregated findings across all supported files.

**Why this priority**: Batch scanning is essential for real-world use cases where users have many files to process.

**Independent Test**: Create directory with multiple files at different depths, run recursive scan, verify all files are processed.

**Acceptance Scenarios**:

1. **Given** a directory with nested folders, **When** running `veil scan ./dir --recursive`, **Then** all supported files are scanned.
2. **Given** mixed file types (supported and unsupported), **When** scanned, **Then** unsupported files are skipped with warning.
3. **Given** `--recursive` flag omitted, **When** scanning directory, **Then** only top-level files are scanned.

---

### User Story 3 - Output Findings as JSON (Priority: P1)

A developer wants to integrate scan results into a pipeline. They run `veil scan file.txt --output findings.json` or `veil scan file.txt --format json` to get machine-readable output.

**Why this priority**: JSON output enables automation and integration with other tools.

**Independent Test**: Run scan with JSON output, verify output is valid JSON with expected schema.

**Acceptance Scenarios**:

1. **Given** `--format json` flag, **When** scanning, **Then** output is valid JSON array of findings.
2. **Given** `--output findings.json` flag, **When** scanning, **Then** results are written to file, stdout shows summary.
3. **Given** both flags, **When** scanning, **Then** JSON is written to file in specified format.

---

### User Story 4 - Protect File with Redaction (Priority: P1)

A privacy analyst runs `veil protect document.txt -o redacted.txt` to create a redacted version of the file with all detected PII replaced.

**Why this priority**: Protection is the primary value proposition - scanning alone doesn't solve the user's problem.

**Independent Test**: Run protect on file with PII, verify output file contains redacted text with no original PII.

**Acceptance Scenarios**:

1. **Given** a file with PII, **When** running `veil protect file.txt -o output.txt`, **Then** output file has PII redacted.
2. **Given** `--style labels` flag, **When** protecting, **Then** PII is replaced with `[CATEGORY]` labels.
3. **Given** `--style bars` flag, **When** protecting, **Then** PII is replaced with `████` matching length.

---

### User Story 5 - Show Progress for Large Operations (Priority: P2)

A user scans a large directory and sees a progress bar indicating files processed, estimated time remaining, and current file being scanned.

**Why this priority**: Progress feedback is essential for user experience during long-running operations.

**Independent Test**: Scan a directory with many files, verify progress bar appears and updates.

**Acceptance Scenarios**:

1. **Given** a directory with 100+ files, **When** scanning, **Then** progress bar shows percentage complete.
2. **Given** single file operation, **When** scanning, **Then** spinner or brief status shown.
3. **Given** `--quiet` flag, **When** scanning, **Then** no progress output, only final results.

---

### User Story 6 - Configure Detection Categories (Priority: P2)

A user only wants to scan for specific PII types. They run `veil scan file.txt --detect email,iban` to limit detection to those categories.

**Why this priority**: Flexibility to focus on specific PII types reduces noise and improves performance.

**Independent Test**: Scan file with multiple PII types using limited detection, verify only specified types found.

**Acceptance Scenarios**:

1. **Given** `--detect email,phone`, **When** scanning file with IBAN, **Then** IBAN not reported.
2. **Given** `--detect all`, **When** scanning, **Then** all detectors enabled (default behavior).
3. **Given** invalid detector name, **When** scanning, **Then** error message lists valid options.

---

### User Story 7 - Apply Policy File (Priority: P1)

A compliance officer uses a YAML policy file to configure detection thresholds, protection actions, and locale settings. They run `veil scan file.txt --policy gdpr.yaml` or `veil protect file.txt --policy gdpr.yaml -o output.txt`.

**Why this priority**: Policy files are the primary configuration mechanism for enterprise use, enabling reproducible and auditable behavior.

**Independent Test**: Create policy file with specific rules, run scan/protect, verify policy rules are applied.

**Acceptance Scenarios**:

1. **Given** `--policy gdpr.yaml` with confidence threshold 0.8, **When** scanning, **Then** findings below 0.8 are filtered out.
2. **Given** `--policy` with locale `de-AT`, **When** scanning, **Then** Austrian-specific detectors are enabled.
3. **Given** `--policy` with protection rules, **When** protecting, **Then** each PII type uses specified action (redact/mask/encrypt).
4. **Given** invalid policy file, **When** running command, **Then** clear validation error shown before processing.
5. **Given** no `--policy` flag, **When** running command, **Then** default policy applied (all detectors, redact with labels).

---

### User Story 8 - Validate Policy File (Priority: P2)

A developer wants to check if their policy file is valid before using it. They run `veil policy validate gdpr.yaml` to verify syntax and semantics.

**Why this priority**: Policy validation prevents runtime errors and provides clear feedback during policy development.

**Independent Test**: Provide valid and invalid policy files, verify validation output.

**Acceptance Scenarios**:

1. **Given** valid policy file, **When** validating, **Then** "Policy valid" message shown.
2. **Given** policy with syntax error, **When** validating, **Then** error with line number shown.
3. **Given** policy with unknown detector name, **When** validating, **Then** warning about unknown detector.

---

### Edge Cases

- What happens when output file already exists? System prompts for confirmation unless `--force` flag used.
- What happens with permission errors? System reports error for specific file and continues with others.
- What happens when stdin is piped? System reads from stdin when `-` is provided as filename.
- What happens with binary files? System detects binary content and skips with warning.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: CLI MUST provide `scan` subcommand for PII detection.
- **FR-002**: CLI MUST provide `protect` subcommand for PII redaction.
- **FR-003**: CLI MUST accept file paths and directory paths as arguments.
- **FR-004**: CLI MUST support `--recursive` flag for directory scanning.
- **FR-005**: CLI MUST support `--output` or `-o` flag for specifying output file.
- **FR-006**: CLI MUST support `--format` flag with options: `text` (default), `json`.
- **FR-007**: CLI MUST support `--style` flag for redaction with options: `labels` (default), `bars`, `mask`.
- **FR-008**: CLI MUST support `--detect` flag to limit PII categories.
- **FR-009**: CLI MUST show progress indication for multi-file operations.
- **FR-010**: CLI MUST support `--quiet` flag to suppress progress output.
- **FR-011**: CLI MUST return appropriate exit codes: 0 (success), 1 (error), 2 (PII found in scan-only mode if `--fail-on-findings`).
- **FR-012**: CLI MUST read from stdin when filename is `-`.
- **FR-013**: CLI MUST provide `--help` for all commands and subcommands.
- **FR-014**: CLI MUST provide `--version` flag showing version information.
- **FR-015**: CLI MUST support `--policy` or `-p` flag to specify a YAML policy file.
- **FR-016**: CLI MUST validate policy file on load and report errors before processing.
- **FR-017**: CLI MUST provide `policy` subcommand with `validate` action for policy file validation.
- **FR-018**: CLI MUST apply default policy when no `--policy` flag is provided.

### Key Entities

- **Command**: A CLI subcommand (scan, protect, policy); defines arguments, flags, and execution logic.
- **ScanOptions**: Configuration for scan operation; includes paths, recursive flag, output format, detector filter, policy file path.
- **ProtectOptions**: Configuration for protect operation; includes input/output paths, redaction style, policy file path.
- **ScanResult**: Output of scan operation; contains list of findings, file statistics, error summary.
- **CliOutput**: Formatted output handler; supports text and JSON rendering of results.
- **PolicyRef**: Reference to a policy file or default policy; validated before use.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: User can scan a single file and see results in under 2 seconds for files under 1MB.
- **SC-002**: User can scan 1000 files recursively with progress indication completing in under 5 minutes.
- **SC-003**: JSON output parses successfully with standard JSON tools (jq, Python json).
- **SC-004**: Protected output files contain zero instances of originally detected PII.
- **SC-005**: CLI help text is clear enough for new users to run basic operations without documentation.
- **SC-006**: Exit codes enable integration with shell scripts and CI/CD pipelines.

## Assumptions

- CLI is the primary interface; other interfaces (API, WASM) will wrap the same core logic.
- File format detection is automatic based on extension and content.
- Default output goes to stdout for scan, requires explicit `-o` for protect.
- The CLI operates synchronously; background/daemon mode is out of scope.
- Policy file format and schema are defined in Spec 009 (Policy Engine).
- Default policy uses all detectors with confidence threshold 0.5 and redaction style `labels`.
