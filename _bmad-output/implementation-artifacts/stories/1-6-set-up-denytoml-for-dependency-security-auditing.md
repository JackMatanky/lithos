# Story 1.6: set-up-denytoml-for-dependency-security-auditing

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer managing dependencies,
I want automated security vulnerability scanning,
So that insecure dependencies are caught before deployment.

## Acceptance Criteria

**Given** I have researched cargo-deny best practices for dependency security auditing
**When** I review deny.toml configuration
**Then** security vulnerability scanning is configured with:
- `[advisories]` with `db-urls = ["https://github.com/rustsec/advisory-db"]` for RustSec database
- `[yanked]` with `enabled = true` to prevent using yanked crates
- `[bans]` with skip-tree for build dependencies to allow dev tools
- `[licenses]` with allow OSI-approved licenses and deny copyleft (GPL, LGPL)
- `[sources]` with allow-registry for trusted crates.io only

**Given** I have researched license compatibility for Rust projects
**When** I check the deny configuration
**Then** licenses are configured to:
- Allow OSI-approved licenses (MIT, Apache-2.0, BSD variants)
- Deny copyleft licenses (GPL-3.0, LGPL-2.1) unless explicitly approved
- Allow exceptions for build-time dependencies with different license requirements

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

- [ ] Research comprehensive cargo-deny best practices and security auditing standards from enterprise Rust projects
   - [ ] Analyze advisories configuration for RustSec database integration
   - [ ] Review bans and licenses settings for dependency control
   - [ ] Study yanked versions and sources checking
   - [ ] Examine CI/CD integration patterns for cargo deny
- [ ] Create deny.toml with all security auditing best practices
   - [ ] Configure [advisories] section with RustSec database and severity thresholds
   - [ ] Set up [bans] for problematic crates with skip-tree for build dependencies
   - [ ] Configure [licenses] to allow OSI-approved licenses and deny copyleft
   - [ ] Enable [yanked] checking to prevent yanked crate usage
   - [ ] Set up [sources] to allow only trusted registries
- [ ] Test deny.toml configuration against existing dependencies
   - [ ] Run cargo deny check all to verify no violations
   - [ ] Address any license or security issues in current dependencies
   - [ ] Ensure configuration doesn't block legitimate dependencies
- [ ] Integrate cargo deny checks into development workflow
   - [ ] Update mise tasks to include cargo deny checks
   - [ ] Verify pre-commit hooks run deny checks successfully
   - [ ] Test integration with existing quality pipeline
- [ ] Document dependency security standards and commit changes
   - [ ] Update README.md with dependency security policies
   - [ ] Add comments to deny.toml explaining each section
   - [ ] Stage and commit with conventional message: "feat(env): implement cargo deny dependency security auditing"

## Dev Notes

- Relevant architecture patterns and constraints
- Source tree components to touch
- Testing standards summary

### Project Structure Notes

- Alignment with unified project structure (paths, modules, naming)
- Detected conflicts or variances (with rationale)

### References

- Cite all technical details with source paths and sections, e.g. [Source: docs/<file>.md#Section]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
