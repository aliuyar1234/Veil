# Feature Specification: Email Parser

**Feature Branch**: `007-email-parser`
**Created**: 2025-12-08
**Status**: Draft
**Input**: EML and MSG email parsing for PII detection

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Extract Email Headers (Priority: P1)

A compliance officer scans email files and the system extracts header fields (From, To, CC, BCC, Subject, Date) which commonly contain PII like names and email addresses.

**Why this priority**: Email headers contain high-density PII and are structured, making them ideal for targeted detection.

**Independent Test**: Provide email with various headers, extract, verify all header fields captured with labels.

**Acceptance Scenarios**:

1. **Given** an EML file, **When** parsed, **Then** From, To, CC, Subject, Date headers are extracted.
2. **Given** multiple recipients in To/CC, **When** parsed, **Then** each recipient is extracted separately.
3. **Given** display names in headers (e.g., `"John Doe" <john@example.com>`), **When** parsed, **Then** both name and email are captured.

---

### User Story 2 - Extract Email Body Text (Priority: P1)

A privacy analyst scans emails for PII in the message body. The system extracts both plain text and HTML body content, converting HTML to readable text.

**Why this priority**: Email bodies contain the primary message content where PII discussions and data sharing occur.

**Independent Test**: Provide email with text and HTML parts, extract, verify both are captured as text.

**Acceptance Scenarios**:

1. **Given** email with plain text body, **When** parsed, **Then** body text is extracted as-is.
2. **Given** email with HTML body only, **When** parsed, **Then** HTML is converted to plain text.
3. **Given** email with both text and HTML parts (multipart/alternative), **When** parsed, **Then** plain text part is preferred, HTML available as fallback.

---

### User Story 3 - List Email Attachments (Priority: P2)

A security team needs to identify email attachments that may contain PII. The system lists all attachments with filename, size, and MIME type for further processing.

**Why this priority**: Attachments often contain the actual sensitive documents being shared via email.

**Independent Test**: Provide email with attachments, verify attachment metadata is listed correctly.

**Acceptance Scenarios**:

1. **Given** email with PDF attachment, **When** parsed, **Then** attachment listed with filename, size, type.
2. **Given** email with multiple attachments, **When** parsed, **Then** all attachments are listed.
3. **Given** inline images, **When** parsed, **Then** images are listed separately from regular attachments.

---

### User Story 4 - Parse MSG Files (Priority: P2)

A Windows user exports emails from Outlook as MSG files. The system parses the proprietary MSG format and extracts the same information as EML files.

**Why this priority**: MSG is the default export format from Microsoft Outlook, widely used in enterprises.

**Independent Test**: Provide MSG file, extract, verify headers and body match original email.

**Acceptance Scenarios**:

1. **Given** an MSG file, **When** parsed, **Then** same headers and body extracted as EML.
2. **Given** MSG with attachments, **When** parsed, **Then** attachments are listed.
3. **Given** MSG with embedded images, **When** parsed, **Then** images are identified.

---

### User Story 5 - Handle Email Threads (Priority: P3)

A compliance team scans email threads where the body contains quoted previous messages. The system identifies and separates quoted content from new content.

**Why this priority**: Email threads contain historical PII that may have different handling requirements.

**Independent Test**: Provide email with quoted replies, verify quoted sections are identified.

**Acceptance Scenarios**:

1. **Given** email with `>` quoted text, **When** parsed, **Then** quoted sections are marked as such.
2. **Given** email with `On <date>, <person> wrote:` pattern, **When** parsed, **Then** reply boundary is detected.
3. **Given** complex nested quotes, **When** parsed, **Then** quote levels are preserved or flattened with metadata.

---

### Edge Cases

- What happens with encrypted emails (S/MIME, PGP)? System reports email is encrypted and body cannot be extracted.
- What happens with malformed headers? System extracts available data and logs warning.
- What happens with non-standard character encodings? System attempts detection and conversion to UTF-8.
- What happens with extremely large attachments? System lists attachment metadata without loading content into memory.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse EML (RFC 5322) email files.
- **FR-002**: System MUST parse MSG (Microsoft Outlook) email files.
- **FR-003**: System MUST extract standard headers: From, To, CC, BCC, Subject, Date, Message-ID.
- **FR-004**: System MUST parse display names and email addresses from header fields.
- **FR-005**: System MUST extract plain text body content.
- **FR-006**: System MUST convert HTML body content to plain text.
- **FR-007**: System MUST list attachments with filename, size, and MIME type.
- **FR-008**: System MUST handle multipart MIME messages correctly.
- **FR-009**: System MUST identify quoted/replied content in email bodies.
- **FR-010**: System MUST handle character encoding detection and conversion.
- **FR-011**: System MUST report encrypted emails as partially processable (headers only).
- **FR-012**: System MUST output TextSegments compatible with parser interface (Spec 001).

### Key Entities

- **EmailMessage**: A parsed email; contains headers, body parts, and attachments.
- **EmailHeader**: A header field; contains name (From, To, etc.) and parsed value(s).
- **EmailAddress**: A parsed email address; contains display name (optional) and address.
- **EmailBody**: Body content; contains content type, text content, and whether it's quoted.
- **EmailAttachment**: Attachment metadata; contains filename, size, MIME type, and content ID.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: EML files parse with 100% header extraction accuracy for standard headers.
- **SC-002**: MSG files parse with same accuracy as EML for equivalent content.
- **SC-003**: HTML-to-text conversion preserves readable content without HTML artifacts.
- **SC-004**: Emails with 10+ attachments are parsed in under 1 second (metadata only).
- **SC-005**: Character encoding is correctly handled for 99% of emails.
- **SC-006**: Quoted content detection works for common reply patterns (>, On...wrote:).

## Assumptions

- Attachment content is not automatically extracted for PII scanning; that requires calling the appropriate parser (PDF, Office, etc.).
- The system focuses on metadata and body text; calendar invites and other special content types are treated as attachments.
- MSG parsing handles the OLE compound document format used by Outlook.
- Email threading/conversation detection is best-effort; complex threads may not be perfectly segmented.
