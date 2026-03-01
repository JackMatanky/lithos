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
    collections::HashMap,
    path::{Component, Path},
    sync::{Arc, OnceLock, RwLock},
};

use super::{error::SchemaError, formats::StringFormat};
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

/// Validated sum type for all supported property specifications.
///
/// # Examples
/// ```
/// use lithos_core::schema::property_spec::{BoolSpec, PropertySpec};
///
/// let spec = PropertySpec::Bool(BoolSpec::default());
/// match spec {
///     PropertySpec::Bool(_) => {}
///     _ => {}
/// }
/// ```
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
    /// # use lithos_core::schema::raw::{RawPropertySpec, RawBoolSpec};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let def = RawPropertySpec::Bool(RawBoolSpec);
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
                    return Err(Self::invalid_type(value, "boolean"));
                }
                Ok(())
            }
            Self::Date(s) => {
                let val = Self::expect_str(value, "string (date)")?;
                s.validate_str(val)
            }
            Self::File(s) => {
                let val = Self::expect_str(value, "string (file path)")?;
                s.validate_str(val)
            }
            Self::Number(s) => {
                let n = Self::expect_f64(value, "number")?;
                s.validate_value(n)
            }
            Self::String(s) => {
                let val = Self::expect_str(value, "string")?;
                s.validate_str(val)
            }
        }
    }

    #[inline]
    fn invalid_type(
        value: &serde_json::Value,
        expected: &'static str,
    ) -> SchemaError {
        SchemaError::InvalidType {
            value: value.to_string(),
            expected: expected.into(),
        }
    }

    #[inline]
    fn expect_str<'value>(
        value: &'value serde_json::Value,
        expected: &'static str,
    ) -> Result<&'value str, SchemaError> {
        value.as_str().ok_or_else(|| Self::invalid_type(value, expected))
    }

    #[inline]
    fn expect_f64(
        value: &serde_json::Value,
        expected: &'static str,
    ) -> Result<f64, SchemaError> {
        value.as_f64().ok_or_else(|| Self::invalid_type(value, expected))
    }
}

