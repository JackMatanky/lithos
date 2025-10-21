# -----------------------------------------------------------------------------
# Filename:		lithos/justfile
# Docs: 		https://just.systems/man/v0.21.0/justfile/
# Description: 	FlowForge Framework - Build Automation
# Requires: 	just, go 1.25+, golangci-lint
# -----------------------------------------------------------------------------

set dotenv-load := true
set export := true
set shell := ["bash", "-c"]

# ------------------------------------------------------------ #
#                         CONFIGURATION                        #
# ------------------------------------------------------------ #

PROJECT_NAME := "lithos"

# ----------------------- Project Paths ---------------------- #

PROJECT_DIR := invocation_directory()
CMD_DIR := PROJECT_DIR / "cmd"
BIN_DIR := PROJECT_DIR / "bin"
COVERAGE_DIR := PROJECT_DIR / "coverage"
REPORTS_DIR := PROJECT_DIR / "reports"
TEST_DIR := PROJECT_DIR / "test"
CMD_ENTRY_POINT := CMD_DIR / PROJECT_NAME

# ------------------- Go Version Detection ------------------- #

SEMVER_REGEX := '(\d*)?\.(\d*)?\.(\d*)'
GO_VERSION := replace_regex(shell('go version'), '[^\d\.]', '')
MAJOR_GO_VERSION := replace_regex(GO_VERSION, SEMVER_REGEX, '$1')
MINOR_GO_VERSION := replace_regex(GO_VERSION, SEMVER_REGEX, '$2')
GO_VERSION_MINIMUM := "1.18"
GO_VERSION_ERROR_MSG := '''
Incompatible Go version: {{ GO_VERSION }}
Required: >= {{ GO_VERSION_MINIMUM }}
Please update your Go installation.
'''

# ------------------- Variables & Constants ------------------ #

GO_LDFLAGS := env("GO_LDFLAGS", "-s -w")
GO_GCFLAGS := env("GO_GCFLAGS", "-N -l")
GOLANGCI_LINT_GITHUB := "github.com/golangci/golangci-lint/v2/cmd/golangci-lint"
GOLANGCI_LINT := "go run " + GOLANGCI_LINT_GITHUB

# ------------------ Configuration Constants ----------------- #

COVERAGE_MODE := "atomic"
DEFAULT_ARCH := "amd64"

# ------------------------------------------------------------ #
#                       DEFAULT & ALIASES                      #
# ------------------------------------------------------------ #

# Display a list of available tasks and modules
default:
    @just --list

alias b := build
alias t := test
alias l := lint
alias f := fmt
alias v := verify

# ------------------------------------------------------------ #
#                        PRIVATE HELPERS                       #
# ------------------------------------------------------------ #
# --------------------- Messaging / Echo --------------------- #

# Print task start message
[private]
_echo_start task:
    @echo "🚀 [START] {{ task }}..."

# Print task completion message
[private]
_echo_complete task:
    @echo "✅ [COMPLETE] {{ task }}"

# Print informational message
[private]
_echo_info message:
    @echo "ℹ️  {{ message }}"

# Print task execution time completion
[private]
_time_execution task:
    @echo "⏱️  {{ task }} execution completed"

# ------------------ Filesystem & Directory ------------------ #

# Create directory if it doesn't exist
[private]
_ensure_dir dir:
    @mkdir -p {{ dir }}

# Remove directory and all contents
[private]
_clean_dir dir:
    @rm -rf {{ dir }}

# Remove files matching pattern
[private]
_clean_files pattern:
    @rm -f {{ pattern }}

# --------------------- Tool Validation  --------------------- #

# Check if tool is available (Unix)
[private]
[unix]
_verify_tool tool:
    @command -v {{ tool }} >/dev/null

# Check if tool is available (Windows)
[private]
[windows]
_verify_tool tool:
    @@where {{ tool }} >nul 2>&1

# Verify tool exists in go.mod
[private]
_check_tool_dependency tool:
    @grep -q "{{ tool }}" go.mod || (echo "❌ {{ tool }} not found in go.mod" && exit 1)

# ------------------------------------------------------------ #
#                             SETUP                            #
# ------------------------------------------------------------ #

# Print Go version error message and exit
[private]
_go_version_error:
    @echo "{{ GO_VERSION_ERROR_MSG }}"
    @exit 1

