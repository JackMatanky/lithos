//! Property specification variants and validation logic.

#![allow(
    clippy::module_name_repetitions,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "Core domain logic with naming convention. rkyv generates \
              exhaustive Archived types despite #[non_exhaustive] on source \
              types. Spec suffix is descriptive"
)]

use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
    path::{Component, Path},
    sync::{Arc, OnceLock, RwLock},
};

use super::error::SchemaError;
use crate::bounds::{Bounds, BoundsError};

// === Public API ===
//
// This module uses a two-layer model:
// - Raw spec definitions live in `schema::raw` (serde-friendly input).
// - `*Spec`: validated runtime constraints (invariants enforced at
//   construction).
//
// Internal invariant helpers live near the bottom of the file.

/// A validated option entry with optional display label.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct OptionEntry {
    /// The option value used in validation.
    pub value: Box<str>,
    /// Optional display label for UI consumers.
    pub label: Option<Box<str>>,
}

/// Supported property specification types.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum PropertySpecType {
    /// Boolean type.
    Bool,
    /// Date type.
    Date,
    /// File reference type.
    File,
    /// Numeric type.
    Number,
    /// String type.
    String,
}

// --- Validated runtime types: *Spec ---

/// Validated sum type for all supported property specifications.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum PropertySpec {
    /// Boolean property constraints.
    Bool(BoolSpec),
    /// Date property constraints.
    Date(DateSpec),
    /// File property constraints.
    File(FileSpec),
    /// Number property constraints.
    Number(NumberSpec),
    /// String property constraints.
    String(StringSpec),
}

impl PropertySpec {
    /// Get the spec type identifier.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &enum are intentional here for \
                  readability"
    )]
    pub fn spec_type(&self) -> PropertySpecType {
        match self {
            Self::Bool(_) => PropertySpecType::Bool,
            Self::Date(_) => PropertySpecType::Date,
            Self::File(_) => PropertySpecType::File,
            Self::Number(_) => PropertySpecType::Number,
            Self::String(_) => PropertySpecType::String,
        }
    }

    /// Feed spec content into a blake3 hasher for stable hashing.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &enum are intentional here for \
                  readability"
    )]
    pub fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        match self {
            Self::Bool(spec) => {
                hasher.update(&[0u8]);
                spec.hash_into_blake3(hasher);
            }
            Self::Date(spec) => {
                hasher.update(&[1u8]);
                spec.hash_into_blake3(hasher);
            }
            Self::File(spec) => {
                hasher.update(&[2u8]);
                spec.hash_into_blake3(hasher);
            }
            Self::Number(spec) => {
                hasher.update(&[3u8]);
                spec.hash_into_blake3(hasher);
            }
            Self::String(spec) => {
                hasher.update(&[4u8]);
                spec.hash_into_blake3(hasher);
            }
        }
    }

    /// Validate a value against this spec's constraints.
    ///
    /// This method uses `serde_json::Value` as a universal Intermediate
    /// Representation (IR) for metadata values, allowing validation of data
    /// loaded from JSON, YAML, or TOML.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::schema::raw::{RawPropertySpec, BoolSpecDef};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let def = RawPropertySpec::Bool(BoolSpecDef::default());
    /// let spec = def.try_into_validated()?;
    /// spec.validate(&serde_json::json!(true))?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &enum are intentional here for \
                  readability"
    )]
    pub fn validate(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), SchemaError> {
        match self {
            Self::Bool(_) => {
                if !value.is_boolean() {
                    return Err(SchemaError::InvalidType {
                        value: value.to_string(),
                        expected: "boolean".into(),
                    });
                }
                Ok(())
            }
            Self::Date(s) => {
                let val =
                    value.as_str().ok_or_else(|| SchemaError::InvalidType {
                        value: value.to_string(),
                        expected: "string (date)".into(),
                    })?;
                s.validate_str(val)
            }
            Self::File(s) => {
                let val =
                    value.as_str().ok_or_else(|| SchemaError::InvalidType {
                        value: value.to_string(),
                        expected: "string (file path)".into(),
                    })?;
                s.validate_str(val)
            }
            Self::Number(s) => {
                let n =
                    value.as_f64().ok_or_else(|| SchemaError::InvalidType {
                        value: value.to_string(),
                        expected: "number".into(),
                    })?;
                s.validate_value(n)
            }
            Self::String(s) => {
                let val =
                    value.as_str().ok_or_else(|| SchemaError::InvalidType {
                        value: value.to_string(),
                        expected: "string".into(),
                    })?;
                s.validate_str(val)
            }
        }
    }
}

