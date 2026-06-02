---
labels: ["ready-for-agent"]
---

## Problem Statement

Lithos parsing responsibilities are split across seams that currently blur context ownership: Vault Root file access, structured decoding, and markdown ingestion primitives are coupled in ways that reduce locality and make change riskier than necessary. In practice, parsing behavior is spread between FS and context-specific modules, with structured parsing entry points tied to `FileReader`, and markdown ingestion behavior embedded in Note parser internals. This makes it harder to evolve parsing policy, improve diagnostics, and reuse frontmatter/offset logic for Template Asset ingestion without duplicating logic.

From the user perspective, this increases implementation friction and confidence cost when adding parsing features, evolving error messages, or introducing shared markdown behavior for Note and Template contexts.

## Solution

Introduce a parser-owned architecture that deepens parsing modules while preserving context boundaries:

- Keep FS focused on Vault Root-safe file operations (Path Validation + reads).
- Move parse policy into a dedicated parser module with clear submodules for structured and markdown parsing.
- Define parser-owned parseable format contracts and parser-owned errors with precise suberrors.
- Share only markdown ingestion primitives (frontmatter/body split, offset-aware event adaptation) between contexts.
- Keep Note and Template semantics context-local (no semantic unification).

This yields a clean `File Source -> Parser -> Domain Construction -> Projection` flow aligned with current type-driven design goals ("parse, don't validate") and with pulldown-cmark event-stream design in ADR 008.

## User Stories

1. As a developer, I want File Source reads to remain isolated from parse policy, so that filesystem safety and parsing evolution can change independently.
2. As a developer, I want structured parsing to have a parser-owned interface, so that Schema and Config flows do not depend on `FileReader` parsing helpers.
3. As a developer, I want parseable format classification to be explicit, so that unsupported-format behavior is deterministic and easy to test.
4. As a developer, I want Markdown included as a parseable format family, so that markdown ingestion can be represented consistently alongside JSON/TOML/YAML.
5. As a developer, I want parser errors split into structured and markdown suberrors, so that diagnostics are precise and operationally actionable.
6. As a developer, I want crate-native parser errors preserved as sources, so that upstream parse details are available without duplicating parser internals.
7. As a developer, I want parser error variants to carry path/format/range context, so that troubleshooting does not require reproducing failures with instrumentation.
8. As a Note maintainer, I want Note semantics (tags/tasks/link extraction) to remain in Note context, so that shared parser modules do not leak domain behavior.
9. As a Template maintainer, I want to reuse markdown frontmatter/body extraction, so that Template Asset ingestion does not reimplement pulldown behavior.
10. As a performance-conscious developer, I want pulldown-based markdown ingestion to remain offset-aware and linear, so that large vault ingestion stays efficient.
11. As a developer, I want byte offset mapping behavior to remain stable through refactor, so that position-derived features and diagnostics remain correct.
12. As an architect, I want `StructuredFileFormat` ownership to align with parse policy rather than file IO, so that discovery precedence logic lives at the right seam.
13. As a developer, I want migration to be staged with compatibility delegates, so that high-blast-radius schema flows are not broken in one cutover.
14. As a developer, I want old parsing entry points deprecated only after caller migration, so that refactor risk remains controlled.
15. As a reviewer, I want seam-level tests to pin behavior before and after migration, so that architectural change is measurable and safe.
16. As a user of Lithos, I want parsing failures to explain what failed and where, so that invalid files can be corrected quickly.
17. As a future contributor, I want parser modules to be deep and discoverable, so that implementation and AI navigation are easier.
18. As a developer, I want context boundaries preserved while sharing primitives, so that code reuse does not collapse domain separation.
19. As a maintainer, I want minimal duplicate frontmatter logic across Note and Template, so that fixes land once.
20. As a maintainer, I want migration docs and deprecation path clarity, so that follow-up work does not stall on architectural ambiguity.

## Implementation Decisions

- **Parser module introduction**: Introduce a parser-owned module with separate submodules for format contracts, structured parsing, markdown parsing, and parser errors.
- **FS context narrowing**: FS remains owner of Vault Root-safe reads and path safety. FS no longer owns parse policy after migration.
- **ParseableFormat contract**: Add `ParseableFormat` with `Json`, `Toml`, `Yaml`, and `Markdown` to represent parser-level format families explicitly.
- **Structured parsing seam**: Structured parsing APIs handle only structured decode behavior (JSON/TOML/YAML), and reject non-structured parse attempts through precise parser errors.
- **Markdown parsing seam**: Shared markdown module exposes frontmatter/body extraction and offset-aware event adaptation primitives only; no Note/Template semantics included.
- **Error hierarchy**: Parser errors are split into suberrors:
  - `StructuredParserError` for structured decode and format-policy failures.
  - `MarkdownParserError` for frontmatter/offset/event adaptation failures.
  - Optional small wrapper (`ParserError`) to normalize top-level propagation.
