# Veil Operations Guide

This document covers operational guidance for running Veil in CI pipelines and enterprise environments.

## Handling Sensitive Outputs

- **Default output is safer**: `veil scan` hides matched values by default. Treat results as sensitive anyway (file paths, categories, positions can still reveal information).
- **Avoid `--include-values` in automation**: it can leak PII into CI logs, terminal scrollback, shell history, and log aggregation systems.
- If you must use `--include-values`, do so only in tightly controlled environments and prefer writing output to a protected file rather than stdout.

## JSON Output Handling

- `--json` output is intended for programmatic consumption. Store it as sensitive data.
- If you persist JSON output, apply access controls and retention policies equivalent to the input data’s classification.
- Prefer `--json` without `--include-values` for CI and batch pipelines; use categories/counts to gate builds (for example with `--fail-on-findings`).

## Audit Logs: Retention & Rotation

`veil-audit` writes one file per day:

- Plain: `audit-YYYY-MM-DD.jsonl`
- Encrypted (optional): `audit-YYYY-MM-DD.enc.jsonl`

Recommendations:

- Restrict filesystem permissions on the audit log directory (least privilege).
- Rotate and archive audit logs on a schedule aligned with your compliance requirements.
- Before archiving, consider verifying the hash chain and storing the verification result (plus the time and tool version used).
- Compress older logs and store them in a write-once or append-only location where feasible.

Key rotation note:

- Rotating the audit integrity key changes what key is needed for verification. Track which key applies to which audit-log “epoch” and retain keys securely for the retention period.

## Logging Guidance (Do Not Log)

When embedding Veil into other systems, avoid logging:

- Raw document contents
- Matched PII values (especially anything emitted by `--include-values`)
- Cryptographic keys (`VEIL_*` key env vars, key files, or KMS access tokens)
- Full unredacted output payloads

Prefer logging only high-level counters and summaries (e.g., number of findings, categories present, file size caps triggered), and ensure logs do not include sensitive file paths if that matters in your environment.

