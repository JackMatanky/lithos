//! Number property validation constraints.

use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    bounds::{Bounds, BoundsError},
    schema::{error::SchemaError, raw::property::RawPropertyNumber},
};

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
#[derive(Debug, Clone, PartialEq, Hash, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
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
            Some(v) => {
                Some(FiniteF64::try_new(v, "min", NonFiniteKind::Constraint)?)
            }
            None => None,
        };
        let max = match max {
            Some(v) => {
                Some(FiniteF64::try_new(v, "max", NonFiniteKind::Constraint)?)
            }
            None => None,
        };

        let bounds = match Bounds::from_options(min, max) {
            None => Bounds::Unbounded,
            Some(Ok(bounds)) => bounds,
            Some(Err(BoundsError::InvalidRange)) => {
                let min = min.map(FiniteF64::get).unwrap_or_default();
                let max = max.map(FiniteF64::get).unwrap_or_default();
                return Err(SchemaError::PropertySpec(
                    crate::schema::error::PropertySpecError::InvalidRange {
                        min,
                        max,
                    },
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
        let finite = FiniteF64::try_new(value, "value", NonFiniteKind::Value)?;

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
            return Err(SchemaError::PropertyValue(
                crate::schema::error::PropertyValueError::NumberOutOfRange {
                    value: finite.get(),
                    min,
                    max,
                },
            ));
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
                return Err(SchemaError::PropertyValue(
                    crate::schema::error::PropertyValueError::InvalidStepValue {
                        value,
                        step,
                    },
                ));
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
    /// use lithos_core::schema::{
    ///     property_spec::NumberSpec, raw::number::RawNumberSpec,
    /// };
    ///
    /// let base = NumberSpec::try_new(None, None, None)?;
    /// let overrides = RawNumberSpec::default();
    /// let _updated = base.apply_overrides(&overrides)?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn apply_overrides(
        self,
        overrides: &crate::schema::raw::number::RawNumberSpec,
    ) -> Result<Self, SchemaError> {
        let min = overrides.min.or(self.bounds.min().map(FiniteF64::get));
        let max = overrides.max.or(self.bounds.max().map(FiniteF64::get));
        let step = overrides.step.or(self.step.map(Step::get));
        Self::try_new(min, max, step)
    }
}

impl ArchivedNumberSpec {
    /// Validates a numeric value against range and step constraints directly
    /// from the database without full deserialization.
    ///
    /// This method deserializes only the `NumberSpec` (a small struct) rather
    /// than the entire property hierarchy, providing significant performance
    /// benefits for validation-heavy workloads.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validate(&self, value: f64) -> Result<(), SchemaError> {
        // Deserialize the spec (small overhead) to reuse existing validation
        // logic
        let spec: NumberSpec =
            rkyv::deserialize::<NumberSpec, rkyv::rancor::Error>(self)
                .map_err(|e| {
                    SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::Deserialization {
                    spec: "NumberSpec",
                    reason: e.to_string().into(),
                },
            )
                })?;
        spec.validate_value(value)
    }
}

impl TryFrom<crate::schema::raw::number::RawNumberSpec> for NumberSpec {
    type Error = SchemaError;

    #[inline]
    fn try_from(
        raw: crate::schema::raw::number::RawNumberSpec,
    ) -> Result<Self, Self::Error> {
        Self::try_new(raw.min, raw.max, raw.step)
    }
}

impl TryFrom<RawPropertyNumber> for NumberSpec {
    type Error = SchemaError;

    #[inline]
    fn try_from(raw: RawPropertyNumber) -> Result<Self, Self::Error> {
        let raw_spec = crate::schema::raw::number::RawNumberSpec {
            min: raw.min,
            max: raw.max,
            step: raw.step,
        };
        raw_spec.try_into()
    }
}

// --- Internal helper types (type-driven invariants) ---

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Default,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
struct FiniteF64(f64);