- **Crate-native error source preservation**: JSON/TOML/YAML parse errors are preserved as sources and wrapped with Lithos context (path/format/range) when needed.
- **Structured format ownership migration**: `StructuredFileFormat` migrates toward parser ownership with compatibility re-exports during transition to avoid wide breakage in discovery flows.
- **No speculative trait now**: Do not add a `StructuredDecoder` trait yet. Use concrete APIs first; introduce a trait only if a second real adapter emerges.
- **Position ownership decision**: Keep Note position types in Note context for now because they carry Note-domain invariants and broad downstream usage.
- **Offset seam strategy**: Shared markdown module may use parser-neutral ranges internally and convert to Note position types at context seam boundaries.
- **Compatibility stage**: Existing `FileReader` structured parsing helpers become transitional delegates to parser module before deprecation/removal.
- **Blast-radius-aware rollout**: Migrate high-dependency structured parsing callers incrementally (especially schema/property bank flows), then retire legacy entry points.
- **Architectural alignment**:
  - Align with current type-driven design goals (`File Source -> Parse -> Construct Domain -> Project`) without relying on ADR 010 as authoritative.
  - Align with ADR 008 (pulldown event-stream + offset-based processing).
- **Rust best-practices constraints**:
  - Prefer concrete static dispatch initially.
  - Use typed errors (`thiserror`) and avoid panic-driven control flow.
  - Keep borrowing/ownership efficient in markdown event paths and avoid unnecessary cloning in hot parsing loops.

## Proposed Interfaces

The following interfaces define seam-level contracts for parser-owned structured and markdown primitives. They are intentionally concrete-first and preserve current behavior while decoupling parse policy from File Source ownership.

```rust
use std::path::Path;

use crate::fs::format::StructuredFileFormat;
use crate::utils::position::{ByteRange, PositionError};

/// Structured parse policy (attribute carried by parser instance).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuredParsePolicy {
    /// Prefer extension over content sniffing when both are present.
    pub prefer_extension: bool,
}

impl Default for StructuredParsePolicy {
    fn default() -> Self {
        Self {
            prefer_extension: true,
        }
    }
}

/// Parser-level format family classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseableFormat {
    Json,
    Toml,
    Yaml,
    Markdown,
}

/// Parser-owned structured parsing seam.
pub struct StructuredParser {
    policy: StructuredParsePolicy,
}

impl StructuredParser {
    #[must_use]
    pub fn new(policy: StructuredParsePolicy) -> Self;

    /// Parse structured content using deterministic format policy.
    pub fn parse_from_str<T>(
        path: &Path,
        content: &str,
    ) -> Result<T, StructuredParserError>
    where
        T: serde::de::DeserializeOwned;

    /// Parse with explicit structured selector (discovery-controlled path).
    pub fn parse_with_format<T>(
        path: &Path,
        content: &str,
        format: StructuredFileFormat,
    ) -> Result<T, StructuredParserError>
    where
        T: serde::de::DeserializeOwned;

    /// Classify parseable family from extension and optional content hint.
    pub fn classify_parseable(
        path: &Path,
        content: Option<&str>,
    ) -> ParseableFormat;
}

/// Parser-owned markdown seam shared by contexts.
///
/// Uses pulldown-cmark directly as the event source and frontmatter detector.
pub struct MarkdownParser;

impl MarkdownParser {
    /// Split leading frontmatter and markdown body without note/template semantics.
    ///
    /// Implementation uses pulldown-cmark metadata block events
    /// (`Tag::MetadataBlock`) instead of ad-hoc delimiter scanning.
    pub fn split_frontmatter<'a>(
        source: &'a str,
    ) -> Result<FrontmatterSplit<'a>, MarkdownParserError>;

    /// Adapt pulldown-cmark events to offset-aware neutral events.
    pub fn adapt_events<'a>(
        source: &'a str,
        options: pulldown_cmark::Options,
    ) -> Result<OffsetEventStream<'a>, MarkdownParserError>;
}

/// Frontmatter/body extraction result with exact source slices.
pub struct FrontmatterSplit<'a> {
    pub frontmatter: Option<FrontmatterSlice<'a>>,
    pub body: &'a str,
    pub body_range: ByteRange,
}

/// Borrowed frontmatter view with exact byte span.
pub struct FrontmatterSlice<'a> {
    pub raw: &'a str,
    pub range: ByteRange,
}

/// Parser-owned markdown event projected with byte offsets.
pub struct OffsetEvent<'a> {
    pub range: ByteRange,
    pub event: pulldown_cmark::Event<'a>,
}

pub type OffsetEventStream<'a> = Vec<OffsetEvent<'a>>;
```

