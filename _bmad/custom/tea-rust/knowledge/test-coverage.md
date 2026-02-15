# TEA Knowledge: Coverage Analysis Tools

## CONTEXT

- **Applies to**: Test quality measurement
- **Purpose**: Identify untested code paths and meet quality gates (Target: 80%+)
- **Tools**: `cargo-llvm-cov`, `cargo-tarpaulin`

## TOOLS

### cargo-llvm-cov (Recommended)
Uses LLVM's source-based code coverage. Accurate and fast.

- **HTML Report**: `cargo llvm-cov --html --open`
- **LCOV (for CI)**: `cargo llvm-cov --lcov --output-path lcov.info`
- **Nextest Integration**: `cargo llvm-cov nextest`

### cargo-tarpaulin
Good cross-platform support and multiple output formats.

- **HTML Report**: `cargo tarpaulin --out Html`
- **LLVM Engine**: `cargo tarpaulin --engine llvm` (recommended for accuracy)

## BEST PRACTICES

1.  **Focus on meaningful coverage**: 100% line coverage doesn't guarantee correctness.
2.  **Cover critical paths**: Prioritize business logic and error handling.
3.  **Exclude generated code**: Use ignore patterns for proto/generated files.
4.  **Use coverage to find gaps**: Do not optimize blindly.

## VALIDATION CHECKLIST

- [ ] Coverage reports generated periodically
- [ ] Critical business logic has 100% path coverage
- [ ] Public APIs are fully exercised
- [ ] Error handling branches are tested (Target: 95%+)

## RELATED MODULES
- See `quality-gates.md` for specific thresholds
- See `ci.md` for pipeline integration