/// Boolean property constraints (marker type).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct BoolSpec;

impl BoolSpec {
    /// Feed spec content into a blake3 hasher for stable hashing.
    #[inline]
    pub fn hash_into_blake3(&self, _hasher: &mut blake3::Hasher) {
        // BoolSpec is a marker type with no fields
    }
}

/// Date property validation constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct DateSpec {
    format: Box<str>,
}

impl DateSpec {
    /// Create a validated `DateSpec`.
    ///
    /// # Errors
    /// Returns `SchemaError::InvalidDateFormat` if the format is empty.
    #[inline]
    pub fn try_new(format: &str) -> Result<Self, SchemaError> {
        if format.is_empty() {
            return Err(SchemaError::InvalidDateFormat(
                "Format cannot be empty".into(),
            ));
        }
        Ok(Self {
            format: format.into(),
        })
    }

    #[inline]
    fn validate_str(&self, value: &str) -> Result<(), SchemaError> {
        let is_valid =
            chrono::NaiveDateTime::parse_from_str(value, &self.format).is_ok()
                || chrono::NaiveDate::parse_from_str(value, &self.format)
                    .is_ok();

        if !is_valid {
            return Err(SchemaError::InvalidDateFormat(format!(
                "Value {value} does not match format {}",
                self.format
            )));
        }
        Ok(())
    }

    /// Feed spec content into a blake3 hasher for stable hashing.
    #[inline]
    pub fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.format.as_bytes());
    }
}

/// File property validation constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct FileSpec {
    directory: Option<VaultRelPath>,
    file_class: Option<Box<str>>,
}

impl FileSpec {
    /// Create a validated `FileSpec`.
    ///
    /// # Errors
    /// Returns `SchemaError` if the directory path is not vault-relative or if
    /// `file_class` is present but empty.
    #[inline]
    pub fn try_new(
        directory: Option<String>,
        file_class: Option<String>,
    ) -> Result<Self, SchemaError> {
        let directory = match directory {
            Some(dir) => Some(VaultRelPath::try_new(&dir)?),
            None => None,
        };

        if let Some(fc) = file_class.as_ref()
            && fc.is_empty()
        {
            return Err(SchemaError::InvalidFileClass(
                "File class cannot be empty".into(),
            ));
        }

        Ok(Self {
            directory,
            file_class: file_class.map(String::into_boxed_str),
        })
    }

    #[inline]
    fn validate_str(&self, value: &str) -> Result<(), SchemaError> {
        validate_vault_rel_path(value)?;

        if let Some(dir) = self.directory.as_ref() {
            let value_path = Path::new(value);
            let dir_path = Path::new(dir.as_str());

            if value_path == dir_path || !value_path.starts_with(dir_path) {
                return Err(SchemaError::InvalidDirectoryPath(format!(
                    "File {value} must be in directory {}",
                    dir.as_str()
                )));
            }
        }

        Ok(())
    }

    /// Feed spec content into a blake3 hasher for stable hashing.
    #[inline]
    pub fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        if let Some(dir) = self.directory.as_ref() {
            hasher.update(&[1u8]);
            hasher.update(dir.as_str().as_bytes());
        } else {
            hasher.update(&[0u8]);
        }
        if let Some(fc) = self.file_class.as_ref() {
            hasher.update(&[1u8]);
            hasher.update(fc.as_bytes());
        } else {
            hasher.update(&[0u8]);
        }
    }
}

/// Number property validation constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[non_exhaustive]
pub struct NumberSpec {
    bounds: Bounds<FiniteF64>,
    step: Option<Step>,
}

impl Default for NumberSpec {
    #[inline]
    fn default() -> Self {
        Self {
            bounds: Bounds::Unbounded,
            step: None,
        }
    }
}

