//! Event-driven testing utilities for Lithos.
//!
//! This module provides reusable patterns for testing domain events and
//! event bus interactions based on ADR 0008.

use std::{error::Error, fmt};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

/// A recorded event with sequence metadata for verification.
#[derive(Debug, Clone, PartialEq)]
pub struct EventRecord<T> {
    /// Monotonic sequence for deterministic ordering checks.
    pub sequence: u64,
    /// Timestamp captured at publish time.
    pub timestamp: DateTime<Utc>,
    /// Event payload.
    pub payload: T,
}

impl<T> EventRecord<T> {
    /// Create a new event record with the current timestamp.
    #[must_use]
    pub fn new(sequence: u64, payload: T) -> Self {
        Self {
            sequence,
            timestamp: Utc::now(),
            payload,
        }
    }
}

/// Error type for event testing assertions.
#[derive(Debug, Clone)]
pub struct EventTestError {
    message: String,
}

impl EventTestError {
    /// Create a new event test error.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for EventTestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for EventTestError {}

/// Payload assertion helpers using serde serialization.
pub struct PayloadAssertion;

impl PayloadAssertion {
    /// Compare two serializable payloads for exact equality.
    pub fn verify<T>(expected: &T, actual: &T) -> Result<(), EventTestError>
    where
        T: Serialize,
    {
        let expected_value = serde_json::to_value(expected).map_err(|err| {
            EventTestError::new(format!(
                "Failed to serialize expected payload: {err}"
            ))
        })?;
        let actual_value = serde_json::to_value(actual).map_err(|err| {
            EventTestError::new(format!(
                "Failed to serialize actual payload: {err}"
            ))
        })?;

        if expected_value == actual_value {
            Ok(())
        } else {
            Err(EventTestError::new(format!(
                "Payload mismatch. Expected {expected_value:?}, got {actual_value:?}"
            )))
        }
    }

    /// Convert a payload into a JSON value for custom inspection.
    pub fn to_value<T>(payload: &T) -> Result<Value, EventTestError>
    where
        T: Serialize,
    {
        serde_json::to_value(payload).map_err(|err| {
            EventTestError::new(format!("Failed to serialize payload: {err}"))
        })
    }
}

/// Sequence assertion helpers for ordered event streams.
pub struct SequenceAssertion;

impl SequenceAssertion {
    /// Ensure event sequences increase monotonically.
    pub fn verify_increasing<T>(
        records: &[EventRecord<T>],
    ) -> Result<(), EventTestError> {
        let mut previous: Option<u64> = None;
        for record in records {
            if let Some(previous_sequence) = previous
                && record.sequence <= previous_sequence
            {
                return Err(EventTestError::new(format!(
                    "Sequence {current} must be greater than {previous_sequence}",
                    current = record.sequence
                )));
            }
            previous = Some(record.sequence);
        }
        Ok(())
    }
}

/// Given-When-Then test framework for CQRS event sourcing.
pub struct EventTestFramework;

impl EventTestFramework {
    /// Start a scenario with a list of historical events.
    pub fn given<E>(
        events: impl IntoIterator<Item = E>,
    ) -> EventTestScenario<E> {
        EventTestScenario {
            given_events: events.into_iter().collect(),
        }
    }
}

/// Scenario builder for Given-When-Then tests.
pub struct EventTestScenario<E> {
    given_events: Vec<E>,
}

impl<E> EventTestScenario<E> {
    /// Execute the command handler to produce new events.
    pub fn when<F>(self, handler: F) -> EventTestResult<E>
    where
        F: FnOnce(Vec<E>) -> Vec<E>,
    {
        let published_events = handler(self.given_events);
        EventTestResult {
            published_events,
        }
    }
}

/// Result stage for asserting expected events.
pub struct EventTestResult<E> {
    published_events: Vec<E>,
}

impl<E> EventTestResult<E> {
    /// Assert that published events match the expected sequence.
    pub fn then_expect_events(
        self,
        expected: &[E],
    ) -> Result<(), EventTestError>
    where
        E: PartialEq + fmt::Debug,
    {
        if self.published_events == expected {
            Ok(())
        } else {
            Err(EventTestError::new(format!(
                "Expected events {expected:?}, got {actual:?}",
                actual = self.published_events
            )))
        }
    }

