//! Generic bounds types for range validation across bounded contexts.
//!
//! Provides a reusable [`Bounds<T>`] enum for min/max/range validation that
//! works with any partially-ordered, copyable type (i64, f64, usize, etc.).

#![expect(
    missing_docs,
    reason = "rkyv generates undocumented archived struct fields"
)]
#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive archived enums"
)]

/// Generic numeric bounds for range validation.
///
/// Represents four possible bound configurations:
/// - `Unbounded`: No constraints
/// - `Min(T)`: Only a minimum value (inclusive)
/// - `Max(T)`: Only a maximum value (inclusive)
/// - `Range { min, max }`: Both minimum and maximum (inclusive)
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
#[rkyv(compare(PartialEq))]
#[non_exhaustive]
pub enum Bounds<T> {
    /// No bounds - any value is valid.
    Unbounded,
    /// Minimum value only (inclusive).
    Min(T),
    /// Maximum value only (inclusive).
    Max(T),
    /// Both minimum and maximum (inclusive).
    Range {
        /// Inclusive minimum.
        min: T,
        /// Inclusive maximum.
        max: T,
    },
}

impl<T: rkyv::Archive> std::fmt::Debug for ArchivedBounds<T>
where
    T::Archived: std::fmt::Debug,
{
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Manual Debug implementation for generic archived type"
    )]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &ArchivedBounds::Unbounded => write!(f, "Unbounded"),
            ArchivedBounds::Min(val) => {
                f.debug_tuple("Min").field(val).finish()
            }
            ArchivedBounds::Max(val) => {
                f.debug_tuple("Max").field(val).finish()
            }
            ArchivedBounds::Range {
                min,
                max,
            } => f
                .debug_struct("Range")
                .field("min", min)
                .field("max", max)
                .finish(),
        }
    }
}

impl<T: PartialOrd + Copy> Bounds<T> {
    /// Build bounds from optional min/max values.
    ///
    /// Returns:
    /// - `None` if both min and max are `None` (no bounds needed)
    /// - `Some(Ok(Bounds::Min))` if only min is `Some`
    /// - `Some(Ok(Bounds::Max))` if only max is `Some`
    /// - `Some(Ok(Bounds::Range))` if both are `Some` and min <= max
    /// - `Some(Err(Error))` if min > max
    #[inline]
    #[must_use]
    pub fn from_options(
        min: Option<T>,
        max: Option<T>,
    ) -> Option<Result<Self, BoundsError>> {
        match (min, max) {
            (None, None) => None,
            (Some(min), None) => Some(Ok(Self::Min(min))),
            (None, Some(max)) => Some(Ok(Self::Max(max))),
            (Some(min), Some(max)) => {
                if min <= max {
                    Some(Ok(Self::Range {
                        min,
                        max,
                    }))
                } else {
                    Some(Err(BoundsError::InvalidRange))
                }
            }
        }
    }

    /// Return true when the value satisfies the bounds.
    #[inline]
    #[must_use]
    pub fn validate(&self, value: T) -> bool {
        match *self {
            Self::Unbounded => true,
            Self::Min(min) => value >= min,
            Self::Max(max) => value <= max,
            Self::Range {
                min,
                max,
            } => value >= min && value <= max,
        }
    }

    /// Get the minimum bound if set.
    ///
    /// Returns `None` for `Unbounded` and `Max` variants.
    #[inline]
    #[must_use]
    pub fn min(&self) -> Option<T> {
        match *self {
            Self::Unbounded | Self::Max(_) => None,
            Self::Min(min)
            | Self::Range {
                min,
                ..
            } => Some(min),
        }
    }

    /// Get the maximum bound if set.
    ///
    /// Returns `None` for `Unbounded` and `Min` variants.
    #[inline]
    #[must_use]
    pub fn max(&self) -> Option<T> {
        match *self {
            Self::Unbounded | Self::Min(_) => None,
            Self::Max(max)
            | Self::Range {
                max,
                ..
            } => Some(max),
        }
    }
}

