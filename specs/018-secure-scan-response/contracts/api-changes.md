# API Contract Changes: Secure Scan Response

## Breaking Change Notice

**Version**: 0.2.0
**Type**: Breaking change (response format)
**Severity**: High - existing integrations will need updates

## Summary

The scan API no longer returns PII values in responses by default. This is a security enhancement to prevent data exposure in logs, network monitoring, and debugging tools.

## API Changes

### POST /api/v1/scan

#### Request Changes

**New Query Parameter**:
```
include_values: boolean (optional, default: false)
```

**New Required Header** (when include_values=true):
```
X-Acknowledge-PII-Exposure: accepted
```

#### Response Changes

**Before** (v0.1.x):
```json
{
  "findings": [
    {
      "category": "email",
      "value": "john@example.com",    // ALWAYS present
      "confidence": 0.95,
      "start": 10,
      "end": 27
    }
  ]
}
```

**After** (v0.2.0) - Default:
```json
{
  "findings": [
    {
      "category": "email",
      "confidence": 0.95,
      "start": 10,
      "end": 27
      // "value" field is OMITTED
    }
  ]
}
```

**After** (v0.2.0) - With include_values=true + header:
```json
{
  "findings": [
    {
      "category": "email",
      "value": "john@example.com",    // Present when opted in
      "confidence": 0.95,
      "start": 10,
      "end": 27
    }
  ]
}
```

#### Error Response (Missing Acknowledgment)

**HTTP 400 Bad Request**:
```json
{
  "error": "pii_exposure_not_acknowledged",
  "message": "The include_values parameter requires explicit acknowledgment. Add header 'X-Acknowledge-PII-Exposure: accepted' to confirm you understand the security implications.",
  "code": "SECURITY_ACKNOWLEDGMENT_REQUIRED"
}
```

## CLI Changes

### veil scan

**New Flag**:
```
--include-values    Include matched PII values in output (requires confirmation)
```

**Behavior**:
- Interactive mode: Prompts for "yes" confirmation
- Non-interactive mode: Requires `--yes` flag

**Before**:
```
$ veil scan document.txt
Found 3 findings:
  [EMAIL] john@example.com at 10..27 (0.95)
```

**After** (default):
```
$ veil scan document.txt
Found 3 findings:
  [EMAIL] at 10..27 (confidence: 0.95)
```

**After** (with flag):
```
$ veil scan document.txt --include-values
WARNING: Including PII values exposes sensitive data in output.
Confirm you understand the security implications (yes/no): yes
Found 3 findings:
  [EMAIL] john@example.com at 10..27 (0.95)
```

## WASM/JavaScript Changes

### scan() Function

**New Options**:
```typescript
interface ScanOptions {
  // ... existing options
  includeValues?: boolean;       // default: false
  acknowledgeExposure?: boolean; // required if includeValues=true
}
```

**Before**:
```javascript
const result = await veil.scan(data);
console.log(result.findings[0].value); // "john@example.com"
```

**After** (default):
```javascript
const result = await veil.scan(data);
console.log(result.findings[0].value); // undefined
```

**After** (with acknowledgment):
```javascript
const result = await veil.scan(data, {
  includeValues: true,
  acknowledgeExposure: true
});
console.log(result.findings[0].value); // "john@example.com"
```

## Migration Guide

### Step 1: Update Your Client

If you currently rely on the `value` field:

1. Determine if you actually need the raw PII values
2. If yes, add the acknowledgment mechanism
3. If no, update your code to work without values

### Step 2: For API Clients

```python
# Before
response = requests.post(url, files={"file": f})

# After (if you need values)
response = requests.post(
    url,
    params={"include_values": "true"},
    headers={"X-Acknowledge-PII-Exposure": "accepted"},
    files={"file": f}
)
```

### Step 3: For CLI Scripts

```bash
# Before
veil scan document.txt --json | jq '.findings[].text'

# After (if you need values)
echo "yes" | veil scan document.txt --include-values --json | jq '.findings[].text'
# Or
veil scan document.txt --include-values --yes --json | jq '.findings[].text'
```

### Step 4: For JavaScript/WASM

```javascript
// Before
const result = await veil.scan(data);
const emails = result.findings
  .filter(f => f.category === "email")
  .map(f => f.value);

// After (if you need values)
const result = await veil.scan(data, {
  includeValues: true,
  acknowledgeExposure: true
});
const emails = result.findings
  .filter(f => f.category === "email")
  .map(f => f.value);

// After (recommended - use positions instead)
const result = await veil.scan(data);
const emailPositions = result.findings
  .filter(f => f.category === "email")
  .map(f => ({ start: f.start, end: f.end }));
```

## Security Rationale

This change addresses a critical data exposure vulnerability:

1. **Network Logs**: Proxies, load balancers, and monitoring tools may log response bodies
2. **Browser DevTools**: Console and Network tabs capture response data
3. **Error Tracking**: Services like Sentry may capture API responses
4. **Audit Trails**: Log aggregation systems may index response content

By omitting PII values by default, we ensure these systems never capture sensitive data.