/// Boolean property constraints (marker type).
///
/// This type intentionally has no methods because it carries no data.
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
    /// Validates that the format string is a valid strftime pattern by probing
    /// it with a known datetime.
    ///
    /// # Errors
    /// Returns `SchemaError::InvalidDateFormat` if the format is empty or not
    /// a valid strftime pattern.
    ///
    /// # Panics
    ///
    /// This function will not panic. The `expect` calls are infallible because
    /// the probe datetime is statically known to be valid.
    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Probe datetime 2000-01-01 00:00:00 is statically valid"
    )]
    pub fn try_new(format: &str) -> Result<Self, SchemaError> {
        if format.is_empty() {
            return Err(SchemaError::InvalidDateFormat(
                "Format cannot be empty".into(),
            ));
        }

        // Probe: attempt to format a known datetime with this format string
        // Use NaiveDateTime to support both date and datetime format strings
        let probe = chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
            .expect("static date should be valid")
            .and_hms_opt(0, 0, 0)
            .expect("static time should be valid");
        let result = probe.format(format).to_string();

        // Verify the format string can parse its own output
        if chrono::NaiveDate::parse_from_str(&result, format).is_err()
            && chrono::NaiveDateTime::parse_from_str(&result, format).is_err()
        {
            return Err(SchemaError::InvalidDateFormat(format!(
                "Format string '{format}' is not a valid strftime pattern"
            )));
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

    /// Apply overrides from a raw date spec.
    ///
    /// If the override format is `None`, the base format is preserved.
    ///
    /// # Errors
    /// Returns `SchemaError::InvalidDateFormat` if the override format is
    /// invalid.
    #[inline]
    pub fn apply_overrides(
        self,
        overrides: &crate::schema::raw::RawDateSpec,
    ) -> Result<Self, SchemaError> {
        if let Some(format) = overrides.format.as_ref() {
            Self::try_new(format.as_ref())
        } else {
            Ok(self)
        }
    }
}

/// File property validation constraints.
///
/// # Examples
/// ```
/// use lithos_core::schema::property_spec::FileSpec;
///
/// let spec = FileSpec::try_new(None, None)?;
/// let _ = spec;
/// # Ok::<_, lithos_core::schema::error::SchemaError>(())
/// ```
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
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::property_spec::FileSpec;
    ///
    /// let spec = FileSpec::try_new(Some("attachments"), None)?;
    /// let _ = spec;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn try_new(
        directory: Option<&str>,
        file_class: Option<&str>,
    ) -> Result<Self, SchemaError> {
        let directory = match directory {
            Some(dir) => Some(VaultRelPath::try_new(dir)?),
            None => None,
        };

        if let Some(fc) = file_class
            && fc.is_empty()
        {
            return Err(SchemaError::InvalidFileClass(
                "File class cannot be empty".into(),
            ));
        }

        Ok(Self {
            directory,
            file_class: file_class.map(Into::into),
        })
    }

    #[inline]
    fn validate_str(&self, value: &str) -> Result<(), SchemaError> {
        VaultRelPath::validate_path(value)?;

        if let Some(dir) = self.directory.as_ref() {
            let value_path = Path::new(value);
            let dir_path = Path::new(dir.as_str());

            // File must be INSIDE directory, not AT directory level
            if !value_path.starts_with(dir_path) || value_path == dir_path {
                return Err(SchemaError::InvalidDirectoryPath(format!(
                    "File {value} must be inside (not at) directory {}",
                    dir.as_str()
                )));
            }
        }

        Ok(())
    }

    /// Apply overrides from a raw file spec.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{property_spec::FileSpec, raw::RawFileSpec};
    ///
    /// let base = FileSpec::try_new(None, None)?;
    /// let overrides = RawFileSpec::default();
    /// let _updated = base.apply_overrides(&overrides)?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    ///
    /// Fields that are `None` in the overrides preserve the base values.
    ///
    /// # Errors
    /// Returns `SchemaError` if override values are invalid.
    #[inline]
    pub fn apply_overrides(
        self,
        overrides: &crate::schema::raw::RawFileSpec,
    ) -> Result<Self, SchemaError> {
        let directory = overrides
            .directory
            .as_deref()
            .or_else(|| self.directory.as_ref().map(VaultRelPath::as_str));
        let file_class =
            overrides.file_class.as_deref().or(self.file_class.as_deref());
        Self::try_new(directory, file_class)
    }
}

/// Number property validation constraints.
///
/// # Examples
/// ```
/// use lithos_core::schema::property_spec::NumberSpec;
///
/// let spec = NumberSpec::try_new(Some(0.0), Some(10.0), None)?;
/// spec.validate_value(5.0)?;
/// # Ok::<_, lithos_core::schema::error::SchemaError>(())
/// ```
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
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::property_spec::NumberSpec;
    ///
    /// let _spec = NumberSpec::try_new(Some(0.0), Some(10.0), Some(1.0))?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
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
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::property_spec::NumberSpec;
    ///
    /// let spec = NumberSpec::try_new(Some(0.0), Some(10.0), None)?;
    /// spec.validate_value(5.0)?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
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
    ///
    /// Uses relative epsilon (scaled to step size) for robust floating-point
    /// comparison across magnitudes.
    #[inline]
    #[expect(
        clippy::float_arithmetic,
        clippy::modulo_arithmetic,
        reason = "Core numeric validation logic with epsilon comparison"
    )]
    fn validate_step(&self, finite: FiniteF64) -> Result<(), SchemaError> {
        let value = finite.get();

        if let Some(step) = self.step {
            let base = self.bounds.min().map_or(0.0f64, FiniteF64::get);
            let offset = (value - base).abs();
            let step = step.get();
            let remainder = offset % step;

            // Use relative epsilon scaled to step size for robust comparison
            // across different magnitudes (handles both large and tiny steps)
            let epsilon = step.abs() * 1e-10f64;

            if remainder > epsilon && (step - remainder) > epsilon {
                return Err(SchemaError::InvalidStepValue {
                    value,
                    step,
                });
            }
        }
        Ok(())
    }

    /// Apply overrides from a raw number spec.
    ///
    /// Fields that are `None` in the overrides preserve the base values.
    ///
    /// # Errors
    /// Returns `SchemaError` if override values are invalid.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{property_spec::NumberSpec, raw::RawNumberSpec};
    ///
    /// let base = NumberSpec::try_new(None, None, None)?;
    /// let overrides = RawNumberSpec::default();
    /// let _updated = base.apply_overrides(&overrides)?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn apply_overrides(
        self,
        overrides: &crate::schema::raw::RawNumberSpec,
    ) -> Result<Self, SchemaError> {
        let min = overrides.min.or(self.bounds.min().map(FiniteF64::get));
        let max = overrides.max.or(self.bounds.max().map(FiniteF64::get));
        let step = overrides.step.or(self.step.map(Step::get));
        Self::try_new(min, max, step)
    }
}

