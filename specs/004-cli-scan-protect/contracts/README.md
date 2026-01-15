# CLI Contracts

This directory contains behavioral contracts for the CLI commands. Contracts define the expected behavior, input/output formats, and guarantees for each command.

## Contract Files

1. **[scan-contract.md](./scan-contract.md)**: Defines the `veil scan` command behavior
2. **[protect-contract.md](./protect-contract.md)**: Defines the `veil protect` command behavior
3. **[policy-contract.md](./policy-contract.md)**: Defines the `veil policy` command behavior

## What is a Contract?

A contract is a formal specification of how a CLI command should behave. It includes:

- **Command signature**: Arguments and flags
- **Output format**: Text and JSON formats with examples
- **Behavioral contracts**: Specific scenarios and expected outcomes
- **Exit codes**: When to return 0, 1, or 2
- **Edge cases**: Handling of errors, empty inputs, etc.
- **Validation rules**: Input validation and error messages
- **Performance guarantees**: Expected execution times
- **Test implementations**: Example test cases

## Usage

### For Developers

When implementing a CLI command:

1. Read the contract file for that command
2. Implement according to the behavioral contracts (BC-*)
3. Handle all edge cases (EC-*)
4. Follow validation rules (VR-*)
5. Write tests based on test implementation examples

### For Testers

When testing a CLI command:

1. Use the examples section for manual testing
2. Verify all behavioral contracts are satisfied
3. Test all edge cases
4. Verify exit codes
5. Check output format matches specification

### For Users

When using the CLI:

1. Refer to the examples section for usage patterns
2. Check output contract for expected format
3. Use exit codes for scripting and automation

## Contract Versioning

Each contract file has a change log at the bottom tracking versions. When a contract changes:

1. Update the change log with date and description
2. Increment the version number
3. Update any dependent contracts
4. Notify stakeholders of breaking changes

## Testing Contracts

Contracts should have corresponding integration tests in `tests/integration/` and contract tests in `tests/contract/`:

```rust
// tests/contract/scan_contract.rs
#[test]
fn contract_scan_single_file() {
    // Test implementation from scan-contract.md
}
```

## References

- **Feature Spec**: [../spec.md](../spec.md)
- **Data Model**: [../data-model.md](../data-model.md)
- **Implementation Plan**: [../plan.md](../plan.md)
