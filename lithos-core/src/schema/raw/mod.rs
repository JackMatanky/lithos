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
//! - **Parent schema exists** (validated by `Loader`)
//! - **No circular inheritance** (validated by `Loader`)
//! - **Depth limits** (validated by `Resolver`)
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

#![expect(
    clippy::module_name_repetitions,
    reason = "Raw* types follow naming conventions for input layer types"
)]

mod aggregate;
pub mod property;
pub mod spec_bool;
pub mod spec_date;
pub mod spec_file;
pub mod spec_number;
pub mod spec_string;

// ─────────────────────────────────────────────────────────────────────────────
// Type Aliases (Re-exports)
// ─────────────────────────────────────────────────────────────────────────────

/// Raw schema definition loaded from vault files.
pub type RawSchema = aggregate::RawSchema;

/// Raw property bank loaded from vault files.
pub type RawPropertyBank = aggregate::RawPropertyBank;

/// Raw file timestamps for staleness detection.
pub type RawFileTimes = aggregate::RawFileTimes;

/// Schema format version.
pub type RawSchemaVersion = aggregate::RawSchemaVersion;

/// Raw property for schema properties map.
pub type RawProperty = property::RawProperty;

/// Entry in the raw property bank.
pub type RawPropertyBankEntry = property::RawPropertyBankEntry;

/// Inline variant of a raw property.
pub type RawPropertyInline = property::RawPropertyInline;

/// Reference variant of a raw property with optional overrides.
pub type RawPropertyRef = property::RawPropertyRef;

/// Validated property map that guarantees all keys are valid `PropertyNames`.
pub type RawPropertyMap<T> = property::RawPropertyMap<T>;

/// Validated reference path to a property bank entry.
pub type RawPropertyRefPath = property::RawPropertyRefPath;

/// Raw property specification (serde-facing input type).
pub type RawPropertySpec = property::RawPropertySpec;

/// Boolean property definition (marker type).
pub type RawBoolSpec = spec_bool::RawBoolSpec;

/// Date property definition.
pub type RawDateSpec = spec_date::RawDateSpec;

/// File property definition.
pub type RawFileSpec = spec_file::RawFileSpec;

/// Number property definition.
pub type RawNumberSpec = spec_number::RawNumberSpec;

/// String property definition.
pub type RawStringSpec = spec_string::RawStringSpec;

/// Raw options definition supporting three formats.
pub type RawOptions = spec_string::RawOptions;

/// Mode 1: Plain array of string values.
pub type RawOptionsList = spec_string::RawOptionsList;

/// Mode 2: Integer-keyed ordered object.
pub type RawOptionsMap = spec_string::RawOptionsMap;

/// Mode 3: Rich entries with optional label and order.
pub type RawOptionsRich = spec_string::RawOptionsRich;

/// Rich option entry with optional label and display order.
pub type RawOptionEntry = spec_string::RawOptionEntry;

/// Named string format for common validation patterns.
pub type RawStringFormat = spec_string::RawStringFormat;

/// Raw string pattern supporting both custom regex and predefined formats.
pub type RawStringPattern = spec_string::RawStringPattern;
