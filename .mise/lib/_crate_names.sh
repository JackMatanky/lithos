#!/usr/bin/env bash
#
# _crate_names.sh — Shared crate discovery and mapping utilities.
#
# Provides functions for dynamically resolving workspace crate names
# from the filesystem, eliminating hardcoded package name mappings.
#
# Usage: source "$(dirname "$0")/_crate_names.sh"
#
# Functions:
#   discover_crate_names    — Lists all workspace crate package names
#   resolve_crate_dir       — Maps a shorthand to a crate directory
#   CrateNameMapping        — A global associative array for lookups

#######################################
# Discover all workspace crate package names from the filesystem.
# Reads crates/*/Cargo.toml and extracts the `name` field.
# Globals:
#   None
# Arguments:
#   None
# Outputs:
#   Writes one package name per line to stdout
# Returns:
#   0 if at least one crate is found, 1 otherwise
#######################################
discover_crate_names() {
    local project_root
    project_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "${MISE_PROJECT_ROOT:-.}")"

    local crates_dir="${project_root}/crates"
    if [[ ! -d "${crates_dir}" ]]; then
        return 1
    fi

    local found=0
    local cargo_file
    for cargo_file in "${crates_dir}"/*/Cargo.toml; do
        if [[ -f "${cargo_file}" ]]; then
            local pkg_name
            pkg_name="$(awk '/^name = / { gsub(/.*name = "/, ""); gsub(/".*/, ""); print; exit }' "${cargo_file}")"
            if [[ -n "${pkg_name}" ]]; then
                echo "${pkg_name}"
                found=1
            fi
        fi
    done

    if [[ "${found}" -eq 0 ]]; then
        return 1
    fi
}

#######################################
# Resolve a shorthand or directory name to a cargo package name.
# Tries: exact match, crates/<name>/ match, prefix match.
# Globals:
#   None
# Arguments:
#   $1 - Crate shorthand (e.g., "cli", "settings", "trace-cli")
# Outputs:
#   Writes the resolved package name to stdout, or empty string
#######################################
resolve_crate_name() {
    local shorthand="$1"
    [[ -z "${shorthand}" ]] && return 1

    # Direct match against known crate dirs
    local project_root
    project_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "${MISE_PROJECT_ROOT:-.}")"

    local cargo_file="${project_root}/crates/${shorthand}/Cargo.toml"
    if [[ -f "${cargo_file}" ]]; then
        awk '/^name = / { gsub(/.*name = "/, ""); gsub(/".*/, ""); print; exit }' "${cargo_file}"
        return 0
    fi

    # It might already be a full package name — verify it exists
    if discover_crate_names | grep -qxF "${shorthand}"; then
        echo "${shorthand}"
        return 0
    fi

    return 1
}

#######################################
# Build cargo --package arguments for one or more crates.
# If no shorthand is given, output --workspace.
# Globals:
#   None
# Arguments:
#   $1 - Optional crate shorthand (empty = workspace)
# Outputs:
#   Writes "--package <name>" or "--workspace" to stdout
#######################################
build_package_arg() {
    local shorthand="$1"
    if [[ -z "${shorthand}" ]]; then
        echo "--workspace"
        return 0
    fi

    local resolved
    resolved="$(resolve_crate_name "${shorthand}")"
    if [[ -n "${resolved}" ]]; then
        echo "--package"
        echo "${resolved}"
    else
        echo "Error: Unknown crate '${shorthand}'" >&2
        return 1
    fi
}