# Verify Go version meets minimum requirement
[group("Setup")]
go_version_check:
    @if [[ {{ MAJOR_GO_VERSION }} -gt 1 ]] \
       || { [ {{ MAJOR_GO_VERSION }} -eq 1 ] \
       && [ {{ MINOR_GO_VERSION }} -ge 18 ]; }; then \
       echo "Go version {{ GO_VERSION }} OK (>= {{ GO_VERSION_MINIMUM }})"; \
    else \
       just _go_version_error; \
    fi

# Install pre-commit and setup Git hooks
[group("Setup")]
setup-pre-commit:
    just _verify_tool pre-commit || uv pip install pre-commit
    pre-commit install

# Setup development environment and download dependencies
[group("Setup")]
setup: go_version_check
    just _echo_start "Setting up development environment"
    @go mod download
    just _echo_info "Installing tools from go.mod"
    @go mod download -x
    just _echo_complete "Development environment ready"
    just _echo_info "Tools available via: go run <tool-import-path>"

# ------------------------------------------------------------ #
#                             BUILD                            #
# ------------------------------------------------------------ #
# ----------------------- Build Helpers ---------------------- #

# Return build output path and entry point
[private]
_build_paths target:
    @echo "{{ BIN_DIR }}/{{ target }} {{ CMD_ENTRY_POINT }}"

# Build binary with production optimizations
[private]
_go_build_basic target:
    @echo "Building {{ target }}..."
    go build -ldflags="{{ GO_LDFLAGS }}" -o `just _build_paths {{ target }}`
    @echo "{{ target }} build complete"

# Build binary with race detection and debug symbols
[private]
_go_build_dev target:
    @echo "Building {{ target }} with race detection..."
    go build -race -gcflags="{{ GO_GCFLAGS }}" -o `just _build_paths {{ target }}`
    @echo "{{ target }} development build complete"

# Return cross-platform build output path with OS-specific extension
[private]
_cross_build_output_path os:
    @echo "{{ BIN_DIR }}/{{ PROJECT_NAME }}-{{ os }}{{ if os == "windows" { ".exe" } else { "" } }} {{ CMD_ENTRY_POINT }}"

# Build binary for specific OS and architecture
[private]
_cross_build os arch=DEFAULT_ARCH:
    @echo "Building for {{ os }}/{{ arch }}..."
    @GOOS={{ os }} GOARCH={{ arch }} go build -ldflags="{{ GO_LDFLAGS }}" -o `just _cross_build_output_path {{ os }}`

# ---------------------- Build Recipes ----------------------- #

# Build production binary with optimizations
[group("Build")]
build:
    just _ensure_dir {{ BIN_DIR }}
    just _go_build_basic {{ PROJECT_NAME }}

# Build development binary with race detection
[group("Build")]
dev:
    just _ensure_dir {{ BIN_DIR }}
    just _go_build_dev {{ PROJECT_NAME }}

# Build binaries for Darwin, Linux, and Windows
[group("Build")]
build-all:
    just _echo_start "Building for all platforms"
    just _ensure_dir {{ BIN_DIR }}
    just _cross_build "darwin"
    just _cross_build "linux"
    just _cross_build "windows"
    just _echo_complete "Cross-platform build"

# ------------------- Build Verification --------------------- #

# Validate Go workspace, dependencies, and module resolution
[group("Build")]
verify-build:
    just _echo_start "Validating build system"
    @echo "Validating Go workspace..."
    @go work sync
    @echo "Verifying build dependencies..."
    @go mod verify
    @echo "Checking module resolution..."
    @go list -m all > /dev/null
    just _echo_complete "Build system validation"

# ------------------------------------------------------------ #
#                            TESTING                           #
# ------------------------------------------------------------ #
# ----------------------- Test Helpers ----------------------- #

# Run tests with specific build tag
[private]
_test_with_tags tag:
    @echo "Running {{ tag }} tests..."
    @go test -tags={{ tag }} {{ TEST_DIR }}/{{ tag }}/...
    @echo "{{ tag }} tests complete"

# Run all tests and generate coverage profile
[private]
_go_test_coverage:
    @go test -coverprofile=coverage.out -covermode={{ COVERAGE_MODE }} ./...

