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
    reason = "Raw* types follow naming conventions for input layer types"
)]

pub mod bank;
pub mod bool;
pub mod date;
pub mod file;
pub mod metadata;
pub mod number;
pub mod property;
pub mod string;
pub mod version;

mod aggregate;

// ─────────────────────────────────────────────────────────────────────────────
// Type Aliases (Re-exports)
// ─────────────────────────────────────────────────────────────────────────────

/// Raw schema definition loaded from vault files.
pub type RawSchema = aggregate::RawSchema;

/// Raw property bank loaded from vault files.
pub type RawPropertyBank = bank::RawPropertyBank;

/// Raw file timestamps for staleness detection.
pub type RawFileTimes = metadata::RawFileTimes;

/// Schema format version.
pub type RawSchemaVersion = version::RawSchemaVersion;

/// Raw property for schema properties map.
pub type RawProperty = property::RawProperty;

/// Entry in the raw property bank.
pub type RawPropertyBankEntry = property::RawPropertyBankEntry;

/// Inline variant of a raw property.
pub type RawPropertyInline = property::RawPropertyInline;

/// Raw boolean property definition.
pub type RawBoolProperty = bool::RawBoolProperty;

/// Raw string property definition.
pub type RawStringProperty = string::RawStringProperty;

/// Raw number property definition.
pub type RawNumberProperty = number::RawNumberProperty;

/// Raw date property definition.
pub type RawDateProperty = date::RawDateProperty;

/// Raw file property definition.
pub type RawFileProperty = file::RawFileProperty;

/// Reference variant of a raw property with optional overrides.
pub type RawPropertyRef = property::RawPropertyRef;

/// Validated property map that guarantees all keys are valid `PropertyNames`.
pub type RawPropertyMap<T> = property::RawPropertyMap<T>;

/// Validated reference path to a property bank entry.
pub type RawPropertyRefPath = property::RawPropertyRefPath;

/// Boolean property definition (marker type).
pub type RawBoolSpec = bool::RawBoolSpec;

/// Date property definition.
pub type RawDateSpec = date::RawDateSpec;

/// File property definition.
pub type RawFileSpec = file::RawFileSpec;

/// Number property definition.
pub type RawNumberSpec = number::RawNumberSpec;

/// String property definition.
pub type RawStringSpec = string::RawStringSpec;

/// Raw options definition supporting three formats.
pub type RawOptions = string::RawOptions;

/// A rich option entry with optional label and input order.
pub type RawEntryValue = string::RawEntryValue;

/// Input order position.
pub type RawEntryInputOrder = string::RawEntryInputOrder;

/// Named string format for common validation patterns.
pub type RawStringFormat = string::RawStringFormat;

/// Raw string pattern supporting both custom regex and predefined formats.
pub type RawStringPattern = string::RawStringPattern;