```rust
/// Structured parser failures with source-chain preservation.
#[derive(Debug, thiserror::Error)]
pub enum StructuredParserError {
    #[error("unsupported structured format")]
    UnsupportedFormat(#[from] crate::fs::error::ParseError),

    #[error("json decode failed")]
    Json(#[from] serde_json::Error),

    #[error("toml decode failed")]
    Toml(#[from] toml::de::Error),

    #[error("yaml decode failed")]
    Yaml(#[from] serde_yaml::Error),

    #[error("structured parse context: {path}")]
    Context {
        path: std::path::PathBuf,
        format: StructuredFileFormat,
        #[source]
        source: Box<StructuredParserError>,
    },
}

/// Markdown primitive failures with positional context.
#[derive(Debug, thiserror::Error)]
pub enum MarkdownParserError {
    #[error("markdown parse failed")]
    Pulldown,

    #[error("yaml frontmatter decode failed")]
    FrontmatterYaml(#[from] serde_yaml::Error),

    #[error("offset mapping failed")]
    Offset(#[from] PositionError),

    #[error("markdown parse context: {path}")]
    Context {
        path: std::path::PathBuf,
        #[source]
        source: Box<MarkdownParserError>,
    },
}

/// Optional top-level wrapper for call sites that need unified propagation.
#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error(transparent)]
    Structured(#[from] StructuredParserError),
    #[error(transparent)]
    Markdown(#[from] MarkdownParserError),
}
```

Notes:
- Keep `FileReader` as File Source owner (`read_to_string`, path safety); do not move Vault Root enforcement into parser modules.
- Keep Note/Template semantics out of `MarkdownParser`; those contexts consume neutral primitives and perform context-local construction.
- Prefer borrowed outputs (`&str` slices, neutral ranges) for hot paths; avoid unnecessary allocation/cloning in event adaptation.
- Prefer `FrontmatterSlice` for parser seam exchange; decode into typed frontmatter happens in consuming context.
- Move source-position primitives out of Note into `utils::position`; parser, Note, Template, and diagnostics consume the same deep byte/position module.

### Position/Byte Ownership Plan

- Introduce shared source-position primitives in `utils::position`:
  - `ByteOffset(u32)`
  - `ByteRange { start: ByteOffset, end: ByteOffset }`
  - `Location { offset: ByteOffset, line: Line, column: Column }`
  - `LocationRange { start: Location, end: Location }`
  - `LineIndex`
  - `Line`
  - `Column`
- Use shorter names because the module path carries meaning (`utils::position::ByteOffset`, not `SourceByteOffset`).
- Keep fields private and validate through constructors/methods rather than public struct fields.
- Split errors by depth:
  - `ByteOffsetError` owns byte-value failures: overflow, out-of-bounds, UTF-8 boundary violations.
  - `PositionError` composes `ByteOffsetError` and owns range/location failures: invalid range, invalid line, invalid column.
- Let contexts use the most specific error they need:
  - offset-only operations return `ByteOffsetError`.
  - range/location/index operations return `PositionError`.
  - Note/Template/Parser errors wrap these transparently where they add context.
- Tighten byte handling during extraction:
  - all offset construction from `usize` checks `u32` overflow
  - all string-facing offset validation checks bounds and UTF-8 boundary
  - all range constructors enforce half-open invariant (`start <= end`)
  - all offset arithmetic uses checked add and returns `ByteOffsetError::Overflow`
  - convert to `usize` only at IO/slice boundary

```rust
pub mod utils::position {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ByteOffset(u32);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ByteRange {
        start: ByteOffset,
        end: ByteOffset,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Location {
        offset: ByteOffset,
        line: Line,
        column: Column,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum ByteOffsetError {
        #[error("byte offset overflow")]
        Overflow { offset: ByteOffset, delta: usize },

        #[error("byte offset out of bounds")]
        OutOfBounds {
            offset: ByteOffset,
            source_len: ByteOffset,
        },

        #[error("byte offset is not a UTF-8 boundary")]
        Utf8Boundary { offset: ByteOffset },
    }

    #[derive(Debug, thiserror::Error)]
    pub enum PositionError {
        #[error(transparent)]
        ByteOffset(#[from] ByteOffsetError),

        #[error("invalid byte range")]
        InvalidRange { start: ByteOffset, end: ByteOffset },

        #[error("invalid line")]
        InvalidLine { line: u32 },

        #[error("invalid column")]
        InvalidColumn { column: u32 },
    }
}
```

## Testing Decisions

- **Test quality definition**: Tests validate externally observable parsing behavior and diagnostics contracts, not internal module topology.
- **Behavior pinning before migration**:
  - Pin structured parse outcomes and error reporting for JSON/TOML/YAML malformed inputs.
  - Pin extension/content classification precedence behavior.
  - Pin markdown frontmatter extraction behavior and exact body preservation.
  - Pin offset-to-range validity expectations for event adaptation.