impl FiniteF64 {
    #[inline]
    fn try_new(
        value: f64,
        ctx: &'static str,
        kind: NonFiniteKind,
    ) -> Result<Self, SchemaError> {
        if !value.is_finite() {
            return Err(kind.into_error(value, ctx));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NonFiniteKind {
    Constraint,
    Value,
}

impl NonFiniteKind {
    fn into_error(self, value: f64, ctx: &'static str) -> SchemaError {
        match self {
            Self::Constraint => SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::NonFinite {
                    value,
                    context: ctx.into(),
                },
            ),
            Self::Value => SchemaError::PropertyValue(
                crate::schema::error::PropertyValueError::NonFinite {
                    value,
                    context: ctx.into(),
                },
            ),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
struct Step(FiniteF64);

impl Step {
    #[inline]
    fn try_new(value: f64) -> Result<Self, SchemaError> {
        let finite =
            FiniteF64::try_new(value, "step", NonFiniteKind::Constraint)?;
        if finite.get() <= 0.0f64 {
            return Err(SchemaError::PropertyValue(
                crate::schema::error::PropertyValueError::InvalidStepValue {
                    value: finite.get(),
                    step: finite.get(),
                },
            ));
        }
        Ok(Self(finite))
    }

    #[inline]
    const fn get(self) -> f64 {
        self.0.get()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::schema::raw::number::RawNumberSpec;

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
        Err(SchemaError::PropertyValue(
            crate::schema::error::PropertyValueError::NumberOutOfRange {
                value: -1.0f64,
                min: Some(0.0f64),
                max: Some(10.0f64)
            }
        ))
    )]
    #[case::above_max(
        RawNumberSpec { min: Some(0.0f64), max: Some(10.0f64), step: None },
        11.0f64,
        Err(SchemaError::PropertyValue(
            crate::schema::error::PropertyValueError::NumberOutOfRange {
                value: 11.0f64,
                min: Some(0.0f64),
                max: Some(10.0f64)
            }
        ))
    )]
    #[case::valid_step(
        RawNumberSpec { min: Some(0.0f64), max: None, step: Some(0.5f64) },
        5.5f64,
        Ok(())
    )]
    #[case::invalid_step(
        RawNumberSpec { min: Some(0.0f64), max: None, step: Some(0.5f64) },
        5.2f64,
        Err(SchemaError::PropertyValue(
            crate::schema::error::PropertyValueError::InvalidStepValue {
                value: 5.2f64,
                step: 0.5f64
            }
        ))
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
            matches!(
                result,
                Err(SchemaError::PropertySpec(
                    crate::schema::error::PropertySpecError::InvalidRange { .. }
                ))
            ),
            "Expected InvalidRange for min > max, got: {result:?}"
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
            matches!(
                result,
                Err(SchemaError::PropertyValue(
                    crate::schema::error::PropertyValueError::NonFinite { .. }
                ))
            ),
            "Expected NonFinite for NaN, got: {result:?}"
        );
    }

    #[test]

    fn number_spec_rejects_infinite_value() {
        let spec = NumberSpec::try_new(Some(0.0f64), Some(10.0f64), None)
            .expect("Expected valid NumberSpec");
        let result = spec.validate_value(f64::INFINITY);
        assert!(
            matches!(
                result,
                Err(SchemaError::PropertyValue(
                    crate::schema::error::PropertyValueError::NonFinite { .. }
                ))
            ),
            "Expected NonFinite for infinity, got: {result:?}"
        );
    }

    #[test]
    fn number_spec_rejects_non_finite_min_bound() {
        let result = NumberSpec::try_new(Some(f64::NAN), Some(10.0f64), None);
        assert!(
            matches!(
                result,
                Err(SchemaError::PropertySpec(
                    crate::schema::error::PropertySpecError::NonFinite { .. }
                ))
            ),
            "Expected NonFinite for non-finite min, got: {result:?}"
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
            matches!(
                result,
                Err(SchemaError::PropertySpec(
                    crate::schema::error::PropertySpecError::NonFinite { .. }
                ))
            ),
            "Expected NonFinite for non-finite step, got: {result:?}"
        );
    }
}
