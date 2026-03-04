//! Retry logic for transient database errors.
//!
//! Provides exponential backoff retry strategy for operations that might
//! fail due to temporary conditions like database locks or I/O errors.

use std::time::Duration;

use super::DbError;

/// Configuration for retry behavior.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
    /// Initial delay before first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Multiplier for exponential backoff.
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    #[inline]
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry a database operation with exponential backoff.
///
/// Automatically retries operations that fail with transient errors
/// (database locked, temporary I/O errors, etc.) using exponential backoff.
///
/// # Errors
///
/// Returns the last error if all retry attempts fail, or immediately
/// returns non-transient errors without retrying.
///
/// # Examples
///
/// ```
/// use lithos_core::db::{
///     DbError,
///     retry::{RetryConfig, retry_on_transient},
/// };
///
/// fn flaky_operation() -> Result<String, DbError> {
///     // Simulated operation that might fail transiently
///     Ok("success".to_string())
/// }
///
/// # fn main() -> Result<(), DbError> {
/// let result =
///     retry_on_transient(RetryConfig::default(), || flaky_operation())?;
/// assert_eq!(result, "success");
/// # Ok(())
/// # }
/// ```
#[expect(
    clippy::disallowed_methods,
    reason = "std::thread::sleep is appropriate for sync retry logic"
)]
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "Retry function is complex and not a hotpath candidate for \
              inlining"
)]
pub fn retry_on_transient<F, T>(
    config: RetryConfig,
    mut operation: F,
) -> Result<T, DbError>
where
    F: FnMut() -> Result<T, DbError>,
{
    let mut attempt = 0u32;
    let mut delay = config.initial_delay;

    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Don't retry non-transient errors
                if !e.is_transient() {
                    return Err(e);
                }

                attempt = attempt.saturating_add(1);

                // Exhausted all retries
                if attempt >= config.max_attempts {
                    tracing::warn!(
                        error = %e,
                        attempts = attempt,
                        "Database operation failed after all retries"
                    );
                    return Err(e);
                }

                // Log retry attempt
                tracing::debug!(
                    error = %e,
                    attempt = attempt,
                    delay_ms = delay.as_millis(),
                    "Retrying transient database error"
                );

                // Sleep before retry
                std::thread::sleep(delay);

                // Exponential backoff with cap
                #[expect(
                    clippy::float_arithmetic,
                    reason = "Floating-point multiplication for backoff is \
                              safe and appropriate"
                )]
                {
                    delay = Duration::from_secs_f64(
                        (delay.as_secs_f64() * config.backoff_multiplier)
                            .min(config.max_delay.as_secs_f64()),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_succeeds_on_first_attempt() {
        let result = retry_on_transient(RetryConfig::default(), || {
            Ok::<_, DbError>("success")
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn retry_does_not_retry_non_transient_errors() {
        let mut attempts = 0u32;

        let result: Result<(), DbError> =
            retry_on_transient(RetryConfig::default(), || {
                attempts += 1;
                Err(DbError::Corruption("permanent error".into()))
            });

        assert!(result.is_err());
        assert_eq!(attempts, 1, "Should not retry non-transient errors");
    }

    #[test]
    fn retry_retries_transient_errors() {
        let mut attempts = 0u32;

        let result = retry_on_transient(
            RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(1),
                ..Default::default()
            },
            || {
                attempts += 1;
                if attempts < 3 {
                    Err(DbError::Database("database is locked".into()))
                } else {
                    Ok("success")
                }
            },
        );

        assert!(result.is_ok());
        assert_eq!(attempts, 3, "Should retry until success");
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn retry_fails_after_max_attempts() {
        let mut attempts = 0u32;

        let result: Result<(), DbError> = retry_on_transient(
            RetryConfig {
                max_attempts: 2,
                initial_delay: Duration::from_millis(1),
                ..Default::default()
            },
            || {
                attempts += 1;
                Err(DbError::Database("database is locked".into()))
            },
        );

        assert!(result.is_err());
        assert_eq!(attempts, 2, "Should attempt exactly max_attempts times");
    }
}
