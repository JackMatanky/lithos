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
    path::{Component, Path},
    sync::{Arc, OnceLock, RwLock},
};

use super::error::SchemaError;

// === Public API ===
//
// This module uses a two-layer model:
// - `*SpecDef`: persisted/serde-friendly schema definitions.
// - `*Spec`: validated runtime constraints (invariants enforced at
//   construction).
//
// Internal invariant helpers live near the bottom of the file.

/// Supported property specification types.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize,
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

// --- Persisted schema types (Serde source-of-truth): *SpecDef ---

/// Persisted sum type for all supported property specifications.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum PropertySpecDef {
    /// Boolean property definition (marker type).
    Bool(BoolSpecDef),
    /// Date property definition.
    Date(DateSpecDef),
    /// File property definition.
    File(FileSpecDef),
    /// Number property definition.
    Number(NumberSpecDef),
    /// String property definition.
    String(StringSpecDef),
}

impl PropertySpecDef {
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

    /// Validate and compile a persisted definition into a validated spec.
    ///
    /// # Errors
    /// Returns `SchemaError` if the definition is invalid.
    #[inline]
    pub fn try_into_validated(self) -> Result<PropertySpec, SchemaError> {
        match self {
            Self::Bool(_) => Ok(PropertySpec::Bool(BoolSpec::default())),
            Self::Date(def) => {
                Ok(PropertySpec::Date(DateSpec::try_new(def.format)?))
            }
            Self::File(def) => Ok(PropertySpec::File(FileSpec::try_new(
                def.directory,
                def.file_class,
            )?)),
            Self::Number(def) => Ok(PropertySpec::Number(NumberSpec::try_new(
                def.min, def.max, def.step,
            )?)),
            Self::String(def) => Ok(PropertySpec::String(StringSpec::try_new(
                def.min_length,
                def.max_length,
                def.pattern,
                def.enum_values,
            )?)),
        }
    }
}

/// Boolean property definition (marker type).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct BoolSpecDef;

/// Date property definition.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct DateSpecDef {
    /// Date format string (using chrono format tokens).
    pub format: String,
}

/// File property definition.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct FileSpecDef {
    /// Optional directory restriction (vault-relative path).
    pub directory: Option<String>,
    /// Optional file class restriction (schema name).
    pub file_class: Option<String>,
}

/// Number property definition.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct NumberSpecDef {
    /// Optional maximum value.
    pub max: Option<f64>,
    /// Optional minimum value.
    pub min: Option<f64>,
    /// Optional step increment.
    pub step: Option<f64>,
}

/// String property definition.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct StringSpecDef {
    /// Optional enum of allowed values.
    pub enum_values: Option<Vec<String>>,
    /// Optional max length.
    pub max_length: Option<usize>,
    /// Optional min length.
    pub min_length: Option<usize>,
    /// Optional regex pattern.
    pub pattern: Option<String>,
}

// --- Validated runtime types: *Spec ---