- **Seam test suites**:
  - FS seam tests verify Vault Root and read behavior remain unchanged.
  - Structured parser seam tests verify format handling and precise suberror behavior.
  - Markdown parser seam tests verify frontmatter and offset adaptation behavior.
  - Note integration tests verify no semantic drift in extracted note artifacts.
  - Template integration tests (new parser use) verify reusable frontmatter/body behavior without Note semantic leakage.
- **Error tests**:
  - Verify source-chain preservation for crate-native errors.
  - Verify added context fields (path/format/range) are present and accurate.
- **Performance safety checks**:
  - Re-run existing parse benchmarks and ensure no regression in hot parsing paths.
- **Prior art in codebase**:
  - Existing FS format and reader tests for structured parsing behavior.
  - Existing Note parser tests around frontmatter and reference handling.
  - Existing note parsing benchmark suites for ingest/parse performance baselines.

## Out of Scope

- Full semantic unification of Note and Template parsing logic.
- Moving all Note position primitives into a shared utility context immediately.
- Redesigning downstream domain validation semantics for Note/Schema/Template.
- Reworking repository traits, persistence model, or database projection contracts.
- Implementing unrelated architectural candidates previously marked irrelevant.

## Further Notes

- Migration should proceed in narrow, reviewable slices to reduce risk on schema/property-bank ingestion flows.
- Deprecation should be explicit and time-boxed after caller migration is complete.
- If future adapters create real seam multiplicity, revisit trait extraction with evidence.
- Shared byte/position primitives belong in `utils::position`, not Note or Parser, because they model source text rather than context semantics.
- ADR 010 is treated as historical context only and is not normative for this effort.

## Migration Safety Contract

| Phase | Scope | Required tests/benchmarks to pass | Rollback trigger |
| --- | --- | --- | --- |
| Phase 0: Behavior pinning baseline | Add seam-level behavior tests before production refactor | Existing FS reader structured tests; classification precedence tests; existing note parser frontmatter/range tests; `lithos-core/benches/note_parsing.rs` baseline capture | Any mismatch between pinned assertions and current main behavior; baseline benchmark capture failure |
| Phase 1: Structured parser seam introduction | Introduce `StructuredParser` and delegate existing `FileReader::parse_structured_from_str` to parser seam with identical behavior | `fs::reader` structured parse tests; schema property bank parsing tests (including malformed input paths); source-chain error assertions; no new clippy warnings | Any changed parse outcome/error contract for existing structured callers; schema/property-bank flow failure |
| Phase 2: Structured caller migration | Move high-blast-radius schema/property-bank callers to parser-owned API directly | Full schema builder/property-bank suite; integration tests for discovery precedence; regression tests for stale/fresh/content-mismatch processor branches; compare with Phase 1 snapshots | Any break in property-bank load/process flows; divergence in precedence or error context fields |
| Phase 2.5: Position primitive extraction | Extract source byte/range/location primitives from `note/position.rs` into `utils::position`; keep Note/Template/Parser semantics local | Existing `note::position` test suite ported/pinned; UTF-8 boundary regression tests; half-open range invariant tests; overflow tests; no semantic API break in Note call sites | Any regression in offset-to-line/column behavior; any context semantic type leak into `utils::position`; adapter/conversion mismatch |
| Phase 3: Markdown seam extraction | Introduce parser-owned `MarkdownParser` for frontmatter split + offset event adaptation; use pulldown metadata events; keep Note semantics unchanged | Note parser integration tests (tags/tasks/links/frontmatter); offset-to-range validity tests; reference extraction tests; benchmark comparison versus Phase 0 note parsing baseline | Semantic drift in Note extraction outputs; offset/range regression; measurable hot-path performance regression |
| Phase 4: Template adoption of shared markdown primitives | Consume shared frontmatter/body primitives in Template Asset ingestion path | Template ingestion tests (new + existing); no Note semantic leakage assertions; frontmatter/body parity tests with note corpus fixtures where applicable | Template behavior regressions; cross-context semantic coupling introduced; incompatible frontmatter behavior |
| Phase 5: Compatibility retirement | Deprecate then remove legacy `FileReader` parsing helpers after all callers migrate | Entire workspace test suite; migration-doc checklist complete; deprecation warnings resolved; lint + fmt + test tasks green | Any remaining production caller on deprecated entry points; unresolved migration docs; post-removal regressions |

Phase gates:
- No phase advances without green required suite + benchmark check where listed.
- Rollback means reverting the current phase patchset and reopening with narrower slice + added failing regression test.
- High-blast-radius structured parsing paths (schema/property-bank) remain mandatory early verification targets on every phase after Phase 1.