    /// Access the published events for further assertions.
    #[must_use]
    pub fn events(&self) -> &[E] {
        &self.published_events
    }
}

#[cfg(test)]
// # LINT_DISABLE_REASON: Assertion macros in tests trigger disallowed-method linting.
// # LINT_DISABLE_REASON: Options tried: manual Result propagation.
// # LINT_DISABLE_REASON: Justification: maintain clear test intent.
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize)]
    struct TestEvent {
        id: u32,
    }

    #[test]
    fn given_when_then_matches_expected_events() {
        let expected = vec![TestEvent {
            id: 2,
        }];
        let result = EventTestFramework::given(vec![TestEvent {
            id: 1,
        }])
        .when(|_history| {
            vec![TestEvent {
                id: 2,
            }]
        })
        .then_expect_events(&expected);

        assert!(result.is_ok());
    }

    #[test]
    fn given_when_then_reports_mismatched_events() {
        let expected = vec![TestEvent {
            id: 3,
        }];
        let result = EventTestFramework::given(vec![TestEvent {
            id: 1,
        }])
        .when(|_history| {
            vec![TestEvent {
                id: 2,
            }]
        })
        .then_expect_events(&expected);

        assert!(result.is_err());
    }

    #[test]
    fn payload_assertion_detects_mismatch() {
        let expected = TestEvent {
            id: 1,
        };
        let actual = TestEvent {
            id: 2,
        };
        let result = PayloadAssertion::verify(&expected, &actual);

        assert!(result.is_err());
    }

    #[test]
    fn sequence_assertion_detects_out_of_order() {
        let records = vec![
            EventRecord::new(
                1,
                TestEvent {
                    id: 1,
                },
            ),
            EventRecord::new(
                1,
                TestEvent {
                    id: 2,
                },
            ),
        ];
        let result = SequenceAssertion::verify_increasing(&records);

        assert!(result.is_err());
    }

    #[test]
    fn sequence_assertion_accepts_increasing_sequences() {
        let records = vec![
            EventRecord::new(
                1,
                TestEvent {
                    id: 1,
                },
            ),
            EventRecord::new(
                2,
                TestEvent {
                    id: 2,
                },
            ),
        ];
        let result = SequenceAssertion::verify_increasing(&records);

        assert!(result.is_ok());
    }

    #[test]
    fn payload_assertion_to_value_serializes_payload() {
        let event = TestEvent {
            id: 9,
        };
        let value_result = PayloadAssertion::to_value(&event);

        assert!(
            matches!(value_result.as_ref(), Ok(value) if *value == serde_json::json!({"id": 9})),
            "unexpected serialized payload: {value_result:?}"
        );
    }

    #[test]
    fn event_test_error_formats_message() {
        let error = EventTestError::new("oops");

        assert_eq!(error.to_string(), "oops");
    }

    #[derive(Debug, Clone, PartialEq)]
    struct MaybeSerialize {
        value: u32,
        fail: bool,
    }

    impl Serialize for MaybeSerialize {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            if self.fail {
                Err(serde::ser::Error::custom("fail"))
            } else {
                serializer.serialize_u32(self.value)
            }
        }
    }

    #[test]
    fn payload_assertion_reports_expected_serialization_failure() {
        let expected = MaybeSerialize {
            value: 1,
            fail: true,
        };
        let actual = MaybeSerialize {
            value: 2,
            fail: false,
        };

        let result = PayloadAssertion::verify(&expected, &actual);
        assert!(result.is_err());
    }

    #[test]
    fn payload_assertion_reports_actual_serialization_failure() {
        let expected = MaybeSerialize {
            value: 1,
            fail: false,
        };
        let actual = MaybeSerialize {
            value: 2,
            fail: true,
        };

        let result = PayloadAssertion::verify(&expected, &actual);
        assert!(result.is_err());
    }

    #[test]
    fn payload_to_value_reports_serialization_failure() {
        let payload = MaybeSerialize {
            value: 1,
            fail: true,
        };

        let result = PayloadAssertion::to_value(&payload);
        assert!(result.is_err());
    }

    #[test]
    fn event_test_result_exposes_published_events() {
        let result = EventTestFramework::given(vec![TestEvent {
            id: 1,
        }])
        .when(|_history| {
            vec![TestEvent {
                id: 2,
            }]
        });

        assert_eq!(result.events().len(), 1);
    }
}
