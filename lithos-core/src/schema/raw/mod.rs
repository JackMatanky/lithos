//! Raw schema and property input definitions (syntax validation layer).
//!
//! This module provides the data structures used for the initial ingestion of
//! schema files and property banks from the vault. It focuses on **syntax-only
//! validation** using the type system and custom deserializers to ensure that
//! invalid states are unrepresentable.
//!
//! ## Validation Strategy
//!
//! This module implements a **two-phase validation** approach following the
//! "Parse, Don't Validate" principle (Alexis King) and the Rust API Guidelines.
//!
//! ### Phase 1: Syntax Validation (During Deserialization)
//!
//! Validation that happens without external context using custom `Deserialize`
//! implementations:
//!
//! - **Schema version**: Must be "1.0" (validated by
//!   `RawSchemaVersion::deserialize`)
//! - **Property name format**: Regex `^[A-Za-z_][A-Za-z0-9_-]*$` (validated by
//!   `PropertyName::deserialize` in `RawPropertyMap` keys)
//! - **Schema name format** (in `extends` field): Regex `^[a-z0-9_-]+$`
//!   (validated by `SchemaName::deserialize`)
//! - **Exclude property names**: Same as property name format (validated by
//!   `PropertyName::deserialize`)
//! - **Field presence**: Required fields enforced by serde
//! - **Unknown fields**: Rejected by `#[serde(deny_unknown_fields)]`
//! - **Property definitions**: Per-type DTOs (`RawStringProperty`,
//!   `RawNumberProperty`, etc.) reject unknown fields to mirror
//!   `additionalProperties: false` in the meta-schema
//! - **Boolean tag compatibility**: `type = "bool"` and `type = "boolean"` are
//!   accepted on input; serialization uses `"bool"`
//!
//! These validations provide **immediate feedback** with line/column context
//! from serde, making invalid syntax unrepresentable at the type level.
//!
//! ### Phase 2: Semantic Validation (Post-Deserialization)
//!
//! Validation that requires external context or cross-schema checks:
//!
//! - **Schema name matches filename** (validated in `RawSchema::validated()`)
//! - **Property references exist** (validated by `Expander`)
//! - **Inheritance sanity** (validated by schema processing pipeline)
//!
//! These validations happen after deserialization with **file path context**
//! for better error messages.
//!
//! ### Special Case: Filename Validation
//!
//! The schema `name` field is **not deserialized** from the file - it's derived
//! from the filename by the `Ingestor`. Therefore, it:
//! - Remains as `Box<str>` (not `SchemaName`)
//! - Is validated in `validated()` with full file path context
//! - Provides better error messages: "schemas/invalid-name!.toml has invalid
//!   filename"
//!
//! ## Error Context Preservation
//!
//! - **Deserialization errors**: Serde provides line/column context
//!   automatically
//! - **Validation errors**: Wrapped in `SchemaIngestionError` with file path
//! - **Filename errors**: Include full path for clear user feedback
//!
//! ## Performance
//!
//! Validation overhead (~10%) is negligible compared to file I/O (100x slower).
//! This design prioritizes **error quality** over micro-optimization per the
//! Apollo Performance Mindset.
//!
//! ## References
//!
//! - ["Parse, Don't Validate" by Alexis King](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
//! - [Rust API Guidelines - Type Safety](https://rust-lang.github.io/api-guidelines/type-safety.html)
//! - [Serde Official Documentation](https://serde.rs/impl-deserialize.html)
//!
//! ## Raw Type Families
//!
//! Raw DTOs are split into two families:
//!
//! - **Inline property definitions**: `RawProperty*` types live in
//!   `raw/property.rs`. These are full per-type definitions used in schema
//!   files and property bank files (`required`, `multi`, and type-specific
//!   fields).
//! - **Override/constraint bundles**: `Raw*Spec` types live in
//!   `raw/{bool,date,file,number,string}.rs`. These are partial shapes used for
//!   reference overrides and `apply_overrides` in the domain layer.
//!
//! Semantic warnings (e.g., `required` in property bank entries, or empty/
//! duplicate string options) are emitted during domain construction rather than
//! at raw parse time.

#![expect(
    clippy::module_name_repetitions,
    clippy::pub_use,
    reason = "Raw* types follow naming conventions; re-exports provide \
              unified input layer API"
)]

pub mod bank;
pub mod bool;
pub mod date;
pub mod file;
pub mod number;
pub mod property;
pub mod string;
pub mod version;

mod aggregate;

// ─────────────────────────────────────────────────────────────────────────────
// Re-exports
// ─────────────────────────────────────────────────────────────────────────────

pub use aggregate::RawSchema;
pub use bank::RawPropertyBank;
pub use bool::{RawBoolProperty, RawBoolSpec};
pub use date::{RawDateProperty, RawDateSpec};
pub use file::{RawFileProperty, RawFileSpec};
pub use number::{RawNumberProperty, RawNumberSpec};
pub use property::{
    RawProperty, RawPropertyBankEntry, RawPropertyInline, RawPropertyMap,
    RawPropertyRef, RawPropertyRefPath,
};
pub use string::{
    RawEntryInputOrder, RawEntryValue, RawOptions, RawStringFormat,
    RawStringPattern, RawStringProperty, RawStringSpec,
};
pub use version::RawSchemaVersion;
