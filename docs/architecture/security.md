# Security

## Input Validation

- SchemaValidatorService enforces frontmatter schemas (required fields, enums, patterns, file references).
- CLI flags validated via Cobra binding with explicit range/default checks.
- Interactive prompts (promptui/fuzzy finder) sanitize input before invoking template functions.

## Authentication & Authorization

- MVP runs entirely on local vaults—no network services, thus no authentication.
- If remote services are introduced (template registry, schema hub), they **MUST** use token-based authentication with least-privilege scopes and TLS 1.2+.

## Secrets Management

- Current workflow stores no secrets; configuration happens via local files/env vars.
- Future credentials (if any) **MUST** be injected through environment variables or OS keychains—never committed or logged.

## API Security

- No HTTP API in MVP.
- Future REST/TUI adapters **MUST** enforce HTTPS, rate limiting, and signed payload verification when calling external services.

## Data Protection

- Encourage users to keep vaults on encrypted storage; `.lithos/` cache inherits vault permissions.
- Logs **MUST NOT** include prompt responses or raw frontmatter values—only field names and error context.

## Dependency Security

- `golangci-lint` + `gitleaks` run on every PR; optional `renovate` (or quarterly review) monitors dependencies.
- Third-party libs (e.g., `promptui`) tracked for maintenance status; migration plans documented if security risks emerge.

## Security Testing

- `gosec ./...` integrates into CI (non-blocking but triaged).
- Manual review of template execution functions ensures no untrusted command execution paths.
- Template packs distributed with Lithos must pass static linting to detect dangerous constructs.

---
