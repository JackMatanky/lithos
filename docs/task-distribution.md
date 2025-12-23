# mise.toml vs justfile: Task Distribution Strategy

## Overview

After analyzing all tasks in both `mise.toml` and `justfile`, we've established a clear division of responsibilities:

- **`mise.toml`**: Simple, frequently-used tasks for everyday development
- **`justfile`**: Complex, advanced tasks with sophisticated logic and messaging

## Task Distribution Analysis

### ✅ **Fully Implemented in mise.toml** (Simple Tasks)

These tasks are simple enough to implement directly in mise.toml:

#### Build Tasks
- `build` - Basic Go build
- `dev` - Go build with race detection
- `build:all` - Cross-platform builds (basic)
- `build:verify` - Calls `just verify-build`

#### Test Tasks
- `test` - Basic `go test ./...`
- `test:v` - Verbose tests
- `test:cov` - HTML coverage report generation
- `test:int` - Integration tests
- `test:security` - Calls `just test-security`
- `test:compliance` - Calls `just test-compliance`
- `test:artifacts` - Calls `just test-artifacts` (enhanced coverage)
- `test:pkg` - Test specific package (basic)
- `test:pkg:enhanced` - Calls `just test-pkg` (with messaging)
- `bench` - Basic benchmarks
- `bench:enhanced` - Calls `just bench`

#### Quality Tasks
- `format` - Basic `golangci-lint fmt`
- `format:enhanced` - Calls `just fmt` (with validation)
- `lint` - Basic `golangci-lint run --fix`
- `lint:enhanced` - Calls `just lint` (with validation)
- `lint:report` - Calls `just lint-report` (conditional logic)
- `verify` - Depends on format + lint + test

#### Performance Tasks
- All `perf:*` tasks - Performance testing is core to this project

#### Maintenance Tasks
- `clean` - Basic cleanup
- `setup` - Basic `go mod download`
- `setup:enhanced` - Calls `just setup` (with version checks)
- `setup:pre-commit` - Calls `just setup-pre-commit`

### ✅ **Calls justfile Commands** (Complex Tasks)

These tasks delegate to justfile for complex functionality:

#### Why They Call justfile:
- **Complex logic**: Version checking, conditional builds, cross-platform detection
- **Enhanced messaging**: Progress indicators, colored output, structured logging
- **Advanced features**: Private helpers, groups, confirmation prompts, dotenv loading
- **Error handling**: Sophisticated error messages and remediation
- **Tool validation**: Dependency checking, external tool setup

#### Tasks That Call justfile:
- `build:all:enhanced` → `just build-all` (messaging + helpers)
- `build:verify` → `just verify-build` (complex validation)
- `test:security` → `just test-security` (tagged tests)
- `test:compliance` → `just test-compliance` (tagged tests)
- `test:artifacts` → `just test-artifacts` (enhanced coverage)
- `test:pkg:enhanced` → `just test-pkg` (with messaging)
- `bench:enhanced` → `just bench` (enhanced output)
- `format:enhanced` → `just fmt` (validation + messaging)
- `lint:enhanced` → `just lint` (validation + messaging)
- `lint:report` → `just lint-report` (conditional logic)
- `setup:enhanced` → `just setup` (version checks + messaging)
- `setup:pre-commit` → `just setup-pre-commit` (external tools)

## Decision Criteria

### ✅ **Implement in mise.toml If:**
- Simple command (1-3 lines)
- No complex logic or conditionals
- No external dependencies beyond basic tools
- Frequently used (everyday development)
- Benefits from mise's caching (`sources`/`outputs`)

### ✅ **Call justfile If:**
- Complex string manipulation (regex, conditionals)
- Cross-platform logic (`[unix]`/`[windows]`)
- Private helper functions
- Confirmation prompts (`[confirm]`)
- Groups and categorization (`[group]`)
- Advanced messaging (progress bars, colors)
- External tool setup (pre-commit, version checks)
- Sophisticated error handling

## Benefits of This Approach

### For mise.toml:
- **Fast parsing**: Smaller, simpler config
- **Caching**: Smart skipping of up-to-date tasks
- **Discovery**: `mise tasks` shows all available tasks
- **Tool management**: Automatic tool installation
- **Simple maintenance**: Easy to understand and modify

### For justfile:
- **Power**: Full scripting capabilities
- **Flexibility**: Complex logic and conditionals
- **Rich output**: Enhanced messaging and progress indicators
- **Reliability**: Battle-tested for complex workflows
- **Backwards compatibility**: Existing scripts continue to work

## Usage Examples

### Everyday Development (mise.toml)
```bash
# Quick development cycle
mise run verify          # Format + lint + test
mise run build           # Production binary
mise run test:cov        # Coverage report

# Performance testing
mise run perf:test       # Quick performance check
mise run perf:profile:all # Full profiling
```

### Advanced/Complex Tasks (justfile)
```bash
# Enhanced builds with messaging
mise run build:all:enhanced    # Cross-platform with progress
mise run build:verify          # Full build validation

# Enhanced testing with messaging
mise run test:artifacts        # Detailed coverage reports
mise run bench:enhanced        # Benchmarks with output

# Setup with validation
mise run setup:enhanced        # Full environment setup
mise run setup:pre-commit       # Git hooks installation
```

## Migration Notes

### From justfile to mise.toml:
- Simple tasks moved when they were frequently used
- Enhanced versions kept in justfile with `:enhanced` suffix
- Basic versions remain in mise.toml for speed

### Keeping in justfile:
- Tasks requiring complex logic stay in justfile
- Advanced features (groups, helpers, confirmation) preserved
- Backwards compatibility maintained

## Future Considerations

- **Monitor usage**: Move tasks to mise.toml if simple versions are used more
- **Add complexity gradually**: Only add justfile calls when needed
- **Document clearly**: Help shows both tool capabilities
- **Maintain balance**: Keep mise.toml focused on simple tasks

This distribution gives us the **best of both worlds**: fast, discoverable tasks in mise.toml for everyday use, and powerful, complex automation in justfile for advanced scenarios.</content>
<parameter name="filePath">/Users/jack/Documents/41_personal/lithos/docs/task-distribution.md
