# Story 1.6: set-up-denytoml-for-dependency-security-auditing

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer managing dependencies,
I want automated security vulnerability scanning,
So that insecure dependencies are caught before deployment.

## Acceptance Criteria

**Given** I have researched cargo-deny best practices for dependency security auditing
**When** I review deny.toml configuration
**Then** security vulnerability scanning is configured with:
- [x] `[advisories]` with `db-urls = ["https://github.com/rustsec/advisory-db"]` for RustSec database
- [x] `yanked` checks enabled (configured via `yanked = "deny"` in 0.19.0) to prevent using yanked crates
- [x] `[bans]` with `skip-tree` available for build dependencies to allow dev tools
- [x] `[licenses]` with allow OSI-approved licenses and deny copyleft (GPL, LGPL)
- [x] `[sources]` with allow-registry for trusted crates.io only

**Given** I have researched license compatibility for Rust projects
**When** I check the deny configuration
**Then** licenses are configured to:
- [x] Allow OSI-approved licenses (MIT, Apache-2.0, BSD variants)
- [x] Deny copyleft licenses (GPL-3.0, LGPL-2.1) unless explicitly approved
- [x] Allow exceptions for build-time dependencies with different license requirements

**Given** deny.toml is configured with security and license checks
**When** I run `cargo deny check`
**Then** all checks pass without security vulnerabilities or license violations

**Given** a dependency has a known security vulnerability
**When** I run cargo deny check advisories
**Then** the command fails with specific vulnerability details, severity levels, and mitigation suggestions

**Given** a yanked crate version is in the dependency tree
**When** I run cargo deny check
**Then** the check fails with yanked crate detection and version recommendations

**Given** I have researched supply chain security best practices for Rust
**When** I check the configuration
**Then** settings align with enterprise Rust projects for comprehensive dependency security auditing

## Tasks / Subtasks

- [x] Research comprehensive cargo-deny best practices and security auditing standards from enterprise Rust projects
   - [x] Analyze advisories configuration for RustSec database integration
   - [x] Review bans and licenses settings for dependency control
   - [x] Study yanked versions and sources checking
   - [x] Examine CI/CD integration patterns for cargo deny
- [x] Create deny.toml with all security auditing best practices
   - [x] Configure [advisories] with db-urls = ["https://github.com/rustsec/advisory-db"]
   - [x] Set up [bans] with skip-tree for build/dev dependencies
   - [x] Configure [licenses] with allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC"]
   - [x] Enable yanked checks (set to "deny")
   - [x] Set up [sources] with unknown-registry = "deny" and allow-registry = ["https://github.com/rust-lang/crates.io-index"]
- [x] Test deny.toml configuration against existing dependencies
   - [x] Run cargo deny check advisories to verify no security vulnerabilities
   - [x] Run cargo deny check licenses to verify license compliance
   - [x] Run cargo deny check bans to verify no banned crates
   - [x] Run cargo deny check sources to verify registry compliance
   - [x] Address any violations in current dependencies or adjust configuration
   - [x] Ensure configuration doesn't block legitimate project dependencies
- [x] Integrate cargo deny checks into development workflow
   - [x] Update .mise.toml tasks to include cargo deny check all
   - [x] Update .pre-commit-config.yaml to run cargo deny check
   - [x] Verify pre-commit hooks run deny checks successfully
   - [x] Test full quality pipeline integration (fmt + lint + test + deny)
- [x] Document dependency security standards and commit changes
   - [x] Update README.md with dependency security policies
   - [x] Add comments to deny.toml explaining each section
   - [x] Stage and commit with conventional message: "feat(env): implement cargo deny dependency security auditing"

## Dev Notes

- Relevant architecture patterns and constraints: Integration with `mise` and `pre-commit`.
- Source tree components to touch: `deny.toml`, `mise.toml`, `README.md`, `.pre-commit-config.yaml`.
- Testing standards summary: Cargo deny checks run in pre-commit and CI, failing builds with any security vulnerabilities, license violations, or banned dependencies to maintain supply chain integrity. Keep RustSec advisory database updated regularly using `cargo deny fetch` to ensure latest vulnerability information.

### Project Structure Notes

- Alignment with unified project structure (paths, modules, naming): Standard `deny.toml` at repo root.
- Detected conflicts or variances (with rationale): `cargo-deny` 0.19.0 introduced breaking changes in configuration keys. Configuration was adjusted to be 0.19.0 compatible while maintaining security requirements.

### References

- [Source: deny.toml]
- [Source: mise.toml]
- [Source: .pre-commit-config.yaml]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}} (Post-Review Fixes applied)

### Debug Log References

- Configured `deny.toml` for `cargo-deny` 0.19.0 with explicit `yanked = "deny"`.
- Resolved duplicate dependency warnings by auditing and adding explicit skips for `unicode-width` and `windows-sys` in `deny.toml`.
- Cleaned up unused license allowances in `deny.toml`.
- Verified integration with `mise run verify` and confirmed `pre-commit` hook is active and passing.
- Corrected documentation regarding `.pre-commit-config.yaml` verification.

### Completion Notes List

- Implemented comprehensive `deny.toml` with advisories (including yanked), licenses, bans (multiple-versions denied), and sources checks.
- Unified quality gates by ensuring `cargo-deny` runs in both `mise` and `pre-commit`.
- Updated `README.md` to reflect new security standards.

### File List

- `deny.toml`
- `mise.toml`
- `.pre-commit-config.yaml`
- `README.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/stories/1-6-set-up-denytoml-for-dependency-security-auditing.md`

### Change Log

- 2026-01-11: Initial implementation of cargo-deny security auditing.
- 2026-01-11: Post-review fixes: explicit yanked configuration, dependency unification/auditing, and license cleanup.