impl NumberSpec {
    /// Create a validated `NumberSpec`.
    ///
    /// # Errors
    /// Returns `SchemaError` if `min`, `max`, or `step` are non-finite, if
    /// `min > max`, or if `step` is non-positive.
    #[inline]
    pub fn try_new(
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
    ) -> Result<Self, SchemaError> {
        let min = match min {
            Some(v) => Some(FiniteF64::try_new(v, "min")?),
            None => None,
        };
        let max = match max {
            Some(v) => Some(FiniteF64::try_new(v, "max")?),
            None => None,
        };

        let bounds = match Bounds::from_options(min, max) {
            None => Bounds::Unbounded,
            Some(Ok(bounds)) => bounds,
            Some(Err(BoundsError::InvalidRange)) => {
                return Err(SchemaError::ValidationFailed(
                    "min cannot be greater than max".into(),
                ));
            }
        };

        let step = match step {
            Some(v) => Some(Step::try_new(v)?),
            None => None,
        };

        Ok(Self {
            bounds,
            step,
        })
    }

    /// Validates a numeric value against range and step constraints.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validate_value(&self, value: f64) -> Result<(), SchemaError> {
        let finite = FiniteF64::try_new(value, "value").map_err(|_err| {
            SchemaError::ValidationFailed(format!(
                "Value {value} is not finite"
            ))
        })?;

        self.validate_range(finite)?;
        self.validate_step(finite)?;
        Ok(())
    }

    /// Validates that a numeric value falls within optional min/max bounds.
    #[inline]
    fn validate_range(&self, finite: FiniteF64) -> Result<(), SchemaError> {
        if !self.bounds.validate(finite) {
            let min = self.bounds.min().map(FiniteF64::get);
            let max = self.bounds.max().map(FiniteF64::get);
            return Err(SchemaError::NumberOutOfRange {
                value: finite.get(),
                min,
                max,
            });
        }
        Ok(())
    }

    /// Validates that a numeric value aligns with a step increment.
    #[inline]
    #[expect(
        clippy::float_arithmetic,
        clippy::modulo_arithmetic,
        reason = "Core numeric validation logic with epsilon comparison"
    )]
    fn validate_step(&self, finite: FiniteF64) -> Result<(), SchemaError> {
        const EPSILON: f64 = 1e-10;
        let value = finite.get();

        if let Some(step) = self.step {
            let base = self.bounds.min().map_or(0.0f64, FiniteF64::get);
            let offset = (value - base).abs();
            let remainder = offset % step.get();

            if remainder > EPSILON && (step.get() - remainder) > EPSILON {
                return Err(SchemaError::InvalidStepValue {
                    value,
                    step: step.get(),
                });
            }
        }
        Ok(())
    }

    /// Feed spec content into a blake3 hasher for stable hashing.
    #[inline]
    #[expect(
        clippy::little_endian_bytes,
        reason = "Little-endian is intentional for consistent hash values \
                  across platforms"
    )]
    pub fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        match self.bounds {
            Bounds::Unbounded => {
                hasher.update(&[0u8]);
            }
            Bounds::Min(min) => {
                hasher.update(&[1u8]);
                hasher.update(&min.get().to_le_bytes());
            }
            Bounds::Max(max) => {
                hasher.update(&[2u8]);
                hasher.update(&max.get().to_le_bytes());
            }
            Bounds::Range {
                min,
                max,
            } => {
                hasher.update(&[3u8]);
                hasher.update(&min.get().to_le_bytes());
                hasher.update(&max.get().to_le_bytes());
            }
        }
        if let Some(step) = self.step {
            hasher.update(&[1u8]);
            hasher.update(&step.get().to_le_bytes());
        } else {
            hasher.update(&[0u8]);
        }
    }
}

/// String property validation constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[non_exhaustive]
pub struct StringSpec {
    options: Option<Vec<OptionEntry>>,
    length: Bounds<usize>,
    pattern: Option<Box<str>>,
}

impl Default for StringSpec {
    #[inline]
    fn default() -> Self {
        Self {
            options: None,
            length: Bounds::Unbounded,
            pattern: None,
        }
    }
}

impl StringSpec {
    /// Create a validated `StringSpec`.
    ///
    /// # Errors
    /// Returns `SchemaError` if `min_length > max_length` or if `pattern` is
    /// present but not a valid regex.
    #[inline]
    pub fn try_new(
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<Box<str>>,
        options: Option<Vec<OptionEntry>>,
    ) -> Result<Self, SchemaError> {
        let length = match Bounds::from_options(min_length, max_length) {
            None => Bounds::Unbounded,
            Some(Ok(bounds)) => bounds,
            Some(Err(BoundsError::InvalidRange)) => {
                return Err(SchemaError::ValidationFailed(
                    "min_length cannot be greater than max_length".into(),
                ));
            }
        };

        let pattern = match pattern {
            Some(p) => {
                get_cached_regex(&p)?;
                Some(p)
            }
            None => None,
        };

        Ok(Self {
            options,
            length,
            pattern,
        })
    }

