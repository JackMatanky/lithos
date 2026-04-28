//! View types for schema staleness detection and version tracking.
//!
//! ## Purpose
//!
//! This module provides the metadata layer that enables **incremental schema
//! updates** in Lithos's "files-as-source-of-truth" architecture. Views track
//! content hashes, file timestamps, and version history to answer "Has this
//! file changed?" **without re-parsing**, delivering massive performance wins
//! for large vaults (1000+ schemas).
//!
//! Views are **not optional** in the schema context—they solve critical
//! architectural problems that cannot be deferred:
//!
//! 1. **Staleness Detection**: O(1) hash comparison instead of O(n) file
//!    parsing
//! 2. **Incremental Resolution**: Track which properties changed to avoid
//!    re-expanding all schemas when property bank updates
//! 3. **Inheritance Metadata**: Extract `extends`/`excludes` for fast zero-copy
//!    queries without deserializing full aggregates
//! 4. **Version History**: Maintain ring buffer (5 versions) for debugging and
//!    rollback
//!
//! ## Architectural Context
//!
//! Views sit at the **persistence boundary** between raw file data and the
//! database, implementing the "cache metadata alongside cached data" principle:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  FILE SYSTEM (Source of Truth)                              │
//! │  schemas/note.yaml, property_bank.yaml                      │
//! └───────────────────────┬─────────────────────────────────────┘
//!                         │ FsReader (security-validated access)
//!                         ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │  PARSE: File → Raw* (syntax validation only)                │
//! │  serde::from_str() → RawSchema, RawPropertyBank             │
//! └───────────────────────┬─────────────────────────────────────┘
//!                         │
//!                         ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │  VIEWS: Staleness Detection Layer (THIS MODULE)             │
//! │                                                              │
//! │  • Compute Blake3 hashes (content + per-property)           │
//! │  • Compare against stored hashes                            │
//! │  • Decision: Fresh → use cache | Stale → re-parse           │
//! │                                                              │
//! │  Types: RawSchemaView, RawPropertyBankView                  │
//! │         SchemaVersion, PropertyBankVersion, HashRecord      │
//! └───────────────────────┬─────────────────────────────────────┘
//!                         │ (if stale/new)
//!                         ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │  VALIDATE: Raw* → Domain (semantic validation)              │
//! │  Schema::try_from(RawSchema) - refs exist, no cycles        │
//! └───────────────────────┬─────────────────────────────────────┘
//!                         │
//!                         ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │  DATABASE: Persist domain aggregates + views                │
//! │  Repository::save(schema) + save_raw_schema_view(view)      │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Core Concepts
//!
//! ### Staleness Detection
//!
//! Views enable a three-tier staleness detection strategy:
//!
//! - **Fast Path**: Timestamp comparison via [`FileStats::is_timestamp_match`]
//!   (filesystem stat only, no I/O)
//! - **Medium Path**: Content hash comparison via
//!   [`HashRecord::is_content_match`] (Blake3 hash of file content)
//! - **Slow Path**: Hash mismatch triggers full re-parse and re-validation
//!
//! ### Version History
//!
//! Each view maintains a **ring buffer** of up to 5 historical versions
//! ([`RawView::MAX_VERSIONS`]). When a new version is added
//! ([`RawView::add_version`]), the oldest is evicted. This enables:
//!
//! - **Debugging**: Compare hash history to diagnose staleness issues
//! - **Rollback**: Restore previous version if user breaks schema
//! - **Audit Trail**: Track when schemas changed over time
//!
//! ### Incremental Resolution
//!
//! Property bank changes are rare but expensive (requires re-expanding all
//! schemas using `$ref`). Views enable **targeted re-expansion** via
//! per-property hashing:
//!
//! ```text
//! 1. Property bank changes → compute new HashRecord
//! 2. Compare property_hashes map: which properties changed?
//! 3. Query schemas: which use changed properties? (via bank_references)
//! 4. Re-expand ONLY affected schemas (not all)
//! ```
//!
//! ### Inheritance Metadata Extraction
//!
//! Schemas have inheritance relationships (`extends`, `excludes`) that must be
//! queryable **before** loading full domain aggregates. Views extract this
//! metadata during ingestion and store it for zero-copy access via
//! `ArchivedRawSchemaView`.
//!
//! ## Module Structure
//!
//! - [`contracts`] - Shared trait boundaries ([`RawView`], [`RawViewRead`],
//!   [`Version`], [`VersionRead`]) implemented for both owned and archived
//!   (`rkyv`) types to ensure consistent behavior across runtime and storage.
//!
//! - [`raw`] - Versioned metadata containers ([`RawSchemaView`],
//!   [`RawPropertyBankView`]) that track file identity, version history (ring
//!   buffer), and inheritance metadata. These are the primary view types
//!   persisted alongside domain aggregates.
//!
//! - [`hashes`] - Content integrity tracking ([`HashRecord`]) combining
//!   file-level content hash (Blake3 of entire file) with per-property hashes
//!   (Blake3 per property definition) to enable incremental resolution.
//!
//! - [`snapshots`] - Version payload types ([`SchemaVersion`],
//!   [`PropertyBankVersion`]) representing individual historical snapshots.
//!   Each version captures file stats, hashes, and extracted metadata at a
//!   point in time.
//!
//! ## Hybrid Serialization Strategy
//!
//! Views follow Lithos's three-layer serialization model:
//!
//! 1. **Raw Layer** (serde only): [`RawSchema`], [`RawPropertyBank`] — Tolerant
//!    parsing with `Option<T>` fields for better error messages.
//!
//! 2. **Domain Layer** (rkyv + optional serde): [`Schema`], [`PropertyBank`] —
//!    Validated aggregates with invariant-preserving types, zero-copy storage.
//!
//! 3. **View Layer** (rkyv only): [`RawSchemaView`], [`RawPropertyBankView`] —
//!    Metadata-only representations optimized for staleness detection and
//!    version tracking.
//!
//! Views use **rkyv exclusively** because:
//! - No human editing required (internal storage format)
//! - Zero-copy deserialization critical for performance (hot path queries)
//! - Stability: View schema changes less frequently than domain aggregates
//!
//! ## View Pattern Guidance
//!
//! **Why schema views are special**: While the general architectural guidance
//! states "views are optional optimizations," schema views are an exception.
//! They solve mandatory architectural requirements:
//!
//! - **Staleness detection** is not optional—it's required for usable
//!   performance at scale
//! - **Inheritance metadata queries** cannot wait for full aggregate
//!   deserialization
//! - **Version history** is a product requirement for debugging and rollback
//!
//! The "optional view" guidance applies to storage shape optimization
//! (`Archived*` types often suffice). Schema views solve fundamentally
//! different problems (metadata tracking, cache invalidation).
//!
//! [`RawSchema`]: crate::schema::raw::RawSchema
//! [`RawPropertyBank`]: crate::schema::raw::RawPropertyBank
//! [`Schema`]: crate::schema::aggregate::Schema
//! [`PropertyBank`]: crate::schema::bank::PropertyBank

#![expect(
    clippy::pub_use,
    reason = "Re-exports provide ergonomic API - users import from views, not \
              submodules"
)]

pub mod contracts;
pub mod hashes;
pub mod raw;
pub mod snapshots;

// Re-export commonly used types for ergonomic access
pub use contracts::{RawView, RawViewRead, Version, VersionRead};
pub use hashes::HashRecord;
pub use raw::{RawPropertyBankView, RawSchemaView};
pub use snapshots::{PropertyBankVersion, SchemaVersion};
