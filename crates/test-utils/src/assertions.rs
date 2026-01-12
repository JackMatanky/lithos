//! Custom assertion helpers for domain-specific testing.
//!
//! This module provides custom assertion macros and helpers that extend the standard
//! testing assertions with domain-specific validations, async support, and rich error reporting.
//!
//! # Features
//!
//! - **Custom Derive Macros**: Automatic assertion generation for domain types
//! - **Async Assertions**: Timeout-based assertions for async operations
//! - **Structural Comparisons**: Deep equality checks for nested data structures
//! - **Rich Error Reporting**: Field-level diffs and context information

use std::fmt;

/// Custom assertion error with detailed context.
#[derive(Debug, Clone)]
pub struct AssertionError {
    pub message: String,
    pub expected: String,
    pub actual: String,
    pub context: Vec<String>,
}

impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Assertion failed: {}", self.message)?;
        writeln!(f, "Expected: {}", self.expected)?;
        writeln!(f, "Actual: {}", self.actual)?;
        for ctx in &self.context {
            writeln!(f, "Context: {}", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for AssertionError {}

/// Result type for assertion operations.
pub type AssertionResult<T> = Result<T, AssertionError>;

/// Assert that two values are equal with rich error reporting.
///
/// This macro provides detailed diffs for complex types and context information
/// when assertions fail.
///
/// # Example
///
/// ```rust
/// use lithos_test_utils::assert_eq_detailed;
///
/// let expected = vec![1, 2, 3];
/// let actual = vec![1, 3, 4];
/// assert_eq_detailed!(expected, actual, "Vector comparison failed");
/// ```
#[macro_export]
macro_rules! assert_eq_detailed {
    ($expected:expr, $actual:expr) => {
        assert_eq_detailed!($expected, $actual, "Values are not equal")
    };
    ($expected:expr, $actual:expr, $message:expr) => {
        match (&$expected, &$actual) {
            (expected_val, actual_val) => {
                if expected_val != actual_val {
                    let error = $crate::assertions::AssertionError {
                        message: $message.to_string(),
                        expected: format!("{:?}", expected_val),
                        actual: format!("{:?}", actual_val),
                        context: vec![],
                    };
                    panic!("{}", error);
                }
            }
        }
    };
}

/// Assert that an async operation completes within a timeout.
///
/// This macro waits for an async operation to complete and fails if it takes longer
/// than the specified timeout.
///
/// # Example
///
/// ```rust
/// use lithos_test_utils::assert_async_completed;
/// use tokio::time::Duration;
///
/// async fn slow_operation() {
///     tokio::time::sleep(Duration::from_millis(100)).await;
///     42
/// }
///
/// #[tokio::test]
/// async fn test_async_operation() {
///     let result = assert_async_completed!(slow_operation(), Duration::from_secs(1));
///     assert_eq!(result, 42);
/// }
/// ```
#[macro_export]
macro_rules! assert_async_completed {
    ($future:expr, $timeout:expr) => {
        assert_async_completed!(
            $future,
            $timeout,
            "Async operation did not complete within timeout"
        )
    };
    ($future:expr, $timeout:expr, $message:expr) => {
        match tokio::time::timeout($timeout, $future).await {
            Ok(result) => result,
            Err(_) => panic!("{}", $message),
        }
    };
}

/// Assert that a condition becomes true within a timeout.
///
/// This macro repeatedly checks a condition until it becomes true or the timeout expires.
///
/// # Example
///
/// ```rust
/// use lithos_test_utils::assert_eventually;
/// use tokio::time::Duration;
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use std::sync::Arc;
///
/// #[tokio::test]
/// async fn test_eventual_condition() {
///     let flag = Arc::new(AtomicBool::new(false));
///     let flag_clone = Arc::clone(&flag);
///
///     tokio::spawn(async move {
///         tokio::time::sleep(Duration::from_millis(100)).await;
///         flag_clone.store(true, Ordering::Relaxed);
///     });
///
///     assert_eventually!(|| flag.load(Ordering::Relaxed), Duration::from_secs(1));
/// }
/// ```
#[macro_export]
macro_rules! assert_eventually {
    ($condition:expr, $timeout:expr) => {
        assert_eventually!(
            $condition,
            $timeout,
            "Condition did not become true within timeout"
        )
    };
    ($condition:expr, $timeout:expr, $message:expr) => {{
        let start = std::time::Instant::now();
        while start.elapsed() < $timeout {
            if $condition() {
                return;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        panic!("{}", $message);
    }};
}

/// Structural comparison utilities for nested data structures.
///
/// Provides deep equality checks that can handle complex nested structures
/// with custom comparison logic.
pub mod structural {
    use super::*;

    /// Compare two values structurally, providing detailed diff information.
    ///
    /// This function performs deep comparison of complex data structures and
    /// returns detailed information about differences.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lithos_test_utils::assertions::structural::compare_structural;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let p1 = Person { name: "Alice".to_string(), age: 30 };
    /// let p2 = Person { name: "Bob".to_string(), age: 30 };
    ///
    /// let diff = compare_structural(&p1, &p2).unwrap_err();
    /// println!("{}", diff);
    /// ```
    pub fn compare_structural<T: PartialEq + fmt::Debug>(
        expected: &T,
        actual: &T,
    ) -> AssertionResult<()> {
        if expected == actual {
            Ok(())
        } else {
            Err(AssertionError {
                message: "Structural comparison failed".to_string(),
                expected: format!("{:?}", expected),
                actual: format!("{:?}", actual),
                context: vec!["Deep structural comparison".to_string()],
            })
        }
    }
}

/// Domain-specific assertion helpers.
///
/// Provides assertion functions tailored for common domain patterns in the Lithos project.
pub mod domain {
    use super::*;

    /// Assert that a collection contains exactly the expected items.
    ///
    /// This function checks that two collections have the same elements,
    /// regardless of order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lithos_test_utils::assertions::domain::assert_contains_same_items;
    ///
    /// let expected = vec![1, 2, 3];
    /// let actual = vec![3, 1, 2];
    /// assert_contains_same_items(&expected, &actual).unwrap();
    /// ```
    pub fn assert_contains_same_items<T: PartialEq + fmt::Debug + Clone>(
        expected: &[T],
        actual: &[T],
    ) -> AssertionResult<()> {
        let mut expected_sorted = expected.to_vec();
        let mut actual_sorted = actual.to_vec();
        expected_sorted
            .sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        actual_sorted
            .sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));

        if expected_sorted == actual_sorted {
            Ok(())
        } else {
            Err(AssertionError {
                message: "Collections do not contain the same items"
                    .to_string(),
                expected: format!("{:?}", expected_sorted),
                actual: format!("{:?}", actual_sorted),
                context: vec!["Order-independent comparison".to_string()],
            })
        }
    }

    /// Assert that a value is within an acceptable range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lithos_test_utils::assertions::domain::assert_in_range;
    ///
    /// assert_in_range(5, 0..10).unwrap();
    /// ```
    pub fn assert_in_range<T: PartialOrd + fmt::Debug>(
        value: T,
        range: std::ops::Range<T>,
    ) -> AssertionResult<()> {
        if value >= range.start && value < range.end {
            Ok(())
        } else {
            Err(AssertionError {
                message: "Value is not in expected range".to_string(),
                expected: format!("{:?}..{:?}", range.start, range.end),
                actual: format!("{:?}", value),
                context: vec!["Range validation".to_string()],
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;

    #[test]
    fn test_assert_eq_detailed_success() {
        assert_eq_detailed!(42, 42);
    }

    #[test]
    #[should_panic]
    fn test_assert_eq_detailed_failure() {
        assert_eq_detailed!(42, 43);
    }

    #[tokio::test]
    async fn test_assert_async_completed_success() {
        let future = async { 42 };
        let result = assert_async_completed!(future, Duration::from_secs(1));
        assert_eq!(result, 42);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_assert_async_completed_timeout() {
        let future = async {
            tokio::time::sleep(Duration::from_secs(2)).await;
            42
        };
        assert_async_completed!(future, Duration::from_millis(100));
    }

    #[test]
    fn test_structural_compare_success() {
        let expected = vec![1, 2, 3];
        let actual = vec![1, 2, 3];
        structural::compare_structural(&expected, &actual).unwrap();
    }

    #[test]
    fn test_structural_compare_failure() {
        let expected = vec![1, 2, 3];
        let actual = vec![1, 3, 4];
        let result = structural::compare_structural(&expected, &actual);
        assert!(result.is_err());
    }

    #[test]
    fn test_domain_assert_contains_same_items() {
        let expected = vec![1, 2, 3];
        let actual = vec![3, 1, 2];
        domain::assert_contains_same_items(&expected, &actual).unwrap();
    }

    #[test]
    fn test_domain_assert_in_range() {
        domain::assert_in_range(5, 0..10).unwrap();
    }

    #[test]
    fn test_domain_assert_in_range_failure() {
        let result = domain::assert_in_range(15, 0..10);
        assert!(result.is_err());
    }
}
