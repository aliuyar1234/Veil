# Zeroization Test Cases

## SensitiveString Tests

### Basic Functionality

| Test | Input | Expected | Notes |
|------|-------|----------|-------|
| Create from &str | `"secret"` | Contains "secret" | Basic construction |
| Create from String | `String::from("secret")` | Contains "secret" | String conversion |
| Deref to str | `&sensitive` | `&str` reference | Transparent string access |
| Clone | `sensitive.clone()` | New independent copy | Each copy zeroizes independently |
| Empty | `SensitiveString::empty()` | Contains "" | Empty string construction |
| Length | `"hello"` | `5` | Byte length |
| Is empty | `""` | `true` | Empty check |

### Zeroization Behavior

| Test | Scenario | Expected | Notes |
|------|----------|----------|-------|
| Drop zeros content | Create and drop | Memory contains zeros | Core security test |
| Clone zeros independently | Clone, drop original | Clone still valid, original zeroed | Independent lifecycle |
| Panic zeros content | Panic during use | Memory still zeroed | Drop runs on unwind |
| Into_inner transfers | `into_inner()` | Original consumed, string returned | Ownership transfer |

### Serialization

| Test | Input | Expected | Notes |
|------|-------|----------|-------|
| Serialize to JSON | `SensitiveString::new("test")` | `"test"` | Transparent serialization |
| Deserialize from JSON | `"test"` | `SensitiveString("test")` | Transparent deserialization |
| Debug output | Any value | `SensitiveString([REDACTED N bytes])` | No PII in debug |

### Edge Cases

| Test | Input | Expected | Notes |
|------|-------|----------|-------|
| Unicode content | `"日本語"` | Properly zeroed | Multi-byte characters |
| Large string | 1MB of data | Properly zeroed | Performance check |
| Empty string | `""` | No-op zeroization | Safe on empty |

## Finding Zeroization Tests

### Automatic Cleanup

| Test | Scenario | Expected |
|------|----------|----------|
| Single finding drop | Create Finding, let go out of scope | `matched_text` zeroed |
| Vec of findings drop | Create `Vec<Finding>`, drop | All `matched_text` fields zeroed |
| Finding in Result | `Ok(finding)` returned and dropped | `matched_text` zeroed |
| Finding clone | Clone finding, drop original | Original zeroed, clone valid |

### Integration with Detection

| Test | Scenario | Expected |
|------|----------|----------|
| Scan and cleanup | Scan document, return findings, drop | All PII values zeroed |
| Registry detect_all | Use registry, process results, cleanup | No PII in memory |

## TextSegment Zeroization Tests

### Automatic Cleanup

| Test | Scenario | Expected |
|------|----------|----------|
| Single segment drop | Create TextSegment, let go out of scope | `content` zeroed |
| ParseResult drop | Create ParseResult with segments, drop | All segment content zeroed |
| Segment in iterator | Iterate segments, drop after use | Each segment zeroed when done |

### Integration with Parsing

| Test | Scenario | Expected |
|------|----------|----------|
| Parse and cleanup | Parse document, process segments, drop | All content zeroed |
| Streaming parse | Stream chunks, process each, cleanup | Each chunk zeroed after processing |

## API Response Cleanup Tests

### Response Body Zeroization

| Test | Scenario | Expected |
|------|----------|----------|
| Scan endpoint | POST /scan, response sent | Response body zeroed |
| Error response | Error with PII context | Error details zeroed |
| Concurrent requests | Multiple requests | Each response independently zeroed |

## Performance Tests

### Overhead Measurement

| Test | Baseline | With Zeroization | Max Overhead |
|------|----------|------------------|--------------|
| 1000 small findings (50 bytes each) | X ms | Y ms | <5% |
| 100 large segments (10KB each) | X ms | Y ms | <5% |
| Full scan of 100-page document | X ms | Y ms | <5% |

## Platform-Specific Tests

### Cross-Platform Verification

| Platform | Test | Expected |
|----------|------|----------|
| Linux x86_64 | Basic zeroization | Memory zeroed |
| macOS arm64 | Basic zeroization | Memory zeroed |
| Windows x86_64 | Basic zeroization | Memory zeroed |
| WASM | Basic zeroization | Best-effort (may vary) |
