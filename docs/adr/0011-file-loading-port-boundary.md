---
name: file-loading-port-boundary-and-text-only-domain-contract
status: accepted
stakeholders: [Jack (Developer), Architecture]
date_proposed: 2026-01-21
date_decided: 2026-01-21
date_implemented: 2026-01-21
---

# ADR 0015: File Loading Port Boundary and Text-Only Domain Contract

## Context

Lithos needs a unified interface for loading TOML/JSON/YAML configuration files with format detection, security validation, and async I/O. The project follows hexagonal architecture with a strict rule that the domain layer remains pure and free of infrastructure concerns. This creates a boundary question: should the domain layer expose file format concepts, or should format detection and parsing live entirely in the adapter layer?

Evaluation criteria:

- Domain must stay IO-free and storage-agnostic.
- Adapters are responsible for filesystem access and parsing.
- Security checks (path traversal, binary rejection, size limits) are mandatory.
- Performance targets: format detection + parsing under 100 microseconds for typical config files.
- Error messages should include path and format context where relevant.
- The port should use the smallest idiomatic Rust surface that preserves these constraints.

## Decision

Adopt a text-only domain contract using `String` and keep all format detection, parsing, and security validation in adapters.

### Domain Port Contract

- `FileReader` returns UTF-8 validated text (`String`) and is exported as `FileReaderPort`.
- The domain does not expose `FileFormat` or detection logic.
- Errors describe path/context but do not implement parsing.

### Adapter Responsibilities

- Detect format by extension and content (TOML/JSON/YAML).
- Validate security constraints (path traversal, binary content, size limits).
- Perform async I/O via `tokio::fs` in `spawn_blocking`.
- Parse and map errors into domain error types.

## Alternatives Considered

### Alternative 1: Domain owns file format detection

- **Pros**: Shared detection logic; explicit format contract in core.
- **Cons**: Moves format semantics into domain; risks IO-driven policy in core; adds domain-facing concept churn if formats change.

### Alternative 2: Domain returns raw bytes (`Vec<u8>`)

- **Pros**: Strictly format-agnostic; preserves binary payloads; minimal domain coupling.
- **Cons**: Conflicts with binary-rejection requirement; forces UTF-8 validation downstream; encourages duplicated parsing guards.

### Alternative 3: Domain returns UTF-8 text (`String`)

- **Pros**: Aligns with config file expectations; validates binary rejection at the boundary; smallest ergonomic surface for callers.
- **Cons**: Loses access to non-UTF-8 payloads (by design); forces adapters to normalize newlines/encoding.

### Alternative 4: Domain returns `serde_json::Value`

- **Pros**: Normalized document shape; convenient for callers that want structured data.
- **Cons**: Couples domain to serde_json and JSON semantics; drops original format fidelity; increases dependency surface in core.

## Technical Validation

### Research Findings

- Hexagonal architecture emphasizes keeping transport/format concerns out of the domain.
- Config file usage patterns are text-first (TOML/JSON/YAML), and binary input is explicitly rejected for security reasons.
- Idiomatic Rust ports prefer minimal, explicit contracts that avoid leaking external serialization details.

### Compatibility and Performance

- **Hexagonal Alignment**: Adapter-only detection/parsing keeps domain pure while allowing adapters to evolve formats.
- **Performance Impact**: Returning `String` avoids normalization to JSON and reduces allocations compared to full AST conversion; returning `Vec<u8>` avoids UTF-8 validation but conflicts with binary rejection.
- **Error Context**: Adapter can include format + path context in domain errors without encoding format logic in domain.

## Consequences

- **Positive**: Domain remains format-agnostic and IO-free; adapters fully own security and parsing policy; minimal dependencies in core.
- **Negative**: Format detection logic is not shareable by domain tests; callers rely on adapter correctness for detection; port choice may require revisiting if non-text formats are introduced.

### Symlink Handling Strategy

The implementation intentionally **allows symlinks** for configuration files, as they are a legitimate and common pattern for:

- Dotfiles management (e.g., `~/.config/lithos/config.toml` → `~/.dotfiles/lithos/config.toml`)
- Shared configuration across environments
- Version-controlled configuration repositories

**Security Approach**:

- Symlinks are followed transparently by `std::fs::read()` within `spawn_blocking`
- Path traversal protection via `..` component detection prevents escape attacks
- Absolute path rejection ensures relative path usage within expected directories
- Binary content rejection prevents reading non-text files regardless of symlink target

This design prioritizes developer flexibility (supporting common dotfile patterns) while maintaining security through content validation rather than path restrictions.
