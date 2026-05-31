# PRD: Root Config Discovery

**Status**: draft
**Created**: 2026-05-31
**Triage**: ready-for-agent

## Problem Statement

Lithos root and config discovery currently mixes concerns and includes circular assumptions (config declares vault root while root is required to find config). The current discovery path logic is also hardcoded and not robust across multiple local/global config locations and formats.

## Solution

Implement a dedicated typestate root-config discovery pipeline in `config/discovery/` that resolves context in deterministic phases: explicit overrides, global config candidates, ascending local discovery, root resolution, local config selection, and completion. This pipeline removes `vault_path` from local config payloads and treats root as runtime-discovered context.

## User Stories

1. As a CLI user, I want explicit override behavior to win deterministically, so that command intent is respected.
2. As a config maintainer, I want vault root resolved before config hydration, so that circular configuration is eliminated.
3. As a cross-platform user, I want root discovery to behave consistently across OSes, so that onboarding is predictable.
4. As a maintainer, I want multiple local config location patterns supported, so that migration and compatibility are simpler.
5. As a maintainer, I want multiple structured file formats supported with precedence, so that config selection is deterministic.
6. As a reliability-focused engineer, I want malformed config to fail fast, so that the system never silently degrades.
7. As a platform engineer, I want global trusted-path fallback after local search misses, so that non-local workflows are supported.
8. As a test author, I want discovery phases explicit and typed, so that edge-case behavior is easy to verify.
9. As an operator, I want trace-level discovery logs, so that root/config resolution decisions are observable.
10. As an architecture reviewer, I want root discovery located under config context, so that module ownership stays clear.

## Implementation Decisions

- Create a new `config/discovery/` submodule and deprecate/remove the existing flat discovery file.
- Implement a typestate processor for root-config discovery phases.
- Place `VaultRoot`, `ConfigLocation`, `GlobalConfigLocation`, `LocalConfigLocation`, and discovery result contracts in this root-discovery submodule.
- Remove `vault_path` from local raw config schema and raw config DTOs.
- Implement ascending discovery from working directory with clear termination boundaries.
- Support explicit override, environment override, and standard global config locations.
- Support structured config format selection via precedence-aware candidates.
- Emit tracing logs for each phase transition and major branch decision.

## Testing Decisions

- Good tests assert phase outcomes and selected discovery contracts, not private helper structure.
- Test modules: override precedence, ascending traversal behavior, local/global fallback behavior, malformed/missing config behavior, multi-format candidate selection, trace emission points.
- Prior art: existing config builder/discovery and fs path validation tests.

## Out of Scope

- Full downstream config processing/hash change detection.
- Filesystem discovery processor behavior.
- Context routing or schema/note/template orchestration.
- Context-level event sourcing adoption.

## Further Notes

- This PRD is a prerequisite for centralized filesystem discovery because it provides the root and config lens contract.