/// Validated sum type for all supported property specifications.
#[derive(
    Debug,
    Clone,
    PartialEq,
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
    /// # use lithos_core::schema::property_spec::{PropertySpecDef, BoolSpecDef};
    /// let def = PropertySpecDef::Bool(BoolSpecDef::default());
    /// let spec = def.try_into_validated().unwrap();
    /// spec.validate(&serde_json::json!(true)).unwrap();
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
                value.as_bool().ok_or_else(|| SchemaError::InvalidType {
                    value: value.to_string(),
                    expected: "boolean".to_owned(),
                })?;
                Ok(())
            }
            Self::Date(s) => {
                let val =
                    value.as_str().ok_or_else(|| SchemaError::InvalidType {
                        value: value.to_string(),
                        expected: "string (date)".to_owned(),
                    })?;
                s.validate_str(val)
            }
            Self::File(s) => {
                let val =
                    value.as_str().ok_or_else(|| SchemaError::InvalidType {
                        value: value.to_string(),
                        expected: "string (file path)".to_owned(),
                    })?;
                s.validate_str(val)
            }
            Self::Number(s) => {
                let n =
                    value.as_f64().ok_or_else(|| SchemaError::InvalidType {
                        value: value.to_string(),
                        expected: "number".to_owned(),
                    })?;
                if !n.is_finite() {
                    return Err(SchemaError::ValidationFailed(format!(
                        "Value {n} is not finite"
                    )));
                }
                s.validate_range(n)?;
                s.validate_step(n)?;
                Ok(())
            }
            Self::String(s) => {
                let val =
                    value.as_str().ok_or_else(|| SchemaError::InvalidType {
                        value: value.to_string(),
                        expected: "string".to_owned(),
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

/// Date property validation constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
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
    pub fn try_new(format: String) -> Result<Self, SchemaError> {
        if format.is_empty() {
            return Err(SchemaError::InvalidDateFormat(
                "Format cannot be empty".to_owned(),
            ));
        }
        Ok(Self {
            format: format.into_boxed_str(),
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
}

/// File property validation constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
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
            Some(dir) => Some(VaultRelPath::try_new(dir)?),
            None => None,
        };

        if let Some(fc) = file_class.as_ref()
            && fc.is_empty()
        {
            return Err(SchemaError::InvalidFileClass(
                "File class cannot be empty".to_owned(),
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
}

/// Number property validation constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
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

        let bounds = Bounds::try_new(min, max).map_err(|e| match e {
            BoundsError::MinGreaterThanMax => SchemaError::ValidationFailed(
                "min cannot be greater than max".to_owned(),
            ),
        })?;

        let step = match step {
            Some(v) => Some(Step::try_new(v)?),
            None => None,
        };

        Ok(Self {
            bounds,
            step,
        })
    }

    /// Validates that a numeric value falls within optional min/max bounds.
    ///
    /// # Errors
    /// Returns `SchemaError::NumberOutOfRange` if validation fails.
    #[inline]
    pub fn validate_range(&self, value: f64) -> Result<(), SchemaError> {
        if !value.is_finite() {
            return Err(SchemaError::ValidationFailed(format!(
                "Value {value} is not finite"
            )));
        }
        let finite = FiniteF64::try_new(value, "value").map_err(|_err| {
            SchemaError::ValidationFailed(format!(
                "Value {value} is not finite"
            ))
        })?;

        if let Err(violation) = self.bounds.check(finite) {
            let min = self.bounds.min.map(FiniteF64::get);
            let max = self.bounds.max.map(FiniteF64::get);
            return Err(match violation {
                BoundsViolation::BelowMin {
                    ..
                }
                | BoundsViolation::AboveMax {
                    ..
                } => SchemaError::NumberOutOfRange {
                    value,
                    min,
                    max,
                },
            });
        }
        Ok(())
    }

    /// Validates that a numeric value aligns with a step increment.
    ///
    /// # Errors
    /// Returns `SchemaError::InvalidStepValue` if validation fails.
    #[inline]
    #[expect(
        clippy::float_arithmetic,
        clippy::modulo_arithmetic,
        reason = "Core numeric validation logic with epsilon comparison"
    )]
    pub fn validate_step(&self, value: f64) -> Result<(), SchemaError> {
        const EPSILON: f64 = 1e-10;

        if !value.is_finite() {
            return Err(SchemaError::ValidationFailed(format!(
                "Value {value} is not finite"
            )));
        }

        if let Some(step) = self.step {
            let base = self.bounds.min.map_or(0.0f64, FiniteF64::get);
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
}

/// String property validation constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[non_exhaustive]
pub struct StringSpec {
    enum_values: Option<Vec<Box<str>>>,
    length: Bounds<usize>,
    pattern: Option<Box<str>>,
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
        pattern: Option<String>,
        enum_values: Option<Vec<String>>,
    ) -> Result<Self, SchemaError> {
        let length =
            Bounds::try_new(min_length, max_length).map_err(|e| match e {
                BoundsError::MinGreaterThanMax => {
                    SchemaError::ValidationFailed(
                        "min_length cannot be greater than max_length"
                            .to_owned(),
                    )
                }
            })?;

        let pattern = match pattern {
            Some(p) => {
                get_cached_regex(&p)?;
                Some(p.into_boxed_str())
            }
            None => None,
        };

        let enum_values = enum_values
            .map(|vals| vals.into_iter().map(String::into_boxed_str).collect());

        Ok(Self {
            enum_values,
            length,
            pattern,
        })
    }

    #[inline]
    fn validate_str(&self, value: &str) -> Result<(), SchemaError> {
        self.validate_length(value)?;
        self.validate_enum(value)?;
        self.validate_pattern(value)?;
        Ok(())
    }

    fn validate_enum(&self, value: &str) -> Result<(), SchemaError> {
        if let Some(enums) = self.enum_values.as_ref()
            && !enums.iter().any(|s| s.as_ref() == value)
        {
            return Err(SchemaError::InvalidEnumValue {
                value: value.to_owned(),
                allowed: enums
                    .iter()
                    .map(std::string::ToString::to_string)
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
        if let Err(violation) = self.length.check(len) {
            return Err(match violation {
                BoundsViolation::BelowMin {
                    min,
                } => SchemaError::StringTooShort {
                    min,
                    actual: len,
                },
                BoundsViolation::AboveMax {
                    max,
                } => SchemaError::StringTooLong {
                    max,
                    actual: len,
                },
            });
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
}

// --- Internal helper types (type-driven invariants) ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundsError {
    MinGreaterThanMax,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BoundsViolation<T> {
    BelowMin {
        min: T,
    },
    AboveMax {
        max: T,
    },
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
struct Bounds<T> {
    min: Option<T>,
    max: Option<T>,
}

impl<T> Default for Bounds<T> {
    #[inline]
    fn default() -> Self {
        Self {
            min: None,
            max: None,
        }
    }
}

impl<T> Bounds<T>
where
    T: Copy + PartialOrd,
{
    #[inline]
    fn try_new(min: Option<T>, max: Option<T>) -> Result<Self, BoundsError> {
        if let (Some(min), Some(max)) = (min, max)
            && min > max
        {
            return Err(BoundsError::MinGreaterThanMax);
        }
        Ok(Self {
            min,
            max,
        })
    }

    #[inline]
    fn check(&self, value: T) -> Result<(), BoundsViolation<T>> {
        if let Some(min) = self.min
            && value < min
        {
            return Err(BoundsViolation::BelowMin {
                min,
            });
        }
        if let Some(max) = self.max
            && value > max
        {
            return Err(BoundsViolation::AboveMax {
                max,
            });
        }
        Ok(())
    }
}

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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
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
                "step must be positive".to_owned(),
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
    fn try_new(path: String) -> Result<Self, SchemaError> {
        validate_vault_rel_path(&path)?;
        Ok(Self(path.into_boxed_str()))
    }

    #[inline]
    fn as_str(&self) -> &str {
        &self.0
    }
}

type RegexCache = HashMap<String, Arc<regex::Regex>>;
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

    match guard.entry(pattern.to_owned()) {
        Entry::Occupied(entry) => Ok(Arc::clone(entry.get())),
        Entry::Vacant(entry) => {
            entry.insert(Arc::clone(&compiled));
            Ok(compiled)
        }
    }
}

fn validate_vault_rel_path(path: &str) -> Result<(), SchemaError> {
    if path.is_empty() {
        return Err(SchemaError::InvalidDirectoryPath(
            "Path cannot be empty".to_owned(),
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

#[cfg(test)]
mod tests {
    use super::*;

    mod string_spec {
        use rstest::rstest;

        use super::*;

        /// 3.3-UNIT-011: String Specification Validation Matrix.
        /// Priority: P1.
        #[rstest]
        #[case::enum_match(
            StringSpecDef {
                enum_values: Some(vec!["A".to_owned(), "B".to_owned()]),
                ..Default::default()
            },
            "A",
            Ok(())
        )]
        #[case::enum_mismatch(
            StringSpecDef {
                enum_values: Some(vec!["A".to_owned(), "B".to_owned()]),
                ..Default::default()
            },
            "C",
            Err(SchemaError::InvalidEnumValue {
                value: "C".to_owned(),
                allowed: vec!["A".to_owned(), "B".to_owned()]
            })
        )]
        #[case::regex_match(
            StringSpecDef { pattern: Some(r"^\d+$".to_owned()), ..Default::default() },
            "123",
            Ok(())
        )]
        #[case::regex_mismatch(
            StringSpecDef { pattern: Some(r"^\d+$".to_owned()), ..Default::default() },
            "abc",
            Err(SchemaError::ValidationFailed("Value abc does not match pattern ^\\d+$".to_owned()))
        )]
        #[case::length_match(
            StringSpecDef { min_length: Some(2), max_length: Some(5), ..Default::default() },
            "abc",
            Ok(())
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
            // GIVEN: a validated spec
            let spec_result = StringSpec::try_new(
                def.min_length,
                def.max_length,
                def.pattern,
                def.enum_values,
            );
            assert!(
                spec_result.is_ok(),
                "Expected valid StringSpecDef, got: {spec_result:?}"
            );
            let Ok(spec) = spec_result else {
                return;
            };

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

        use super::*;

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
            // GIVEN: a validated spec
            let spec_result = NumberSpec::try_new(def.min, def.max, def.step);
            assert!(
                spec_result.is_ok(),
                "Expected valid NumberSpecDef, got: {spec_result:?}"
            );
            let Ok(spec) = spec_result else {
                return;
            };

            // WHEN: validating a numeric value
            let result = (|| {
                spec.validate_range(value)?;
                spec.validate_step(value)?;
                Ok(())
            })();

            // THEN: the result matches the expectation
            assert_eq!(
                result, expected,
                "Number validation failed for value={value}: expected \
                 {expected:?}, got {result:?}"
            );
        }

        #[test]
        fn number_spec_validates_spec_definition() {
            // GIVEN: an invalid NumberSpec (min > max)
            let result = NumberSpec::try_new(Some(10.0f64), Some(5.0f64), None);
            assert!(
                matches!(result, Err(SchemaError::ValidationFailed(_))),
                "Expected ValidationFailed for min > max, got: {result:?}"
            );

            // AND: valid specs pass
            let valid =
                NumberSpec::try_new(Some(5.0f64), Some(10.0f64), Some(1.0f64));
            assert!(
                valid.is_ok(),
                "Valid NumberSpec should succeed, got error: {:?}",
                valid.err()
            );
        }

        #[test]
        fn number_spec_rejects_non_finite_values() {
            let spec_result =
                NumberSpec::try_new(Some(0.0f64), Some(10.0f64), None);
            assert!(
                spec_result.is_ok(),
                "Expected valid NumberSpec, got: {spec_result:?}"
            );
            let Ok(spec) = spec_result else {
                return;
            };

            let nan_result = spec.validate_range(f64::NAN);
            assert!(matches!(
                nan_result,
                Err(SchemaError::ValidationFailed(_))
            ));

            let inf_result = spec.validate_range(f64::INFINITY);
            assert!(matches!(
                inf_result,
                Err(SchemaError::ValidationFailed(_))
            ));
        }

        #[test]
        fn number_spec_rejects_non_finite_spec_fields() {
            let invalid_min_result =
                NumberSpec::try_new(Some(f64::NAN), Some(10.0f64), None);
            assert!(matches!(
                invalid_min_result,
                Err(SchemaError::ValidationFailed(_))
            ));

            let invalid_step_result = NumberSpec::try_new(
                Some(-10.0f64),
                Some(10.0f64),
                Some(f64::INFINITY),
            );
            assert!(matches!(
                invalid_step_result,
                Err(SchemaError::ValidationFailed(_))
            ));
        }
    }

    mod file_spec {
        use rstest::rstest;

        use super::*;

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
            // GIVEN: a FileSpec with a directory restriction
            let spec_result = FileSpec::try_new(Some(dir.to_owned()), None);
            assert!(
                spec_result.is_ok(),
                "Expected valid FileSpec, got: {spec_result:?}"
            );
            let Ok(spec) = spec_result else {
                return;
            };

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
            let spec_result =
                FileSpec::try_new(Some("notes/".to_owned()), None);
            assert!(
                spec_result.is_ok(),
                "Expected valid FileSpec, got: {spec_result:?}"
            );
            let Ok(spec) = spec_result else {
                return;
            };

            let result = spec.validate_str("notes_evil/note.md");
            assert!(matches!(
                result,
                Err(SchemaError::InvalidDirectoryPath(_))
            ));
        }

        #[test]
        fn file_spec_rejects_parent_dir_traversal() {
            let spec_result =
                FileSpec::try_new(Some("notes/".to_owned()), None);
            assert!(
                spec_result.is_ok(),
                "Expected valid FileSpec, got: {spec_result:?}"
            );
            let Ok(spec) = spec_result else {
                return;
            };

            let result = spec.validate_str("../notes/note.md");
            assert!(matches!(
                result,
                Err(SchemaError::InvalidDirectoryPath(_))
            ));
        }

        #[test]
        fn file_spec_rejects_absolute_paths() {
            let spec_result =
                FileSpec::try_new(Some("notes/".to_owned()), None);
            assert!(
                spec_result.is_ok(),
                "Expected valid FileSpec, got: {spec_result:?}"
            );
            let Ok(spec) = spec_result else {
                return;
            };

            let result = spec.validate_str("/notes/note.md");
            assert!(matches!(
                result,
                Err(SchemaError::InvalidDirectoryPath(_))
            ));
        }

        #[test]
        fn file_spec_rejects_value_equal_to_directory() {
            let spec_result =
                FileSpec::try_new(Some("notes/".to_owned()), None);
            assert!(
                spec_result.is_ok(),
                "Expected valid FileSpec, got: {spec_result:?}"
            );
            let Ok(spec) = spec_result else {
                return;
            };

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
        use super::*;

        #[test]
        fn bool_spec_validates_type() {
            // GIVEN: a BoolSpec
            let spec = BoolSpec::default();

            // THEN: it accepts booleans
            let p = PropertySpec::Bool(spec);
            let true_result = p.validate(&serde_json::Value::Bool(true));
            assert!(
                true_result.is_ok(),
                "Expected bool validation to succeed, got: {true_result:?}"
            );
            let false_result = p.validate(&serde_json::Value::Bool(false));
            assert!(
                false_result.is_ok(),
                "Expected bool validation to succeed, got: {false_result:?}"
            );
        }
    }

    mod date_spec {
        use super::*;

        #[test]
        fn date_spec_validates_iso8601() {
            // GIVEN: a DateSpec with RFC3339-like format
            let spec_result =
                DateSpec::try_new("%Y-%m-%dT%H:%M:%SZ".to_owned());
            assert!(
                spec_result.is_ok(),
                "Expected valid DateSpec, got: {spec_result:?}"
            );
            let Ok(spec) = spec_result else {
                return;
            };

            // THEN: it accepts matching strings
            let ok_result = spec.validate_str("2024-01-15T14:30:00Z");
            assert!(
                ok_result.is_ok(),
                "Expected date validation to succeed, got: {ok_result:?}"
            );

            // AND: rejects invalid dates
            let result = spec.validate_str("not-a-date");
            assert!(
                matches!(result, Err(SchemaError::InvalidDateFormat(_))),
                "Expected InvalidDateFormat error for invalid date string, \
                 got: {result:?}"
            );
        }
    }

    mod property_spec {
        use super::*;

        #[test]
        fn property_spec_dispatch_works() {
            // GIVEN: various spec variants
            let b_result = PropertySpecDef::Bool(BoolSpecDef::default())
                .try_into_validated();
            assert!(
                b_result.is_ok(),
                "Expected default BoolSpec to validate, got: {b_result:?}"
            );
            let Ok(b) = b_result else {
                return;
            };

            let s_result = PropertySpecDef::String(StringSpecDef::default())
                .try_into_validated();
            assert!(
                s_result.is_ok(),
                "Expected default StringSpec to validate, got: {s_result:?}"
            );
            let Ok(s) = s_result else {
                return;
            };

            let n_result = PropertySpecDef::Number(NumberSpecDef::default())
                .try_into_validated();
            assert!(
                n_result.is_ok(),
                "Expected default NumberSpec to validate, got: {n_result:?}"
            );
            let Ok(n) = n_result else {
                return;
            };

            // THEN: spec_type returns correct discriminant
            assert_eq!(
                b.spec_type(),
                PropertySpecType::Bool,
                "BoolSpec should return Bool type"
            );
            assert_eq!(
                s.spec_type(),
                PropertySpecType::String,
                "StringSpec should return String type"
            );
            assert_eq!(
                n.spec_type(),
                PropertySpecType::Number,
                "NumberSpec should return Number type"
            );

            // AND: validate dispatches to inner spec (tested via successful
            // bool parse)
            let result = b.validate(&serde_json::Value::Bool(true));
            assert!(
                result.is_ok(),
                "Bool validation should succeed, got error: {:?}",
                result.err()
            );
        }
    }
}
