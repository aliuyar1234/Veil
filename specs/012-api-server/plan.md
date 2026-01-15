# Implementation Plan: API Server

**Branch**: `012-api-server` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/012-api-server/spec.md`

## Summary

Build a REST API server for Veil that provides HTTP endpoints for scanning and protecting documents. The API enables integration with external applications, automated workflows, and browser clients. It supports file upload via multipart/form-data, JWT authentication, rate limiting, async job processing for large files, webhook notifications, and comprehensive health monitoring. The server uses the same core detection and redaction logic as the CLI, ensuring consistency across all interfaces.

## Technical Context

**Language/Version**: Rust (stable, 1.75+)
**Primary Dependencies**: axum (web framework), tower (middleware), tokio (async runtime), serde (serialization), jsonwebtoken (JWT auth), tower-http (CORS/compression)
**Storage**: Temporary filesystem storage for async job results (configurable retention), JSONL for audit logs
**Testing**: cargo test (unit + integration tests), contract tests for API endpoints
**Target Platform**: Linux server (containerized with Docker), cross-platform compatible
**Project Type**: Web application (API server)
**Performance Goals**: <5s response for files <1MB, <100ms for health checks, handle 100 concurrent requests
**Constraints**: Configurable max file size (default 100MB), configurable rate limits (default 100 req/min), async processing for operations >30s
**Scale/Scope**: Single-server deployment initially, stateless for horizontal scaling

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | JWT authentication, rate limiting, input validation, no unsafe needed |
| II. Stability & Error Handling | ✅ PASS | Result types for all endpoints; graceful error responses with appropriate HTTP status codes |
| III. Performance | ✅ PASS | Async processing for large files; streaming file uploads; efficient middleware stack |
| IV. Simplicity & Minimalism | ✅ PASS | REST endpoints map directly to core Veil operations; minimal abstraction layers |
| V. Test-First Development | ✅ PASS | Contract tests for API endpoints; integration tests with real HTTP requests |
| VI. Dependency Discipline | ⚠️ REVIEW | axum (web framework), tower (middleware), jsonwebtoken needed for API functionality |
| VII. Rust Standards | ✅ PASS | Clippy/fmt; documented public API; idiomatic axum patterns |

**Gate Result**: PASS (dependencies justified for REST API and authentication)

## Project Structure

### Documentation (this feature)

```text
specs/012-api-server/
├── plan.md              # This file
├── research.md          # Phase 0 output (web frameworks, auth strategies, async patterns)
├── data-model.md        # Phase 1 output (API request/response models, auth tokens, jobs)
├── quickstart.md        # Phase 1 output (API setup, authentication, example requests)
├── contracts/           # Phase 1 output (OpenAPI spec, API contract tests)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
Cargo.toml               # Workspace root
crates/
├── veil-parsers/        # Existing: text parsing
├── veil-detect/         # Dependency: PII detection
├── veil-redact/         # Dependency: redaction engine
├── veil-policy/         # Dependency: policy management
├── veil-audit/          # Dependency: audit logging
└── veil-api/            # NEW: API server crate
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs           # Server entry point, configuration
    │   ├── lib.rs            # Library exports for testing
    │   ├── config.rs         # Server configuration (port, auth, limits)
    │   ├── error.rs          # API error types and HTTP status mapping
    │   ├── models/           # Request/response DTOs
    │   │   ├── mod.rs
    │   │   ├── scan.rs       # ScanRequest, ScanResponse
    │   │   ├── protect.rs    # ProtectRequest, ProtectResponse
    │   │   ├── job.rs        # AsyncJob, JobStatus, JobResult
    │   │   └── health.rs     # HealthStatus, ReadinessStatus
    │   ├── routes/           # API endpoints
    │   │   ├── mod.rs
    │   │   ├── scan.rs       # POST /api/v1/scan
    │   │   ├── protect.rs    # POST /api/v1/protect
    │   │   ├── jobs.rs       # GET /api/v1/jobs/{id}, GET /api/v1/jobs/{id}/result
    │   │   └── health.rs     # GET /health, /health/ready, /health/live
    │   ├── middleware/       # HTTP middleware
    │   │   ├── mod.rs
    │   │   ├── auth.rs       # JWT authentication
    │   │   ├── rate_limit.rs # Rate limiting
    │   │   ├── request_id.rs # Request ID generation/logging
    │   │   └── cors.rs       # CORS configuration
    │   ├── services/         # Business logic
    │   │   ├── mod.rs
    │   │   ├── scan_service.rs    # Scan orchestration
    │   │   ├── protect_service.rs # Protection orchestration
    │   │   ├── job_service.rs     # Async job management
    │   │   └── webhook_service.rs # Webhook notifications
    │   └── storage/          # Temporary file/job storage
    │       ├── mod.rs
    │       └── job_store.rs  # In-memory or file-based job storage
    └── tests/
        ├── fixtures/         # Test files
        ├── integration/      # HTTP integration tests
        │   ├── scan_tests.rs
        │   ├── protect_tests.rs
        │   ├── auth_tests.rs
        │   ├── rate_limit_tests.rs
        │   └── health_tests.rs
        └── contract/         # OpenAPI contract tests
            └── openapi_tests.rs

examples/
└── api-server/           # Example API usage
    ├── curl-examples.sh
    ├── python-client.py
    └── config.example.toml

docker/
└── api-server/
    ├── Dockerfile
    └── docker-compose.yml
```

**Structure Decision**: New `veil-api` crate for the REST API server. This keeps the API separate from core logic, allowing independent deployment and testing. The server depends on existing `veil-detect`, `veil-redact`, `veil-policy`, and `veil-audit` crates, ensuring consistent behavior with the CLI. The crate includes both a binary (`src/main.rs`) for running the server and a library (`src/lib.rs`) for testing.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| axum web framework | Modern async web framework with type-safe routing | actix-web is more complex; warp is less ergonomic; hand-rolled HTTP server is error-prone |
| tower middleware | Industry-standard middleware for logging, CORS, compression | Custom middleware would duplicate well-tested functionality |
| jsonwebtoken crate | RFC 7519 JWT validation is non-trivial | Hand-rolled JWT parsing is security-critical and error-prone |
| Async job storage | Large files require async processing to avoid blocking requests | Synchronous-only processing would timeout on large files |
| tokio runtime | Required for async HTTP server and concurrent request handling | Blocking server would not meet 100 concurrent request requirement |

## Post-Design Constitution Re-Check

*Re-evaluated after Phase 1 design completion*

| Principle | Status | Post-Design Notes |
|-----------|--------|-------------------|
| I. Security First | ✅ PASS | JWT validation, rate limiting per client, input size limits, audit logging, no unsafe code |
| II. Stability & Error Handling | ✅ PASS | All endpoints return Result; HTTP status codes map to error types; graceful degradation |
| III. Performance | ✅ PASS | Async/await throughout; streaming file uploads; configurable timeouts; horizontal scaling support |
| IV. Simplicity & Minimalism | ✅ PASS | Thin API layer over core Veil logic; no duplicate business logic; minimal state |
| V. Test-First Development | ✅ PASS | Contract tests for OpenAPI spec; integration tests for all endpoints; fixture files |
| VI. Dependency Discipline | ✅ PASS | 5 primary crates justified: axum, tower, tokio, serde, jsonwebtoken (all widely used) |
| VII. Rust Standards | ✅ PASS | Idiomatic axum handlers; proper error propagation; documented API models |

**Post-Design Gate Result**: PASS - Ready for task generation
