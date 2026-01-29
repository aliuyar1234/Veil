# Veil Development Justfile
# Install just: cargo install just
# Run: just <recipe>

# Default recipe - show available commands
default:
    @just --list

# Build all crates
build:
    cargo build --workspace

# Build in release mode
release:
    cargo build --workspace --release

# Run all tests
test:
    cargo test --workspace --all-features

# Run tests with verbose output
test-verbose:
    cargo test --workspace --all-features -- --nocapture

# Run tests for a specific crate
test-crate crate:
    cargo test -p {{crate}}

# Run clippy lints
lint:
    cargo clippy --workspace --all-features -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run all checks (format, lint, test)
check: fmt-check lint test

# Generate code coverage report
coverage:
    cargo llvm-cov --workspace --all-features --html
    @echo "Coverage report: target/llvm-cov/html/index.html"

# Run mutation testing
mutants:
    cargo mutants --workspace -- --all-features

# Run mutation testing for a specific crate
mutants-crate crate:
    cargo mutants -p {{crate}} -- --all-features

# Run security audit
audit:
    cargo audit

# Run cargo-deny checks
deny:
    cargo deny check

# Run cargo-vet checks
vet:
    cargo vet check

# Build documentation
docs:
    cargo doc --workspace --no-deps --all-features
    @echo "Documentation: target/doc/veil_core/index.html"

# Open documentation in browser
docs-open: docs
    open target/doc/veil_core/index.html || xdg-open target/doc/veil_core/index.html

# Run benchmarks
bench:
    cargo bench --workspace

# Run benchmarks for a specific crate
bench-crate crate:
    cargo bench -p {{crate}}

# Run fuzzing (requires nightly)
fuzz target:
    cd fuzz && cargo +nightly fuzz run {{target}} -- -max_total_time=300

# List fuzz targets
fuzz-list:
    @ls fuzz/fuzz_targets/*.rs | xargs -I {} basename {} .rs

# Build WASM package
wasm:
    wasm-pack build crates/veil-wasm --target web

# Build WASM for Node.js
wasm-node:
    wasm-pack build crates/veil-wasm --target nodejs

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Run the CLI
cli *args:
    cargo run -p veil-cli -- {{args}}

# Install development tools
setup:
    cargo install cargo-watch cargo-audit cargo-deny cargo-llvm-cov cargo-mutants cargo-outdated cargo-vet
    cargo install wasm-pack
    pip install pre-commit
    pre-commit install
    @echo "Development tools installed!"

# Watch for changes and run tests
watch:
    cargo watch -x "test --workspace"

# Watch for changes and run clippy
watch-lint:
    cargo watch -x "clippy --workspace --all-features -- -D warnings"

# Generate SBOM
sbom:
    cargo sbom --output-format cyclonedx_json > sbom.json
    @echo "SBOM generated: sbom.json"

# Pre-release checklist
pre-release: check audit docs
    @echo "Pre-release checks passed!"

# Create a new release (dry run)
release-dry version:
    cargo release {{version}} --no-publish --no-push --no-tag

# Show workspace crates
crates:
    @cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name'