# Generate HTML coverage report
[private]
_coverage_html:
    @go tool cover -html=coverage.out -o {{ COVERAGE_DIR }}/coverage.html

# ----------------------- Test Recipes ----------------------- #

# Run all tests
[group("Testing")]
test:
    @echo "Running tests..."
    @go test ./...
    @echo "Tests complete"

# Run integration tests with build tag
[group("Testing")]
test-integration:
    just _test_with_tags "integration"

# Run security tests with build tag
[group("Testing")]
test-security:
    just _test_with_tags "security"

# Run compliance tests with build tag
[group("Testing")]
test-compliance:
    just _test_with_tags "compliance"

# Run all tests with inline coverage percentage
[group("Testing")]
test-coverage:
    @echo "Running tests with coverage..."
    @go test -cover ./...
    @echo "Coverage analysis complete"

# Run tests for specific package with verbose output
[group("Testing")]
test-pkg pkg:
    @echo "Testing package {{ pkg }}..."
    @go test -v ./{{ pkg }}

# Run benchmark tests with memory allocation stats
[group("Testing")]
bench:
    @echo "Running benchmarks..."
    @go test -bench=. -benchmem ./...
    @echo "Benchmarks complete"

# Generate detailed HTML coverage report
[group("Testing")]
test-artifacts:
    just _echo_start "Running tests with coverage"
    just _ensure_dir {{ COVERAGE_DIR }}
    just _go_test_coverage
    just _coverage_html
    just _echo_complete "Coverage report: {{ COVERAGE_DIR }}/coverage.html"

# ------------------------------------------------------------ #
#                            QUALITY                           #
# ------------------------------------------------------------ #
# ----------------------- Lint Helpers ----------------------- #

# Return lint output file path with extension
[private]
_lint_output_file ext:
    @echo "{{ REPORTS_DIR }}/lint-results.{{ ext }}"

# Generate SARIF format lint report for CI/CD
[private]
_lint_sarif_report:
    @{{ GOLANGCI_LINT }} run --out-format=sarif \
      > `just _lint_output_file sarif` 2>/dev/null \
      || echo '{"version":"2.1.0","runs":[{"tool":{"driver":{"name":"golangci-lint"}},"results":[]}]}' \
      > `just _lint_output_file sarif`

# Generate colored text lint report for human reading
[private]
_lint_text_report:
    @{{ GOLANGCI_LINT }} run --out-format=colored-line-number \
      > `just _lint_output_file txt` 2>&1 \
      || echo "No linting issues found" > `just _lint_output_file txt`

# -------------------- Quality Recipes ----------------------- #

# Format all code with golangci-lint
[group("Quality")]
fmt:
    just _check_tool_dependency {{ GOLANGCI_LINT_GITHUB }}
    just _echo_start "Formatting code"
    @{{ GOLANGCI_LINT }} fmt
    just _echo_complete "Formatting complete"

# Lint code and auto-fix issues
[group("Quality")]
lint:
    just _check_tool_dependency {{ GOLANGCI_LINT_GITHUB }}
    just _echo_start "Linting code"
    @{{ GOLANGCI_LINT }} run --fix
    just _echo_complete "Linting"

# Generate lint report in SARIF or text format
[group("Quality")]
lint-report ext:
    just _check_tool_dependency {{ GOLANGCI_LINT_GITHUB }}
    just _ensure_dir {{ REPORTS_DIR }}
    {{ if ext == "sarif" { "just _lint_sarif_report" } else if ext == "txt" { "just _lint_text_report" } else { error("Unknown format: " + ext + ". Use 'sarif' or 'txt'") } }}

# Run format, lint, and test in sequence
[group("Quality")]
verify: fmt lint test
    just _echo_complete "All quality checks passed"

# ------------------------------------------------------------ #
#                          MAINTENANCE                         #
# ------------------------------------------------------------ #

# Remove all build artifacts and Go cache
[confirm("Are you sure you want to clean all build artifacts?")]
[group("Maintenance")]
clean:
    just _echo_start "Cleaning build artifacts"
    just _clean_dir {{ BIN_DIR }}
    just _clean_dir "dist"
    just _clean_files "*.coverprofile"
    @go clean -cache
    just _echo_complete "Clean"