    #[inline]
    fn validate_str(&self, value: &str) -> Result<(), SchemaError> {
        self.validate_length(value)?;
        self.validate_options(value)?;
        self.validate_pattern(value)?;
        Ok(())
    }

    fn validate_options(&self, value: &str) -> Result<(), SchemaError> {
        if let Some(entries) = self.options.as_ref()
            && !entries.iter().any(|e| e.value.as_ref() == value)
        {
            return Err(SchemaError::InvalidEnumValue {
                value: value.into(),
                allowed: entries
                    .iter()
                    .map(|e| e.value.as_ref().into())
                    .collect(),
            });
        }
        Ok(())
    }

    /// Validates that a string length falls within optional min/max bounds.
    ///
    /// Length is measured in UTF-8 bytes (i.e., `value.len()`), not Unicode
    /// scalar values or grapheme clusters.
    ///
    /// # Errors
    /// Returns `SchemaError::StringTooShort` or `SchemaError::StringTooLong` if
    /// validation fails.
    #[inline]
    pub fn validate_length(&self, value: &str) -> Result<(), SchemaError> {
        let len = value.len();
        if !self.length.validate(len) {
            if let Some(min) = self.length.min()
                && len < min
            {
                return Err(SchemaError::StringTooShort {
                    min,
                    actual: len,
                });
            }
            if let Some(max) = self.length.max()
                && len > max
            {
                return Err(SchemaError::StringTooLong {
                    max,
                    actual: len,
                });
            }
        }
        Ok(())
    }

    fn validate_pattern(&self, value: &str) -> Result<(), SchemaError> {
        if let Some(pattern) = self.pattern.as_ref() {
            let re = get_cached_regex(pattern)?;
            if !re.is_match(value) {
                return Err(SchemaError::ValidationFailed(format!(
                    "Value {value} does not match pattern {pattern}"
                )));
            }
        }
        Ok(())
    }

    /// Feed spec content into a blake3 hasher for stable hashing.
    #[inline]
    #[expect(
        clippy::as_conversions,
        reason = "usize to u64 conversion for hash stability; usize <= u64 on \
                  all supported platforms"
    )]
    #[expect(
        clippy::little_endian_bytes,
        reason = "Little-endian is intentional for consistent hash values \
                  across platforms"
    )]
    pub fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        if let Some(entries) = self.options.as_ref() {
            hasher.update(&(entries.len() as u64).to_le_bytes());
            for entry in entries {
                hasher.update(entry.value.as_bytes());
                if let Some(label) = entry.label.as_ref() {
                    hasher.update(&[1u8]);
                    hasher.update(label.as_bytes());
                } else {
                    hasher.update(&[0u8]);
                }
            }
        } else {
            hasher.update(&0u64.to_le_bytes());
        }
        match self.length {
            Bounds::Unbounded => {
                hasher.update(&[0u8]);
            }
            Bounds::Min(min) => {
                hasher.update(&[1u8]);
                hasher.update(&(min as u64).to_le_bytes());
            }
            Bounds::Max(max) => {
                hasher.update(&[2u8]);
                hasher.update(&(max as u64).to_le_bytes());
            }
            Bounds::Range {
                min,
                max,
            } => {
                hasher.update(&[3u8]);
                hasher.update(&(min as u64).to_le_bytes());
                hasher.update(&(max as u64).to_le_bytes());
            }
        }
        if let Some(pattern) = self.pattern.as_ref() {
            hasher.update(&[1u8]);
            hasher.update(pattern.as_bytes());
        } else {
            hasher.update(&[0u8]);
        }
    }
}

// --- Internal helper types (type-driven invariants) ---

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(transparent)]
struct FiniteF64(f64);

impl FiniteF64 {
    #[inline]
    fn try_new(value: f64, ctx: &'static str) -> Result<Self, SchemaError> {
        if !value.is_finite() {
            return Err(SchemaError::ValidationFailed(format!(
                "{ctx} must be finite"
            )));
        }
        Ok(Self(value))
    }

    #[inline]
    const fn get(self) -> f64 {
        self.0
    }
}

