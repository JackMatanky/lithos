//! View types for schema staleness detection and version tracking.
//!
//! ## Purpose
//!
//! This module provides **rkyv-serializable metadata containers** for tracking
//! versions, hashes, and inheritance metadata extracted from **Raw\* types**
//! ([`RawSchema`], [`RawPropertyBank`]) which use **serde-only** serialization.
//!
//! **Primary architectural constraint**: Raw\* types do **not** have `rkyv`
//! derives—they exist solely for **parsing** human-editable file formats
//! (YAML, JSON, TOML). To persist staleness detection metadata alongside these
//! parsed structures, we need separate view types that **do** support `rkyv`.
//!
//! Views enable **incremental schema updates** in Lithos's
//! "files-as-source-of-truth" architecture by tracking content hashes, file
//! timestamps, and version history to answer "Has this file changed?" **without
//! re-parsing**, delivering massive performance wins for large vaults (1000+
//! schemas).
//!
//! Views solve critical architectural problems that cannot be deferred:
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
//! │  PARSE: File → Raw* (serde-only, syntax validation)         │
//! │  serde::from_str() → RawSchema, RawPropertyBank             │
//! │  (Raw* types do NOT have rkyv derives)                      │
//! └───────────────────────┬─────────────────────────────────────┘
//!                         │
//!                         ├─────────────────────────────────────┐
//!                         │                                     │
//!                         ▼                                     ▼
//! ┌──────────────────────────────────┐  ┌─────────────────────────────┐
//! │  EXTRACT METADATA (THIS MODULE)  │  │  VALIDATE: Raw* → Domain    │
//! │  RawSchema → SchemaVersion       │  │  Schema::try_from(RawSchema)│
//! │  • File stats, hashes            │  │  (Domain types have rkyv)   │
//! │  • Inheritance metadata          │  └──────────┬──────────────────┘
//! │  • Bank references               │             │
//! │                                  │             │
//! │  Wrap in versioned container:    │             │
//! │  RawSchemaView (ring buffer)     │             │
//! └──────────────────┬───────────────┘             │
//!                    │                             │
//!                    │  Both persisted separately  │
//!                    ▼                             ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │  DATABASE (rkyv-only storage)                               │
//! │  • Views: RawSchemaView, RawPropertyBankView (rkyv)         │
//! │  • Domains: Schema, PropertyBank (rkyv)                     │
//! │                                                             │
//! │  Views enable staleness checks WITHOUT re-parsing Raw*      │
//! └─────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────┐
//! │  STALENESS CHECK (On Next Load)                             │
//! │                                                             │
//! │  1. Compute current file hash                               │
//! │  2. Load RawSchemaView from database (zero-copy)            │
//! │  3. Compare: view.is_content_match(&hash)                   │
//! │  4. Match → use cached Schema (skip parsing)                │
//! │  5. Mismatch → re-parse RawSchema, update view & domain     │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Core Concepts
//!
//! ### Staleness Detection
//!
//! Views enable a three-tier staleness detection strategy:
//!
//! - **Fast Path**: Timestamp comparison via `FileMetadata::is_timestamp_match`
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
//! ## Why Views Exist: The Raw\* Serialization Constraint
//!
//! Views exist because **Raw\* types cannot be directly persisted**:
//!
//! ```text
//! ❌ CANNOT DO THIS:
//!    RawSchema (serde only) → Database (requires rkyv)
//!    └─ No rkyv derives on Raw* types by design
//!
//! ✅ SOLUTION:
//!    RawSchema (serde) → Extract metadata → SchemaVersion (rkyv) → Database
//!    └─ Views bridge the serialization gap
//! ```
//!
//! **Why don't Raw\* types have rkyv derives?**
//!
//! 1. **Separation of concerns**: Raw\* types are **parsing-only** artifacts
//!    optimized for serde's tolerant parsing (`Option<T>` fields, better error
//!    messages).
//!
//! 2. **Avoid dual serialization**: Adding rkyv to Raw\* types would create
//!    maintenance burden (two serialization strategies on same type).
//!
//! 3. **Raw\* are transient**: They exist only during ingestion pipeline, then
//!    convert to domain types ([`Schema`], [`PropertyBank`]) which **do** have
//!    rkyv.
//!
//! **Views solve this by**:
//! - Extracting **metadata-only** from Raw\* types during ingestion
//! - Storing that metadata in **rkyv-serializable** containers
//! - Enabling staleness checks **without re-parsing** Raw\* from files
//!
//! ## Hybrid Serialization Strategy
//!
//! Views sit between Raw\* types (serde-only) and the database (rkyv-only):
//!
//! 1. **Raw Layer** (serde only): [`RawSchema`], [`RawPropertyBank`] — Parsed
//!    from files, never persisted directly.
//!
//! 2. **View Layer** (rkyv only): [`RawSchemaView`], [`RawPropertyBankView`] —
//!    Metadata extracted from Raw\*, persisted for staleness detection.
//!
//! 3. **Domain Layer** (rkyv + optional serde): [`Schema`], [`PropertyBank`] —
//!    Validated aggregates created via `TryFrom<Raw*>`, persisted separately.
//!
//! Views use **rkyv exclusively** because:
//! - No human editing required (internal storage format)
//! - Zero-copy deserialization critical for performance (hot path queries)
//! - Stability: View schema changes less frequently than Raw\* types
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
pub mod properties;
pub mod raw;
pub mod snapshots;

// Re-export commonly used types for ergonomic access
pub(crate) use contracts::{RawView, RawViewRead};
pub use hashes::{HashRecord, RawPropertyHashIndex};
pub use properties::BasePropertiesView;
pub use raw::{RawPropertyBankView, RawSchemaView};
pub use snapshots::{PropertyBankVersion, SchemaVersion};