/// String property validation constraints.
///
/// # Invariants
/// - `format` and `pattern` are mutually exclusive (only one can be set).
/// - If `pattern` is set, it must be a valid regex.
///
/// # Examples
/// ```
/// use lithos_core::schema::property_spec::StringSpec;
///
/// let spec = StringSpec::try_new(None, None, None)?;
/// let _ = spec;
/// # Ok::<_, lithos_core::schema::error::SchemaError>(())
/// ```
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
#[non_exhaustive]
pub struct StringSpec {
    options: Option<Vec<OptionEntry>>,
    pattern: Option<Box<str>>,
    format: Option<StringFormat>,
}

impl Default for StringSpec {
    #[inline]
    fn default() -> Self {
        Self {
            options: None,
            pattern: None,
            format: None,
        }
    }
}

impl StringSpec {
    /// Create a validated `StringSpec`.
    ///
    /// # Errors
    /// Returns `SchemaError` if:
    /// - `pattern` is present but not a valid regex.
    /// - Both `pattern` and `format` are specified (mutually exclusive).
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::property_spec::StringSpec;
    ///
    /// let _spec = StringSpec::try_new(None, None, None)?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn try_new(
        pattern: Option<Box<str>>,
        format: Option<StringFormat>,
        options: Option<Vec<OptionEntry>>,
    ) -> Result<Self, SchemaError> {
        // Validate mutual exclusivity
        if pattern.is_some() && format.is_some() {
            return Err(SchemaError::ValidationFailed(
                "pattern and format are mutually exclusive".into(),
            ));
        }

        // Validate pattern if present (compile to check validity, then discard)
        if let Some(p) = pattern.as_ref() {
            regex::Regex::new(p).map_err(|e| {
                SchemaError::InvalidRegex(format!("Invalid pattern {p}: {e}"))
            })?;
        }

        Ok(Self {
            options,
            pattern,
            format,
        })
    }

    #[inline]
    fn validate_str(&self, value: &str) -> Result<(), SchemaError> {
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

    fn validate_pattern(&self, value: &str) -> Result<(), SchemaError> {
        // Use format regex if specified (pre-compiled static)
        if let Some(format) = self.format {
            let re = format.regex();
            if !re.is_match(value) {
                return Err(SchemaError::ValidationFailed(format!(
                    "Value {value} does not match format '{format}' (pattern: \
                     {})",
                    format.pattern()
                )));
            }
            return Ok(());
        }

        // Otherwise use custom pattern if specified (cached compilation)
        if let Some(pattern) = self.pattern.as_ref() {
            let re = get_or_compile_pattern(pattern);
            if !re.is_match(value) {
                return Err(SchemaError::ValidationFailed(format!(
                    "Value {value} does not match pattern {pattern}"
                )));
            }
        }
        Ok(())
    }

    /// Apply overrides from a raw string spec.
    ///
    /// Fields that are `None` in the overrides preserve the base values.
    ///
    /// # Errors
    /// Returns `SchemaError` if override values are invalid.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{property_spec::StringSpec, raw::RawStringSpec};
    ///
    /// let base = StringSpec::try_new(None, None, None)?;
    /// let overrides = RawStringSpec::default();
    /// let _updated = base.apply_overrides(&overrides)?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn apply_overrides(
        self,
        overrides: &crate::schema::raw::RawStringSpec,
    ) -> Result<Self, SchemaError> {
        let pattern = overrides.pattern.clone().or(self.pattern);
        let format = overrides.format.or(self.format);
        let options = overrides
            .options
            .as_ref()
            .map(|o| o.clone().into_entries())
            .or(self.options);
        Self::try_new(pattern, format, options)
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
        Self::validate_path(path)?;
        Ok(Self(path.into()))
    }

    #[inline]
    fn as_str(&self) -> &str {
        &self.0
    }

    #[inline]
    fn validate_path(path: &str) -> Result<(), SchemaError> {
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
}

