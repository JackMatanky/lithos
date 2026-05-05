//! Common prelude types and utilities.

/// Generic wrapper tuple struct for newtype pattern.
///
/// Mostly used for external type to type `From`/`TryFrom` conversions.
pub struct W<T>(pub T);
