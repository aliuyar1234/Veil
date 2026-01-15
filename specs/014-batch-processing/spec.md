# Feature Specification: Batch Processing

**Feature Branch**: `014-batch-processing`
**Created**: 2025-12-15
**Status**: Draft
**Input**: Multi-file and directory processing for bulk PII operations

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scan Entire Directory (Priority: P1)

A privacy analyst needs to scan all documents in a folder structure for PII. The system recursively processes all supported file types and aggregates findings.

**Why this priority**: Enterprise use requires scanning thousands of files across nested directories.

**Independent Test**: Provide directory with mixed file types, scan recursively, verify all files processed.

**Acceptance Scenarios**:

1. **Given** a directory with 100 files, **When** batch scan executed, **Then** all supported files are processed.
2. **Given** nested subdirectories 5 levels deep, **When** `--recursive` flag used, **Then** all files in all subdirectories scanned.
3. **Given** mix of .txt, .csv, .json, .pdf files, **When** scanned, **Then** each file uses appropriate parser.

---

### User Story 2 - Filter Files by Pattern (Priority: P1)

A security team wants to scan only specific file types in a large archive. The system supports glob patterns to include/exclude files.

**Why this priority**: Targeted scanning improves performance and relevance for specific audits.

**Independent Test**: Provide directory with various file types, filter by pattern, verify only matching files processed.

**Acceptance Scenarios**:

1. **Given** `--include "*.csv"` flag, **When** scanned, **Then** only CSV files are processed.
2. **Given** `--exclude "*.log"` flag, **When** scanned, **Then** log files are skipped.
3. **Given** complex pattern `"**/reports/*.pdf"`, **When** scanned, **Then** only PDFs in reports folders processed.

---

### User Story 3 - Process ZIP Archives (Priority: P1)

A compliance officer receives a data export as a ZIP file. The system extracts and processes all files within the archive without manual extraction.

**Why this priority**: Data exports, DSAR responses, and backups are often delivered as archives.

**Independent Test**: Provide ZIP with various file types, process, verify all contents scanned.

**Acceptance Scenarios**:

1. **Given** a ZIP file, **When** processed, **Then** all files inside are scanned.
2. **Given** nested ZIPs (ZIP within ZIP), **When** processed, **Then** inner archives are also extracted and scanned.
3. **Given** password-protected ZIP, **When** password provided via `--password`, **Then** archive is decrypted and processed.

---

### User Story 4 - Parallel Processing (Priority: P2)

A data engineer needs to scan a large dataset quickly. The system processes multiple files concurrently using available CPU cores.

**Why this priority**: Performance is critical for enterprise-scale scanning of millions of records.

**Independent Test**: Scan 1000 files with parallel processing, verify speedup vs sequential.

**Acceptance Scenarios**:

1. **Given** `--parallel 8` flag, **When** scanning 100 files, **Then** up to 8 files processed concurrently.
2. **Given** default settings, **When** scanning, **Then** uses number of CPU cores minus 1.
3. **Given** `--parallel 1` flag, **When** scanning, **Then** files processed sequentially.

---

### User Story 5 - Progress Reporting (Priority: P2)

A user running a long batch job needs visibility into progress. The system reports progress with file count, percentage, and ETA.

**Why this priority**: Long-running jobs need progress feedback for user confidence and planning.

**Independent Test**: Run batch scan, verify progress updates are emitted.

**Acceptance Scenarios**:

1. **Given** batch scan in progress, **When** `--progress` flag enabled, **Then** shows `[123/1000] 12% - ETA: 5m`.
2. **Given** interactive terminal, **When** scanning, **Then** progress bar updates in place.
3. **Given** piped output, **When** scanning, **Then** progress written as discrete lines.

---

### User Story 6 - Aggregate Results (Priority: P2)

A compliance team needs a summary report across all scanned files. The system aggregates findings into a single report with per-file breakdown.

**Why this priority**: Aggregate views enable trend analysis and risk prioritization.

**Independent Test**: Scan directory, generate aggregate report, verify totals match individual files.

**Acceptance Scenarios**:

1. **Given** 100 files scanned, **When** aggregate report generated, **Then** shows total findings by category.
2. **Given** JSON output format, **When** generated, **Then** includes per-file findings array.
3. **Given** findings across files, **When** grouped by category, **Then** shows which files contain each PII type.

---

### Edge Cases

- What happens with symlinks? System follows symlinks by default; `--no-follow-symlinks` to disable.
- What happens with very large files (>1GB)? System processes in streaming mode or skips with warning.
- What happens with permission denied? System logs error and continues with other files.
- What happens with corrupted files? System logs error for file and continues processing.
- What happens with duplicate files? System processes each file independently; deduplication is future feature.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST recursively scan directories for supported file types.
- **FR-002**: System MUST support glob patterns for file include/exclude filtering.
- **FR-003**: System MUST process ZIP archives without manual extraction.
- **FR-004**: System MUST support password-protected ZIP archives.
- **FR-005**: System MUST support parallel file processing with configurable concurrency.
- **FR-006**: System MUST report progress during batch operations.
- **FR-007**: System MUST generate aggregate reports across all processed files.
- **FR-008**: System MUST handle file access errors gracefully and continue processing.
- **FR-009**: System MUST detect and use appropriate parser based on file extension/content.
- **FR-010**: System MUST support cancellation of in-progress batch jobs.
- **FR-011**: System MUST output results in streaming mode for large batches.
- **FR-012**: System MUST report batch statistics: files processed, skipped, failed, duration.

### Key Entities

- **BatchJob**: A batch processing job; contains source paths, filters, options, and state.
- **BatchResult**: Aggregate result of batch processing; contains per-file results and summary.
- **FileEntry**: A file to be processed; contains path, size, detected format, and processing status.
- **BatchProgress**: Progress state; contains processed count, total count, current file, ETA.
- **BatchOptions**: Configuration for batch job; contains parallelism, filters, output options.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Directory with 10,000 files is scanned in under 10 minutes with parallel processing.
- **SC-002**: ZIP archives up to 1GB are processed without memory exhaustion.
- **SC-003**: File filtering reduces processing to only matching files with 100% accuracy.
- **SC-004**: Progress reporting updates at least every 1 second during active processing.
- **SC-005**: Aggregate reports correctly sum per-file findings.
- **SC-006**: Parallel processing achieves near-linear speedup up to 8 cores.

## Assumptions

- Supported file types are determined by installed parsers (txt, csv, json, html, pdf, docx, xlsx, pptx, eml, msg).
- Archive formats supported: ZIP initially; TAR, 7Z, RAR as future enhancements.
- Memory usage stays bounded regardless of batch size by streaming results.
- Batch jobs can be resumed after interruption via checkpoint files (future enhancement).