impl TryFrom<Box<str>> for VaultRelPath {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        Self::validate_path(&value)?;
        Ok(Self(value))
    }
}

/// Cache for user-defined custom regex patterns.
///
/// Built-in formats use static `OnceLock` per format. Custom patterns use this
/// shared cache to avoid recompiling on every validation.
///
/// Design: Simple unbounded cache since:
/// 1. Patterns are validated at schema load time (guaranteed valid)
/// 2. Number of unique patterns is bounded by number of properties (~100s)
/// 3. Cache is per-process, shared across all validations
type CustomPatternCache = HashMap<Box<str>, Arc<regex::Regex>>;

static CUSTOM_PATTERN_CACHE: OnceLock<RwLock<CustomPatternCache>> =
    OnceLock::new();

/// Get or compile a custom regex pattern.
///
/// Uses a simple cache to avoid recompiling patterns on every validation.
/// Patterns are guaranteed valid (validated at construction time).
#[expect(
    clippy::expect_used,
    reason = "Pattern validated at StringSpec construction, expect documents \
              invariant"
)]
fn get_or_compile_pattern(pattern: &str) -> Arc<regex::Regex> {
    let cache =
        CUSTOM_PATTERN_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

    // Fast path: read lock
    {
        let guard =
            cache.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(re) = guard.get(pattern) {
            return Arc::clone(re);
        }
    }

    // Slow path: compile and cache
    // Pattern is guaranteed valid (validated in try_new), so expect is safe
    let compiled =
        Arc::new(regex::Regex::new(pattern).expect(
            "Custom pattern should be valid (validated at construction)",
        ));

    let mut guard =
        cache.write().unwrap_or_else(std::sync::PoisonError::into_inner);
    // Check again in case another thread inserted while we compiled
    if let Some(re) = guard.get(pattern) {
        return Arc::clone(re);
    }
    guard.insert(pattern.into(), Arc::clone(&compiled));
    compiled
}

#[cfg(test)]
mod tests {
    mod string_spec {
        use rstest::rstest;

        use crate::schema::{
            error::SchemaError,
            property_spec::StringSpec,
            raw::{RawOptions, RawStringSpec},
        };

