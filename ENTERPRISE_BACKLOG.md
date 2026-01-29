# Enterprise Readiness Backlog (Post-Audit)

Last updated: 2026-01-29

This file tracks the remaining work to move Veil from "good" to "enterprise-grade".
Items here are intentionally operational/governance-heavy (things that don't naturally show up in `cargo test`).

## GitHub Governance (Manual Settings)

- [ ] Protect `main`:
  - [ ] Require PRs (disable direct pushes)
  - [ ] Require approvals (at least 1-2) and CODEOWNERS reviews
  - [ ] Require status checks: CI, Security, Coverage (and any future checks)
  - [ ] Require linear history (optional, but recommended)
  - [ ] Require signed commits (optional, but recommended)
- [ ] Enable Dependabot security updates (GitHub setting; currently separate from dependabot.yml)
- [ ] Turn on auto-merge (optional) for dependabot PRs once gates are green
- [ ] Add tag protection rules for release tags (e.g. `v*`)

## Supply Chain / Dependency Risk

- [x] Add `cargo vet` (or similar) to document third-party dependency review decisions
  - Note: current bootstrap uses `supply-chain/config.toml` exemptions; convert exemptions to real audits over time.
- [x] Reduce `cargo deny` duplicate-dependency warnings if feasible:
  - `html5ever` / `markup5ever` / `tendril` duplicates are currently pulled by different upstream crates (non-blocking)
- [ ] Consider pinning/monitoring high-risk parser dependencies (PDF/Office/email), and document update cadence

## Security Hardening

- [x] Document and enforce key management:
  - [x] How `VEIL_AUDIT_HMAC_KEY_HEX` is generated, stored, rotated (`SECURITY_GUIDE.md`)
  - [x] How encrypted audit logging keys are managed (if `veil-audit/encryption` is used) (`SECURITY_GUIDE.md`)
- [x] Add a short threat model (what Veil protects against / does not) (`THREAT_MODEL.md`)
- [x] Consider adding CodeQL/SAST (Rust support varies, but still useful for workflow/infra review)

## Testing & Quality Gates

- [ ] Expand mutation testing coverage beyond the current subset (trade-off: CI time)
- [x] Increase fuzzing depth/time for parser targets (weekly job currently runs short)
- [x] Add "golden file" integration tests for tricky real-world formats (PDF/Office/email edge cases)
- [ ] Decide and document required coverage target per crate (not just workspace-wide)

## Performance & Scalability

- [ ] Add criterion benchmarks for:
  - [ ] scan throughput (large text, large PDFs, many small files)
  - [x] detector hot paths (dictionary + regex detectors) (existing `veil-detect` benches)
  - [x] PDF parsing microbench (existing `veil-parsers` benches)
- [ ] Add perf regression monitoring strategy (periodic benchmark runs, stored baselines, or manual process)

## Ops / Compliance

- [x] Add an "Operations" doc:
  - [x] audit log retention/rotation recommendations (`OPERATIONS.md`)
  - [x] safe usage guidance for `--include-values` and JSON output handling (`OPERATIONS.md`)
  - [x] logging guidance (what should never be logged) (`OPERATIONS.md`)

## Handy Commands (Local)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-features -- -D warnings
cargo test --workspace --all-features
cargo deny check
cargo audit
cargo vet check
```
