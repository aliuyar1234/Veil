# Data Model: Codebase Excellence Initiative

**Branch**: `022-codebase-improvements` | **Date**: 2025-12-18

## New Entities

### 1. EncryptedVault

Secure storage for token-to-original mappings with encryption at rest.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| version | u8 | Vault format version (currently 1) |
| key_id | String | Identifier for the KEK used |
| encrypted_dek | Vec<u8> | Data Encryption Key encrypted with KEK |
| nonce | [u8; 12] | AES-GCM nonce for DEK encryption |
| entries | Vec<EncryptedEntry> | Encrypted token mappings |

**Relationships**:
- Uses KeyProvider to obtain KEK
- Replaces plaintext FileVault

**Validation Rules**:
- key_id must be non-empty
- encrypted_dek must be 32 bytes (when decrypted)
- nonce must be unique per encryption operation

---

### 2. KeyProvider (Trait)

Abstraction for key management backends.

**Methods**:
| Method | Signature | Description |
|--------|-----------|-------------|
| get_key | fn(&self, key_id: &str) -> Result<SecretKey> | Retrieve encryption key |
| rotate_key | fn(&self, key_id: &str) -> Result<SecretKey> | Generate new key version |
| list_keys | fn(&self) -> Result<Vec<KeyMetadata>> | List available keys |

**Implementations**:
- LocalKeyProvider: File-based key storage
- EnvKeyProvider: Environment variable keys
- (Future) AwsKmsProvider, AzureKeyVaultProvider

---

### 3. StreamingParserConfig

Configuration for memory-bounded streaming parsers.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| memory_threshold | usize | Bytes before streaming activates |
| chunk_size | usize | Streaming chunk size |
| max_memory | usize | Hard memory limit |
| timeout | Duration | Per-chunk timeout |

**Defaults**:
- memory_threshold: 10MB
- chunk_size: 64KB
- max_memory: 200MB
- timeout: 30s

---

### 4. BatchRequest

Request for batch file processing.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| files | Vec<BatchFile> | Files to process |
| options | ScanOptions | Shared scan options |
| fail_fast | bool | Stop on first error |
| parallel | bool | Process files in parallel |

**BatchFile**:
| Field | Type | Description |
|-------|------|-------------|
| name | String | Filename for format detection |
| content | Vec<u8> | File contents |
| options | Option<ScanOptions> | Per-file overrides |

---

### 5. BenchmarkResult

Output from criterion benchmarks.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| name | String | Benchmark name |
| mean | Duration | Mean execution time |
| std_dev | Duration | Standard deviation |
| throughput | Option<f64> | Items/second if applicable |
| comparison | Option<Comparison> | vs baseline if available |

---

## Modified Entities

### SensitiveString (veil-core)

**Additions**:
- Implement PartialEq with constant-time comparison
- Implement Clone with explicit zeroization of source
- Add serialization tests

### Finding (veil-detect)

**Additions**:
- Add benchmark_id: Option<String> for performance tracking

### ParseResult (veil-parsers)

**Additions**:
- Add memory_used: usize for memory tracking
- Add streaming_used: bool flag

---

## State Transitions

### Key Rotation State Machine

```
[Active] --rotate--> [Rotating]
                         |
    +--------------------+--------------------+
    |                                         |
    v                                         v
[Re-encrypting] --complete--> [Active]    [Failed]
                                              |
                                              v
                                          [Rollback] --recover--> [Active]
```

## Index Strategy

This is primarily a library crate, not a database application. No database indexes required.

For in-memory caching:
- LRU cache for compiled regex patterns (keyed by pattern string)