impl std::hash::Hash for FiniteF64 {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }

    #[inline]
    fn hash_slice<H: std::hash::Hasher>(data: &[Self], state: &mut H) {
        for value in data {
            value.hash(state);
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(transparent)]
struct Step(FiniteF64);

impl Step {
    #[inline]
    fn try_new(value: f64) -> Result<Self, SchemaError> {
        let finite = FiniteF64::try_new(value, "step")?;
        if finite.get() <= 0.0f64 {
            return Err(SchemaError::ValidationFailed(
                "step must be positive".into(),
            ));
        }
        Ok(Self(finite))
    }

    #[inline]
    const fn get(self) -> f64 {
        self.0.get()
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(transparent)]
struct VaultRelPath(Box<str>);

impl VaultRelPath {
    #[inline]
    fn try_new(path: &str) -> Result<Self, SchemaError> {
        validate_vault_rel_path(path)?;
        Ok(Self(path.into()))
    }

    #[inline]
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[inline]
fn validate_vault_rel_path(path: &str) -> Result<(), SchemaError> {
    if path.is_empty() {
        return Err(SchemaError::InvalidDirectoryPath(
            "Path cannot be empty".into(),
        ));
    }

    for component in Path::new(path).components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {
                return Err(SchemaError::InvalidDirectoryPath(format!(
                    "Invalid path {path}: '.' component is not allowed"
                )));
            }
            Component::ParentDir => {
                return Err(SchemaError::InvalidDirectoryPath(format!(
                    "Invalid path {path}: '..' component is not allowed"
                )));
            }
            Component::RootDir => {
                return Err(SchemaError::InvalidDirectoryPath(format!(
                    "Invalid path {path}: absolute paths are not allowed"
                )));
            }
            Component::Prefix(_) => {
                return Err(SchemaError::InvalidDirectoryPath(format!(
                    "Invalid path {path}: path prefixes are not allowed"
                )));
            }
        }
    }

    Ok(())
}

type RegexCache = HashMap<Box<str>, Arc<regex::Regex>>;
type RegexCacheLock = RwLock<RegexCache>;

static REGEX_CACHE: OnceLock<RegexCacheLock> = OnceLock::new();

fn get_cached_regex(pattern: &str) -> Result<Arc<regex::Regex>, SchemaError> {
    let cache = REGEX_CACHE.get_or_init(|| RwLock::new(RegexCache::new()));

    // Fast path: read lock.
    {
        let guard = match cache.read() {
            Ok(guard) => guard,
            Err(e) => e.into_inner(),
        };

        if let Some(re) = guard.get(pattern) {
            return Ok(Arc::clone(re));
        }
    }

    // Slow path: compile without holding any locks.
    let compiled = Arc::new(regex::Regex::new(pattern).map_err(|e| {
        SchemaError::InvalidRegex(format!("Invalid pattern {pattern}: {e}"))
    })?);

    // Insert (or reuse) under a write lock.
    let mut guard = match cache.write() {
        Ok(guard) => guard,
        Err(e) => e.into_inner(),
    };

    match guard.entry(pattern.into()) {
        Entry::Occupied(entry) => Ok(Arc::clone(entry.get())),
        Entry::Vacant(entry) => {
            entry.insert(Arc::clone(&compiled));
            Ok(compiled)
        }
    }
}

#[cfg(test)]
mod tests {
    mod string_spec {
        use rstest::rstest;

        use crate::schema::{
            error::SchemaError,
            property_spec::StringSpec,
            raw::{RawOptions, StringSpecDef},
        };

        /// 3.3-UNIT-011: String Specification Validation Matrix.
        /// Priority: P1.
        #[rstest]
        #[case::options_match(
            StringSpecDef {
                options: Some(RawOptions::List(vec!["A".into(), "B".into()])),
                ..Default::default()
            },
            "A",
            Ok(())
        )]
        #[case::options_mismatch(
            StringSpecDef {
                options: Some(RawOptions::List(vec!["A".into(), "B".into()])),
                ..Default::default()
            },
            "C",
            Err(SchemaError::InvalidEnumValue {
                value: "C".to_owned(),
                allowed: vec!["A".to_owned(), "B".to_owned()]
            })
        )]
        #[case::regex_match(
            StringSpecDef { pattern: Some(r"^\d+$".into()), ..Default::default() },
            "123",
            Ok(())
        )]
        #[case::regex_mismatch(
            StringSpecDef { pattern: Some(r"^\d+$".into()), ..Default::default() },
            "abc",
            Err(SchemaError::ValidationFailed("Value abc does not match pattern ^\\d+$".to_owned()))
        )]
        #[case::length_match(
            StringSpecDef { min_length: Some(2), max_length: Some(5), ..Default::default() },
            "abc",
            Ok(())
        )]
        #[case::length_utf8_bytes_match(
            StringSpecDef { min_length: Some(5), max_length: Some(5), ..Default::default() },
            "caf\u{00e9}",
            Ok(())
        )]
        #[case::length_utf8_bytes_too_long(
            StringSpecDef { max_length: Some(4), ..Default::default() },
            "caf\u{00e9}",
            Err(SchemaError::StringTooLong { max: 4, actual: 5 })
        )]
        #[case::too_short(
            StringSpecDef { min_length: Some(2), ..Default::default() },
            "a",
            Err(SchemaError::StringTooShort { min: 2, actual: 1 })
        )]
        #[case::too_long(
            StringSpecDef { max_length: Some(5), ..Default::default() },
            "abcdef",
            Err(SchemaError::StringTooLong { max: 5, actual: 6 })
        )]
        fn string_spec_validation_matrix(
            #[case] def: StringSpecDef,
            #[case] value: &str,
            #[case] expected: Result<(), SchemaError>,
        ) {
            fn validated_spec(def: StringSpecDef) -> StringSpec {
                StringSpec::try_new(
                    def.min_length,
                    def.max_length,
                    def.pattern,
                    def.options.map(RawOptions::into_entries),
                )
                .expect("Expected valid StringSpecDef")
            }

            let spec = validated_spec(def);

            // WHEN: validating a string value
            let result = spec.validate_str(value);

            // THEN: the result matches the expectation
            assert_eq!(
                result, expected,
                "String validation failed for value='{value}': expected \
                 {expected:?}, got {result:?}"
            );
        }
    }

    mod number_spec {
        use rstest::rstest;

        use crate::schema::{
            error::SchemaError, property_spec::NumberSpec, raw::NumberSpecDef,
        };

        fn validated_spec(def: &NumberSpecDef) -> NumberSpec {
            NumberSpec::try_new(def.min, def.max, def.step)
                .expect("Expected valid NumberSpecDef")
        }

        /// 3.3-UNIT-012: Number Specification Validation Matrix.
        /// Priority: P1.
        #[rstest]
        #[case::in_range(
            NumberSpecDef { min: Some(0.0f64), max: Some(10.0f64), step: None },
            5.0f64,
            Ok(())
        )]
        #[case::at_min(
            NumberSpecDef { min: Some(0.0f64), max: Some(10.0f64), step: None },
            0.0f64,
            Ok(())
        )]
        #[case::at_max(
            NumberSpecDef { min: Some(0.0f64), max: Some(10.0f64), step: None },
            10.0f64,
            Ok(())
        )]
        #[case::below_min(
            NumberSpecDef { min: Some(0.0f64), max: Some(10.0f64), step: None },
            -1.0f64,
            Err(SchemaError::NumberOutOfRange {
                value: -1.0f64,
                min: Some(0.0f64),
                max: Some(10.0f64)
            })
        )]
        #[case::above_max(
            NumberSpecDef { min: Some(0.0f64), max: Some(10.0f64), step: None },
            11.0f64,
            Err(SchemaError::NumberOutOfRange {
                value: 11.0f64,
                min: Some(0.0f64),
                max: Some(10.0f64)
            })
        )]
        #[case::valid_step(
            NumberSpecDef { min: Some(0.0f64), max: None, step: Some(0.5f64) },
            5.5f64,
            Ok(())
        )]
        #[case::invalid_step(
            NumberSpecDef { min: Some(0.0f64), max: None, step: Some(0.5f64) },
            5.2f64,
            Err(SchemaError::InvalidStepValue { value: 5.2f64, step: 0.5f64 })
        )]
        fn number_spec_validation_matrix(
            #[case] def: NumberSpecDef,
            #[case] value: f64,
            #[case] expected: Result<(), SchemaError>,
        ) {
            let spec = validated_spec(&def);

            // WHEN: validating a numeric value
            let result = spec.validate_value(value);

            // THEN: the result matches the expectation
            assert_eq!(
                result, expected,
                "Number validation failed for value={value}: expected \
                 {expected:?}, got {result:?}"
            );
        }

        #[test]
        fn number_spec_rejects_min_greater_than_max() {
            let result = NumberSpec::try_new(Some(10.0f64), Some(5.0f64), None);
            assert!(
                matches!(result, Err(SchemaError::ValidationFailed(_))),
                "Expected ValidationFailed for min > max, got: {result:?}"
            );
        }

        #[test]
        fn number_spec_accepts_valid_bounds() {
            let result =
                NumberSpec::try_new(Some(5.0f64), Some(10.0f64), Some(1.0f64));
            assert!(
                result.is_ok(),
                "Valid NumberSpec should succeed, got error: {:?}",
                result.err()
            );
        }

        #[test]

        fn number_spec_rejects_nan_value() {
            let spec = NumberSpec::try_new(Some(0.0f64), Some(10.0f64), None)
                .expect("Expected valid NumberSpec");
            let result = spec.validate_value(f64::NAN);
            assert!(
                matches!(result, Err(SchemaError::ValidationFailed(_))),
                "Expected ValidationFailed for NaN, got: {result:?}"
            );
        }

        #[test]

        fn number_spec_rejects_infinite_value() {
            let spec = NumberSpec::try_new(Some(0.0f64), Some(10.0f64), None)
                .expect("Expected valid NumberSpec");
            let result = spec.validate_value(f64::INFINITY);
            assert!(
                matches!(result, Err(SchemaError::ValidationFailed(_))),
                "Expected ValidationFailed for infinity, got: {result:?}"
            );
        }

        #[test]
        fn number_spec_rejects_non_finite_min_bound() {
            let result =
                NumberSpec::try_new(Some(f64::NAN), Some(10.0f64), None);
            assert!(
                matches!(result, Err(SchemaError::ValidationFailed(_))),
                "Expected ValidationFailed for non-finite min, got: {result:?}"
            );
        }

        #[test]
        fn number_spec_rejects_non_finite_step() {
            let result = NumberSpec::try_new(
                Some(-10.0f64),
                Some(10.0f64),
                Some(f64::INFINITY),
            );
            assert!(
                matches!(result, Err(SchemaError::ValidationFailed(_))),
                "Expected ValidationFailed for non-finite step, got: \
                 {result:?}"
            );
        }
    }

    mod file_spec {
        use rstest::rstest;

        use crate::schema::{error::SchemaError, property_spec::FileSpec};

        fn validated_spec_with_dir(dir: &str) -> FileSpec {
            FileSpec::try_new(Some(dir.to_owned()), None)
                .expect("Expected valid FileSpec")
        }

        /// 3.3-UNIT-013: File Specification Validation Matrix.
        /// Priority: P1.
        #[rstest]
        #[case::in_dir("notes/my_note.md", "notes/", Ok(()))]
        #[case::out_dir(
            "other/note.md",
            "notes/",
            Err(SchemaError::InvalidDirectoryPath(
                "File other/note.md must be in directory notes/".to_owned(),
            ))
        )]
        fn file_spec_validation_matrix(
            #[case] path: &str,
            #[case] dir: &str,
            #[case] expected: Result<(), SchemaError>,
        ) {
            let spec = validated_spec_with_dir(dir);

            // WHEN: validating file paths
            let result = spec.validate_str(path);

            // THEN: the result matches the expectation
            assert_eq!(
                result, expected,
                "FileSpec validation result should match expected for path: \
                 {path}"
            );
        }

        #[test]
        fn file_spec_rejects_prefix_bypass() {
            let spec = validated_spec_with_dir("notes/");

            let result = spec.validate_str("notes_evil/note.md");
            assert!(matches!(
                result,
                Err(SchemaError::InvalidDirectoryPath(_))
            ));
        }

        #[test]
        fn file_spec_rejects_parent_dir_traversal() {
            let spec = validated_spec_with_dir("notes/");

            let result = spec.validate_str("../notes/note.md");
            assert!(matches!(
                result,
                Err(SchemaError::InvalidDirectoryPath(_))
            ));
        }

        #[test]
        fn file_spec_rejects_absolute_paths() {
            let spec = validated_spec_with_dir("notes/");

            let result = spec.validate_str("/notes/note.md");
            assert!(matches!(
                result,
                Err(SchemaError::InvalidDirectoryPath(_))
            ));
        }

        #[test]
        fn file_spec_rejects_value_equal_to_directory() {
            let spec = validated_spec_with_dir("notes/");

            let result = spec.validate_str("notes/");
            assert!(matches!(
                result,
                Err(SchemaError::InvalidDirectoryPath(_))
            ));
        }

        #[test]
        fn file_spec_validates_file_class_format() {
            // GIVEN: a valid file_class spec
            let result =
                FileSpec::try_new(None, Some("any-schema-name".to_owned()));
            assert!(
                result.is_ok(),
                "Expected valid file_class spec, got: {result:?}"
            );
        }

        #[test]
        fn file_spec_rejects_empty_file_class() {
            // GIVEN: an empty file_class spec
            let result = FileSpec::try_new(None, Some(String::new()));

            // THEN: it should be invalid
            assert!(
                result.is_err(),
                "Expected invalid file_class spec, got: {result:?}"
            );
        }
    }

    mod bool_spec {
        use crate::schema::property_spec::{BoolSpec, PropertySpec};

        #[test]
        fn bool_spec_accepts_true() {
            let spec = PropertySpec::Bool(BoolSpec::default());
            let result = spec.validate(&serde_json::Value::Bool(true));

            assert!(
                result.is_ok(),
                "Expected bool validation to succeed, got: {result:?}"
            );
        }

        #[test]
        fn bool_spec_accepts_false() {
            let spec = PropertySpec::Bool(BoolSpec::default());
            let result = spec.validate(&serde_json::Value::Bool(false));

            assert!(
                result.is_ok(),
                "Expected bool validation to succeed, got: {result:?}"
            );
        }
    }

    mod date_spec {
        use crate::schema::{error::SchemaError, property_spec::DateSpec};

        #[test]

        fn date_spec_accepts_valid_date() {
            let spec = DateSpec::try_new("%Y-%m-%dT%H:%M:%SZ")
                .expect("Expected valid DateSpec");
            let result = spec.validate_str("2024-01-15T14:30:00Z");
            assert!(
                result.is_ok(),
                "Expected date validation to succeed, got: {result:?}"
            );
        }

        #[test]

        fn date_spec_rejects_invalid_date() {
            let spec = DateSpec::try_new("%Y-%m-%dT%H:%M:%SZ")
                .expect("Expected valid DateSpec");
            let result = spec.validate_str("not-a-date");
            assert!(
                matches!(result, Err(SchemaError::InvalidDateFormat(_))),
                "Expected InvalidDateFormat error for invalid date string, \
                 got: {result:?}"
            );
        }
    }

    mod property_spec {
        use crate::schema::{
            property_spec::{PropertySpec, PropertySpecType},
            raw::{BoolSpecDef, NumberSpecDef, RawPropertySpec, StringSpecDef},
        };

        fn bool_spec() -> PropertySpec {
            RawPropertySpec::Bool(BoolSpecDef::default())
                .try_into_validated()
                .expect("Expected default BoolSpec to validate")
        }

        fn string_spec() -> PropertySpec {
            RawPropertySpec::String(StringSpecDef::default())
                .try_into_validated()
                .expect("Expected default StringSpec to validate")
        }

        fn number_spec() -> PropertySpec {
            RawPropertySpec::Number(NumberSpecDef::default())
                .try_into_validated()
                .expect("Expected default NumberSpec to validate")
        }

        #[test]
        fn bool_spec_type_reports_bool() {
            let spec = bool_spec();
            assert_eq!(
                spec.spec_type(),
                PropertySpecType::Bool,
                "BoolSpec should return Bool type"
            );
        }

        #[test]
        fn string_spec_type_reports_string() {
            let spec = string_spec();
            assert_eq!(
                spec.spec_type(),
                PropertySpecType::String,
                "StringSpec should return String type"
            );
        }

        #[test]
        fn number_spec_type_reports_number() {
            let spec = number_spec();
            assert_eq!(
                spec.spec_type(),
                PropertySpecType::Number,
                "NumberSpec should return Number type"
            );
        }

        #[test]
        fn validate_dispatches_to_bool_spec() {
            let spec = bool_spec();
            let result = spec.validate(&serde_json::Value::Bool(true));
            assert!(
                result.is_ok(),
                "Bool validation should succeed, got error: {:?}",
                result.err()
            );
        }
    }
}
