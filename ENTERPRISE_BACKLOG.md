# Enterprise Readiness Backlog (Post-Audit)

Last updated: 2026-01-19

This file tracks the remaining work to move Veil from “good” to “enterprise-grade”.
Items here are intentionally operational/governance-heavy (things that don’t naturally show up in `cargo test`).

## GitHub Governance (Manual Settings)

- [ ] Protect `main`:
  - [ ] Require PRs (disable direct pushes)
  - [ ] Require approvals (at least 1–2) and CODEOWNERS reviews
  - [ ] Require status checks: CI, Security, Coverage (and any future checks)
  - [ ] Require linear history (optional, but recommended)
  - [ ] Require signed commits (optional, but recommended)
- [ ] Enable Dependabot security updates (GitHub setting; currently separate from dependabot.yml)
- [ ] Turn on auto-merge (optional) for dependabot PRs once gates are green
- [ ] Add tag protection rules for release tags (e.g. `v*`)

## Supply Chain / Dependency Risk

- [ ] Add `cargo vet` (or similar) to document third-party dependency review decisions
- [ ] Reduce `cargo deny` duplicate-dependency warnings if feasible:
  - `html5ever` / `markup5ever` / `tendril` duplicates are currently pulled by different upstream crates (non-blocking)
- [ ] Consider pinning/monitoring high-risk parser dependencies (PDF/Office/email), and document update cadence

## Security Hardening

- [ ] Document and enforce key management:
  - [ ] How `VEIL_AUDIT_HMAC_KEY_HEX` is generated, stored, rotated
  - [ ] How encrypted audit logging keys are managed (if `veil-audit/encryption` is used)
- [ ] Add a short threat model (what Veil protects against / does not)
- [ ] Consider adding CodeQL/SAST (Rust support varies, but still useful for workflow/infra review)

## Testing & Quality Gates

- [ ] Expand mutation testing coverage beyond the current subset (trade-off: CI time)
- [ ] Increase fuzzing depth/time for parser targets (weekly job currently runs short)
- [ ] Add “golden file” integration tests for tricky real-world formats (PDF/Office/email edge cases)
- [ ] Decide and document required coverage target per crate (not just workspace-wide)

## Performance & Scalability

- [ ] Add criterion benchmarks for:
  - scan throughput (large text, large PDFs, many small files)
  - detector hot paths (dictionary + regex detectors)
- [ ] Add perf regression monitoring strategy (periodic benchmark runs, stored baselines, or manual process)

## Ops / Compliance

- [ ] Add an “Operations” doc:
  - audit log retention/rotation recommendations
  - safe usage guidance for `--include-values` and JSON output handling
  - logging guidance (what should never be logged)

## Handy Commands (Local)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-features -- -D warnings
cargo test --workspace --all-features
cargo deny check
cargo audit
```
