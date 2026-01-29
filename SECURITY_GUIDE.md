# Veil Security Guide

Veil is a local-first PII detection and redaction toolkit. This repository ships a CLI (`veil`) and reusable Rust crates; it does **not** include a long-running API server.

If you embed Veil in a networked service, you must implement your own authentication, authorization, CSRF/CORS protections, TLS, rate limiting, and request logging at that service layer.

## Trust Boundaries

- **Inputs are untrusted**: documents (PDF/Office), archives, and emails can be attacker-controlled.
- **Outputs are sensitive**: findings, redaction reports, and audit logs may contain metadata that should be access-controlled.

## Safety Defaults

- CLI output hides matched values by default; `scan --include-values` requires explicit opt-in.
- Directory scans avoid following symlinks to prevent recursion loops and out-of-tree traversal.
- Office parsers validate ZIP-based formats (DOCX/XLSX/PPTX) to mitigate ZIP bombs and path traversal; encrypted Office documents are rejected.

## Secret & Key Management

### Audit Log Integrity Key (Required)

`veil-audit` uses HMAC-SHA256 to provide tamper-evident audit logs. `AuditLogger::new(...)` requires an integrity key.

- Env var: `VEIL_AUDIT_HMAC_KEY_HEX`
- Format: **32 bytes** (64 hex characters)
- Example generation:
  - `openssl rand -hex 32`
  - `python -c "import secrets; print(secrets.token_hex(32))"`

Storage guidance:

- Store this key in your OS secrets manager (or an enterprise secrets manager) and inject it at runtime.
- Never commit it to git and never place it in `.env` files used by CI.
- Restrict who can read the key; treat it like an application signing key.

Rotation guidance:

- Plan rotation explicitly: audit log verification requires the key that was used to compute each entry's HMAC.
- When rotating, prefer starting a new audit log directory (or otherwise separating "epochs") and retaining old keys securely for verifying historical logs.
- Record the rotation time and which key applies to which audit log epoch in your secrets inventory.

### Redaction Verification Key (Optional)

`veil-redact::AppliedRedaction` stores a one-way hash of the original value (`original_hash`) to support verification without storing the raw PII.

- Env var: `VEIL_REDACTION_HMAC_KEY_HEX`
- Format: **32 bytes** (64 hex characters)
- Behavior:
  - If set: hashes are deterministic across processes (enables cross-run verification).
  - If not set: a per-process key is generated at runtime (prevents cross-run correlation, but `verify_original(...)` only works within the same process run).

### Encrypted Audit Logs (Optional)

If you enable the `veil-audit/encryption` feature and use `EncryptedAuditLogger`, the audit log file contents are encrypted at rest using AES-256-GCM.

- Key: **32 bytes** (256-bit AES key), passed via `EncryptionConfig::new(key_bytes)`
- Guidance:
  - Keep the audit HMAC key and the AES encryption key separate (independent rotation, independent access control).
  - Load keys from a secrets manager/KMS when possible, and ensure backups include a secure way to recover keys needed to decrypt historical logs.

## Threat Model

See `THREAT_MODEL.md` for the project's short threat model and explicitly out-of-scope scenarios.

## CI / Supply Chain Controls

- GitHub Actions are pinned to commit SHAs.
- Dependency scanning: `cargo audit` + `cargo deny check` + `cargo vet check`.
- Secret scanning: `detect-secrets` baseline (`.secrets.baseline`).

## High-Risk Dependencies (Parsers)

Veil parses attacker-controlled inputs (PDF/Office/email). These components are higher-risk than typical application dependencies and should be updated intentionally.

Current parser dependencies to monitor closely:

- PDF: `pdf-extract` (optional feature in `veil-parsers`)
- Office: `calamine`, `zip`, `quick-xml` (in `veil-office`)
- Email: `mailparse`, `msg_parser` (in `veil-email`)

Recommended update cadence:

- Review and apply parser/security updates on a fixed cadence (for example monthly) and immediately for relevant advisories/CVEs.
- Prefer small updates and review changelogs for parser crates; avoid large "bundle" upgrades without a test run against representative fixtures.
