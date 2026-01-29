# Veil Threat Model (Short)

This document describes what Veil is designed to protect against, and what is explicitly out of scope.

Veil is a **local-first** toolkit: it runs on the host you execute it on and does not provide a network API server.

## Assets

- Input documents (raw files, attachments, archives)
- Extracted text/segments and structured parse results
- Findings and redaction outputs (including reports)
- Audit logs
- Cryptographic keys (HMAC keys, encryption keys)

## Actors & Threats

- **Malicious input provider**: supplies crafted PDFs/Office docs/emails intended to crash Veil or exploit parser vulnerabilities.
- **Curious/unauthorized local user**: attempts to read outputs, logs, or keys stored on disk.
- **Operator error**: accidentally exposes PII via `--include-values`, logs, or JSON output handling.
- **Supply-chain attacker**: compromises a dependency or CI tooling.

## In Scope (What Veil Aims to Provide)

- **Local processing with safe defaults**: avoid network calls at runtime; avoid exposing matched values unless explicitly requested.
- **Defensive parsing posture**: treat inputs as untrusted and constrain common risks (e.g., size caps, zip validation, rejecting encrypted Office docs).
- **Tamper-evident audit logs**: hash-chained audit entries with optional encryption at rest.
- **Secure data handling primitives**: minimize accidental leakage via redacted `Display`/`Debug`/`Serialize` and zeroization where applicable.
- **Supply-chain hygiene**: automated checks (audit/deny/vet) and pinned CI actions.

## Out of Scope / Non-Goals

- **A compromised host**: if the OS/user account is fully compromised (malware, root/Administrator), Veil cannot protect secrets or outputs.
- **Network service security**: Veil does not ship an API server. If you embed it, your service must implement authn/authz, TLS, rate limiting, and request logging safely.
- **Perfect detection**: PII detection is probabilistic. False positives and false negatives are expected; policies should assume imperfect classification.
- **Preventing exfiltration by privileged operators**: if an operator can run Veil with `--include-values` and redirect output, Veil cannot stop them.

## Recommended Deployment Posture

- Run Veil in a sandboxed environment (container/VM) when processing untrusted documents at scale.
- Store keys in a secrets manager/KMS; avoid plaintext files and environment leakage.
- Treat scan outputs (even without raw values) as sensitive; restrict access and retention.
- Use audit logs with integrity keys, and verify chains before archiving/rotation.

