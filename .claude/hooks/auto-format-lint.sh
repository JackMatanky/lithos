#!/usr/bin/env bash
#shellcheck shell=bash

# =============================================================================
# Format and lint a single file based on extension.
# - Web: prettier --write
# - Python (*.py): ruff format + ruff check --fix
# - Go (*.go): golangci-lint fmt + golangci-lint run --fix
# - Rust (*.rs): rustfmt + cargo clippy --fix (if in a Cargo project)
# - Shell (*.sh): shfmt -w + shellcheck (no auto-fix available)
#
# Reads a JSON payload from STDIN with fields:
#   tool_name: string (ignored but parsed for completeness)
#   tool_input.file_path or tool_input.path: string path to the file
#
# Quiet no-op if the file path is missing or tools are not installed.
# Always exits 0.
#
# Dependencies:
#   Required: bash, jq
#   Optional: prettier, ruff, golangci-lint (v2+), rustfmt, cargo, shfmt
# =============================================================================

#######################################
# Validate that a tool exists in PATH.
# Globals:
#   None
# Arguments:
#   cmd: Command name to check.
# Outputs:
#   None
# Returns:
#   0 if command exists, non-zero otherwise.
#######################################
_validate_tool() {
    command -v "$1" >/dev/null 2>&1
}

#######################################
# Run a formatter if available and print a success message.
# Globals:
#   None
# Arguments:
#   cmd: Formatter command.
#   verb: Subcommand/verb for the formatter.
#   path: File path to format.
#   ... : Extra args after verb (optional).
# Outputs:
#   Success message to stderr if formatting succeeds.
# Returns:
#   0 if formatter is missing or succeeds, non-zero if command fails.
#######################################
_fmt() {
    local cmd="$1" verb="$2" path="$3"
    shift 3
    _validate_tool "$cmd" || return 0
    "$cmd" "$verb" "$@" -- "$path" >/dev/null 2>&1 &&
        echo "✅ [FORMAT] ${cmd} ${verb}: ${path}" >&2
}

#######################################
# Run a linter if available and print a success message.
# Globals:
#   None
# Arguments:
#   cmd: Linter command.
#   verb: Subcommand/verb for the linter.
#   path: File or dir path to lint.
#   ... : Extra args after verb (optional).
# Outputs:
#   Success message to stderr if linting succeeds.
# Returns:
#   0 if linter is missing or succeeds, non-zero if command fails.
#######################################
_lint() {
    local cmd="$1" verb="$2" path="$3"
    shift 3
    _validate_tool "$cmd" || return 0
    "$cmd" "$verb" "$@" -- "$path" >/dev/null 2>&1 &&
        echo "🔎 [LINT] ${cmd} ${verb}: ${path}" >&2
}

#######################################
# Find Cargo project root by walking up from a path.
# Globals:
#   None
# Arguments:
#   start_path: File path inside (or equal to) the Cargo project.
# Outputs:
#   Writes the Cargo root directory to stdout on success.
# Returns:
#   0 if a Cargo.toml is found; non-zero otherwise.
#######################################
_find_cargo_root() {
    local start_path="$1" dir
    dir="$(dirname "$start_path")"
    while [[ "$dir" != "/" ]]; do
        if [[ -f "$dir/Cargo.toml" ]]; then
            echo "$dir"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    return 1
}

#######################################
# Run a linter in a specific working directory.
# Intended for project-scoped tools like "cargo clippy".
# Globals:
#   None
# Arguments:
#   workdir: Directory to run the linter in.
#   cmd: Linter command.
#   verb: Subcommand/verb for the linter.
#   display_path: Path to show in messages (informational only).
#   ... : Extra args after verb (optional).
# Outputs:
#   Success message to stderr if linting succeeds.
# Returns:
#   0 if linter is missing or succeeds, non-zero if command fails.
#######################################
_lint_in_dir() {
    local workdir="$1" cmd="$2" verb="$3" display_path="$4"
    shift 4
    _validate_tool "$cmd" || return 0
    (cd "$workdir" && "$cmd" "$verb" "$@" >/dev/null 2>&1) &&
        echo "🔎 [LINT] ${cmd} ${verb}: ${display_path}" >&2
}

#######################################
# Main entry point: parse input and dispatch to the appropriate tools.
# Globals:
#   None
# Arguments:
#   None
# Outputs:
#   Writes formatter/linter success messages to stderr.
# Returns:
#   0 always (no-op on unsupported/missing data).
#######################################
main() {
    local input tool_name file_path ext cargo_root

    # Read the tool input from stdin.
    input="$(cat)"

    # Extract fields (keep tool_name even if unused).
    tool_name="$(echo "$input" |
        jq -r '.tool_name')"
    file_path="$(echo "$input" |
        jq -r '.tool_input.file_path // .tool_input.path // ""')"

    # No file path? Nothing to do.
    [[ -z "$file_path" ]] && return 0

    # Get file extension.
    ext="${file_path##*.}"

    case "$ext" in
    js | jsx | ts | tsx | json | css | scss | html | vue | yaml | yml)
        # Web: Prettier
        _fmt prettier --write "$file_path"
        ;;
    py)
        # Python: format + lint (fix)
        _fmt ruff format "$file_path"
        _lint ruff check "$file_path" --fix
        ;;
    go)
        # Go: format + lint (fix)
        _fmt golangci-lint fmt "$file_path"
        _lint golangci-lint run "$file_path" --fix
        ;;
    rs)
        # Rust: rustfmt (per-file)
        _fmt rustfmt "" "$file_path"
        # Rust lint (fix) via cargo clippy at project root, if available.
        if cargo_root="$(_find_cargo_root "$file_path")"; then
            _lint_in_dir "$cargo_root" cargo clippy "$file_path" \
                --fix --allow-dirty --allow-staged -- -W clippy::all
        fi
        ;;
    sh)
        # Shell: shfmt (per-file) + shellcheck (no auto-fix)
        _fmt shfmt -w "$file_path"
        _lint shellcheck "" "$file_path"
        ;;
    *)
        # Unsupported extension: no-op
        :
        ;;
    esac

    return 0
}

main "$@"