        /// 3.3-UNIT-011: String Specification Validation Matrix.
        /// Priority: P1.
        #[rstest]
        #[case::options_match(
            RawStringSpec {
                options: Some(RawOptions::List(vec!["A".into(), "B".into()])),
                ..Default::default()
            },
            "A",
            Ok(())
        )]
        #[case::options_mismatch(
            RawStringSpec {
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
            RawStringSpec { pattern: Some(r"^\d+$".into()), ..Default::default() },
            "123",
            Ok(())
        )]
        #[case::regex_mismatch(
            RawStringSpec { pattern: Some(r"^\d+$".into()), ..Default::default() },
            "abc",
            Err(SchemaError::ValidationFailed("Value abc does not match pattern ^\\d+$".to_owned()))
        )]
        fn string_spec_validation_matrix(
            #[case] def: RawStringSpec,
            #[case] value: &str,
            #[case] expected: Result<(), SchemaError>,
        ) {
            fn validated_spec(def: RawStringSpec) -> StringSpec {
                StringSpec::try_new(
                    def.pattern,
                    def.format,
                    def.options.map(RawOptions::into_entries),
                )
                .expect("Expected valid RawStringSpec")
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
            error::SchemaError, property_spec::NumberSpec, raw::RawNumberSpec,
        };

        fn validated_spec(def: &RawNumberSpec) -> NumberSpec {
            NumberSpec::try_new(def.min, def.max, def.step)
                .expect("Expected valid RawNumberSpec")
        }

        /// 3.3-UNIT-012: Number Specification Validation Matrix.
        /// Priority: P1.
        #[rstest]
        #[case::in_range(
            RawNumberSpec { min: Some(0.0f64), max: Some(10.0f64), step: None },
            5.0f64,
            Ok(())
        )]
        #[case::at_min(
            RawNumberSpec { min: Some(0.0f64), max: Some(10.0f64), step: None },
            0.0f64,
            Ok(())
        )]
        #[case::at_max(
            RawNumberSpec { min: Some(0.0f64), max: Some(10.0f64), step: None },
            10.0f64,
            Ok(())
        )]
        #[case::below_min(
            RawNumberSpec { min: Some(0.0f64), max: Some(10.0f64), step: None },
            -1.0f64,
            Err(SchemaError::NumberOutOfRange {
                value: -1.0f64,
                min: Some(0.0f64),
                max: Some(10.0f64)
            })
        )]
        #[case::above_max(
            RawNumberSpec { min: Some(0.0f64), max: Some(10.0f64), step: None },
            11.0f64,
            Err(SchemaError::NumberOutOfRange {
                value: 11.0f64,
                min: Some(0.0f64),
                max: Some(10.0f64)
            })
        )]
        #[case::valid_step(
            RawNumberSpec { min: Some(0.0f64), max: None, step: Some(0.5f64) },
            5.5f64,
            Ok(())
        )]
        #[case::invalid_step(
            RawNumberSpec { min: Some(0.0f64), max: None, step: Some(0.5f64) },
            5.2f64,
            Err(SchemaError::InvalidStepValue { value: 5.2f64, step: 0.5f64 })
        )]
        fn number_spec_validation_matrix(
            #[case] def: RawNumberSpec,
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
            FileSpec::try_new(Some(dir), None).expect("Expected valid FileSpec")
        }

        /// 3.3-UNIT-013: File Specification Validation Matrix.
        /// Priority: P1.
        #[rstest]
        #[case::in_dir("notes/my_note.md", "notes/", Ok(()))]
        #[case::out_dir(
            "other/note.md",
            "notes/",
            Err(SchemaError::InvalidDirectoryPath(
                "File other/note.md must be inside (not at) directory notes/".to_owned(),
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
        fn file_spec_rejects_directory_path_without_trailing_slash() {
            // GIVEN: FileSpec with directory "assets"
            let spec = validated_spec_with_dir("assets");

            // WHEN: validating path exactly equal to directory (without slash)
            let result = spec.validate_str("assets");

            // THEN: it should reject (file must be INSIDE directory, not AT
            // directory level)
            assert!(
                matches!(result, Err(SchemaError::InvalidDirectoryPath(_))),
                "Expected InvalidDirectoryPath error for exact directory match"
            );
        }

        #[test]
        fn file_spec_validates_file_class_format() {
            // GIVEN: a valid file_class spec
            let result = FileSpec::try_new(None, Some("any-schema-name"));
            assert!(
                result.is_ok(),
                "Expected valid file_class spec, got: {result:?}"
            );
        }

        #[test]
        fn file_spec_rejects_empty_file_class() {
            // GIVEN: an empty file_class spec
            let result = FileSpec::try_new(None, Some(""));

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
        use crate::schema::raw::{RawBoolSpec, RawPropertySpec};

        #[test]
        fn validate_dispatches_to_bool_spec() {
            let spec = RawPropertySpec::Bool(RawBoolSpec)
                .try_into_validated()
                .expect("Expected default BoolSpec to validate");
            let result = spec.validate(&serde_json::Value::Bool(true));
            assert!(
                result.is_ok(),
                "Bool validation should succeed, got error: {:?}",
                result.err()
            );
        }
    }
}
