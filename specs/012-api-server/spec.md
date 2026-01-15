# Feature Specification: API Server

**Feature Branch**: `012-api-server`
**Created**: 2025-12-08
**Status**: Draft
**Input**: REST API for integration with external systems

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scan Document via API (Priority: P1)

A developer integrates Veil into their application by calling the REST API to scan documents for PII. They POST a file and receive findings as JSON response.

**Why this priority**: API integration enables Veil to be embedded in automated workflows and other applications.

**Independent Test**: POST file to scan endpoint, verify JSON response with findings.

**Acceptance Scenarios**:

1. **Given** POST to `/api/v1/scan` with file, **When** processed, **Then** JSON response with findings returned.
2. **Given** large file upload, **When** scanned, **Then** response includes all findings with positions.
3. **Given** unsupported file type, **When** submitted, **Then** appropriate error response returned.

---

### User Story 2 - Protect Document via API (Priority: P1)

A developer calls the API to redact a document and receives the protected version. They POST original file and policy, receive redacted file in response.

**Why this priority**: Automated protection is essential for pipeline integration.

**Independent Test**: POST file to protect endpoint, verify redacted file returned.

**Acceptance Scenarios**:

1. **Given** POST to `/api/v1/protect` with file and policy, **When** processed, **Then** redacted file returned.
2. **Given** JSON policy in request body, **When** processed, **Then** protection rules applied.
3. **Given** `Accept: application/json` header, **When** protecting, **Then** findings + download URL returned instead of file.

---

### User Story 3 - Authenticate API Requests (Priority: P1)

An administrator configures API authentication so only authorized clients can access the service. The system supports JWT tokens for authentication.

**Why this priority**: Security is mandatory for any API handling sensitive data.

**Independent Test**: Make request without token, verify rejected; with valid token, verify accepted.

**Acceptance Scenarios**:

1. **Given** request without Authorization header, **When** sent, **Then** 401 Unauthorized returned.
2. **Given** valid JWT token, **When** included in header, **Then** request processed.
3. **Given** expired token, **When** used, **Then** 401 with "token expired" message.

---

### User Story 4 - Apply Rate Limiting (Priority: P2)

An administrator configures rate limits to prevent abuse. The system enforces request limits per client/token.

**Why this priority**: Rate limiting protects the service from overload and abuse.

**Independent Test**: Exceed rate limit, verify requests rejected with 429 status.

**Acceptance Scenarios**:

1. **Given** 100 requests/minute limit, **When** 101st request sent, **Then** 429 Too Many Requests.
2. **Given** rate limit headers, **When** response received, **Then** includes X-RateLimit-Remaining.
3. **Given** different clients, **When** both active, **Then** each has independent limits.

---

### User Story 5 - Receive Webhook Notifications (Priority: P3)

A developer configures webhooks to be notified when async operations complete. The system POSTs results to registered webhook URLs.

**Why this priority**: Webhooks enable async processing for large files without polling.

**Independent Test**: Register webhook, submit async job, verify webhook called on completion.

**Acceptance Scenarios**:

1. **Given** webhook URL registered, **When** async scan completes, **Then** POST sent to webhook.
2. **Given** webhook payload, **When** received, **Then** includes job ID, status, findings summary.
3. **Given** webhook failure, **When** delivery fails, **Then** retry with exponential backoff.

---

### User Story 6 - Check Service Health (Priority: P1)

An operations team monitors the API health. The system provides health check endpoints for load balancers and monitoring systems.

**Why this priority**: Health checks are essential for production deployment and monitoring.

**Independent Test**: Call health endpoint, verify 200 OK response with status.

**Acceptance Scenarios**:

1. **Given** GET `/health`, **When** service healthy, **Then** 200 OK with `{"status": "healthy"}`.
2. **Given** GET `/health/ready`, **When** all dependencies available, **Then** 200 OK.
3. **Given** dependency unavailable, **When** readiness checked, **Then** 503 Service Unavailable.

---

### Edge Cases

- What happens with very large file uploads? System enforces configurable max file size (default 100MB).
- What happens when processing takes too long? System supports async processing with job IDs for long operations.
- What happens with malformed requests? System returns 400 Bad Request with validation errors.
- What happens during server restart? In-flight requests are lost; clients should retry.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide REST API endpoints for scan operations.
- **FR-002**: System MUST provide REST API endpoints for protect operations.
- **FR-003**: System MUST support file upload via multipart/form-data.
- **FR-004**: System MUST support JSON request/response bodies.
- **FR-005**: System MUST authenticate requests using JWT tokens.
- **FR-006**: System MUST enforce configurable rate limits per client.
- **FR-007**: System MUST return appropriate HTTP status codes (200, 400, 401, 403, 429, 500).
- **FR-008**: System MUST provide health check endpoints (/health, /health/ready, /health/live).
- **FR-009**: System MUST support async processing with job status polling.
- **FR-010**: System MUST support webhook notifications for async job completion.
- **FR-011**: System MUST include request ID in all responses for tracing.
- **FR-012**: System MUST log all API requests for audit purposes.
- **FR-013**: System MUST support CORS configuration for browser clients.
- **FR-014**: System MUST provide OpenAPI/Swagger specification.

### Key Entities

- **ApiRequest**: Incoming HTTP request; includes method, path, headers, body, client identity.
- **ApiResponse**: Outgoing HTTP response; includes status, headers, body.
- **AuthToken**: JWT authentication token; contains client ID, permissions, expiration.
- **RateLimitConfig**: Rate limiting settings; includes requests per period, period duration.
- **AsyncJob**: A long-running operation; has ID, status, result when complete.
- **WebhookConfig**: Webhook settings; includes URL, secret for signature, retry policy.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: API responds to scan requests in under 5 seconds for files under 1MB.
- **SC-002**: Authentication correctly rejects 100% of invalid/expired tokens.
- **SC-003**: Rate limiting enforces limits with less than 1% variance.
- **SC-004**: Health endpoints respond in under 100ms.
- **SC-005**: API handles 100 concurrent requests without errors.
- **SC-006**: OpenAPI spec is complete and passes validation.

## Assumptions

- The API server is deployed behind a reverse proxy (nginx, etc.) that handles TLS termination.
- JWT tokens are issued by an external identity provider; the API validates but does not issue tokens.
- File storage for async jobs is temporary; results are deleted after configurable retention period.
- The API uses the same core logic as the CLI; no duplicate implementation.

## API Endpoints Summary

| Method | Path | Description |
|--------|------|-------------|
| POST | /api/v1/scan | Scan file for PII |
| POST | /api/v1/protect | Protect file with redaction |
| GET | /api/v1/jobs/{id} | Get async job status |
| GET | /api/v1/jobs/{id}/result | Download job result |
| GET | /health | Basic health check |
| GET | /health/ready | Readiness probe |
| GET | /health/live | Liveness probe |
