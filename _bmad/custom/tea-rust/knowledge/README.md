# TEA Agent Knowledge Base: Rust Testing

This directory contains modular knowledge files for the TEA agent to reference when working with Rust testing in the Lithos project.

## Knowledge Module Index

### Test Types
| Module | Purpose | Load When |
|--------|---------|-----------|
| `testing-unit.md` | Unit testing patterns and location rules | Reviewing unit tests |
| `testing-integration.md` | Integration testing with I/O | Reviewing integration tests |
| `testing-e2e.md` | End-to-end CLI testing | Reviewing E2E tests |

### Code Quality
| Module | Purpose | Load When |
|--------|---------|-----------|
| `testing-naming.md` | Test naming conventions | Reviewing any tests |
| `testing-assertions.md` | Assertion patterns and messages | Reviewing assertions |
| `testing-fixtures.md` | Fixture strategies | Reviewing test setup |
| `testing-anti-patterns.md` | Patterns to detect and reject | Always load for test review |

### Tools
| Module | Purpose | Load When |
|--------|---------|-----------|
| `testing-tools-nextest.md` | Nextest configuration | Test runner questions |
| `testing-tools-rstest.md` | rstest patterns | Parameterized tests |
| `testing-tools-proptest.md` | Property-based testing | Edge case testing |

## Usage Guide

### For Test Review
When reviewing tests, load:
1. `testing-anti-patterns.md` (always)
2. `testing-naming.md` (for naming issues)
3. `testing-assertions.md` (for assertion issues)
4. Appropriate type module (unit/integration/e2e)

### For Test Generation
When generating tests, load all modules for comprehensive guidance.

### For Specific Questions
| Question | Load Module |
|----------|-------------|
| "Where should this test go?" | `testing-unit.md` (decision tree) |
| "Is this a good test name?" | `testing-naming.md` |
| "How should I assert this?" | `testing-assertions.md` |
| "What's wrong with this test?" | `testing-anti-patterns.md` |
| "How do I set up test data?" | `testing-fixtures.md` |
| "Should I use rstest?" | `testing-tools-rstest.md` |

## Structure of Each Module

Each knowledge module follows this structure:

1. **CONTEXT** - When does this apply?
2. **DECISION TREE** - Flowchart for decisions
3. **VALIDATION CHECKLIST** - Checklist of requirements
4. **ANTI-PATTERNS** - What to flag (with severity)
5. **CORRECT EXAMPLES** - Code examples showing best practices
6. **QUICK REFERENCE** - Summary tables

## Quick Decision Reference

```
What are you working on?
├── Reviewing unit tests?
│   └── → Load: anti-patterns, naming, assertions, unit
│
├── Reviewing integration tests?
│   └── → Load: anti-patterns, naming, assertions, integration
│
├── Reviewing E2E tests?
│   └── → Load: anti-patterns, naming, assertions, e2e
│
├── Generating new tests?
│   └── → Load: ALL modules
│
└── Specific tool question?
    └── → Load: appropriate tools module
```

## File Locations

- Knowledge modules: `_bmad/tea/knowledge/rust/`
- Agent customization: `_bmad/_config/agents/tea-tea.customize.yaml`
- Test documentation: `_bmad-output/test-developer-guide.md`
- Test strategy: `_bmad-output/test-design-system.md`

## Maintenance

When updating testing standards:
1. Update relevant knowledge module(s)
2. Update `_bmad-output/test-developer-guide.md` (human reference)
3. Update `_bmad-output/test-design-system.md` (strategy)
4. Ensure consistency across all files