/// Errors that can occur when constructing bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[expect(
    clippy::module_name_repetitions,
    reason = "BoundsError follows the project naming convention (TypeError \
              pattern)"
)]
#[non_exhaustive]
pub enum BoundsError {
    /// Minimum value is greater than maximum.
    InvalidRange,
}

impl std::fmt::Display for BoundsError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BoundsError::InvalidRange => write!(f, "min must be <= max"),
        }
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Default Error trait methods are sufficient"
)]
impl std::error::Error for BoundsError {}

#[cfg(test)]
mod tests {
    use super::*;

    mod bounds_creation {
        use super::*;

        #[test]
        fn from_options_none_none_returns_none() {
            assert!(Bounds::<i64>::from_options(None, None).is_none());
        }

        #[test]
        fn from_options_min_only_returns_min() {
            let result = Bounds::from_options(Some(10i64), None);
            assert!(matches!(result, Some(Ok(Bounds::Min(10)))));
        }

        #[test]
        fn from_options_max_only_returns_max() {
            let result = Bounds::from_options(None, Some(100i64));
            assert!(matches!(result, Some(Ok(Bounds::Max(100)))));
        }

        #[test]
        fn from_options_valid_range_returns_range() {
            let result = Bounds::from_options(Some(0i64), Some(100i64));
            assert!(matches!(
                result,
                Some(Ok(Bounds::Range {
                    min: 0,
                    max: 100
                }))
            ));
        }

        #[test]
        fn from_options_invalid_range_returns_error() {
            let result = Bounds::from_options(Some(100i64), Some(0i64));
            assert!(matches!(result, Some(Err(BoundsError::InvalidRange))));
        }
    }

    mod bounds_validation {
        use super::*;

        #[test]
        fn unbounded_validates_all_values() {
            let bounds = Bounds::<i64>::Unbounded;
            assert!(bounds.validate(i64::MIN));
            assert!(bounds.validate(0));
            assert!(bounds.validate(i64::MAX));
        }

        #[test]
        fn min_bounds_validate_correctly() {
            let bounds = Bounds::Min(10i64);
            assert!(!bounds.validate(9));
            assert!(bounds.validate(10)); // inclusive
            assert!(bounds.validate(11));
        }

        #[test]
        fn max_bounds_validate_correctly() {
            let bounds = Bounds::Max(100i64);
            assert!(bounds.validate(99));
            assert!(bounds.validate(100)); // inclusive
            assert!(!bounds.validate(101));
        }

        #[test]
        fn range_bounds_validate_correctly() {
            let bounds = Bounds::Range {
                min: 0i64,
                max: 100,
            };
            assert!(!bounds.validate(-1));
            assert!(bounds.validate(0)); // inclusive min
            assert!(bounds.validate(50));
            assert!(bounds.validate(100)); // inclusive max
            assert!(!bounds.validate(101));
        }

        #[test]
        fn float_bounds_work() {
            let bounds = Bounds::Range {
                min: 0.0f64,
                max: 1.0f64,
            };
            assert!(!bounds.validate(-0.1f64));
            assert!(bounds.validate(0.0f64));
            assert!(bounds.validate(0.5f64));
            assert!(bounds.validate(1.0f64));
            assert!(!bounds.validate(1.1f64));
        }
    }

    mod bounds_accessors {
        use super::*;

        #[test]
        fn min_accessor_works() {
            assert_eq!(Bounds::<i64>::Unbounded.min(), None);
            assert_eq!(Bounds::Max(100i64).min(), None);
            assert_eq!(Bounds::Min(10i64).min(), Some(10));
            assert_eq!(
                Bounds::Range {
                    min: 5i64,
                    max: 10
                }
                .min(),
                Some(5)
            );
        }

        #[test]
        fn max_accessor_works() {
            assert_eq!(Bounds::<i64>::Unbounded.max(), None);
            assert_eq!(Bounds::Min(10i64).max(), None);
            assert_eq!(Bounds::Max(100i64).max(), Some(100));
            assert_eq!(
                Bounds::Range {
                    min: 5i64,
                    max: 10
                }
                .max(),
                Some(10)
            );
        }
    }
}
