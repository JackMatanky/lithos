//! Event identifier primitives and redb contracts.

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Canonical event-log identifier.
///
/// `EventId` is a strictly monotonic, positive sequence value within a single
/// context event stream. Gaps are allowed.
pub struct EventId(u64);

impl EventId {
    #[cfg(test)]
    fn new_unchecked(value: u64) -> Self {
        Self(value)
    }

    #[inline]
    #[must_use]
    /// Returns the numeric event identifier.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
/// Parse and conversion errors for [`EventId`].
pub enum EventIdError {
    /// Input byte slice is empty.
    #[error("event id bytes cannot be empty")]
    EmptyBytes,
    /// Input byte slice length differs from the required fixed width.
    #[error("invalid event id length: expected {expected}, got {got}")]
    InvalidLength {
        /// Required byte width.
        expected: usize,
        /// Actual byte width provided.
        got: usize,
    },
    /// Zero is reserved and cannot represent a committed event.
    #[error("event id zero is not allowed")]
    ZeroNotAllowed,
}

impl TryFrom<u64> for EventId {
    type Error = EventIdError;

    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(EventIdError::ZeroNotAllowed);
        }

        Ok(Self(value))
    }
}

impl TryFrom<[u8; 8]> for EventId {
    type Error = EventIdError;

    #[inline]
    fn try_from(bytes: [u8; 8]) -> Result<Self, Self::Error> {
        Self::try_from(u64::from_be_bytes(bytes))
    }
}

impl TryFrom<&[u8]> for EventId {
    type Error = EventIdError;

    #[inline]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            return Err(EventIdError::EmptyBytes);
        }

        if bytes.len() != 8 {
            return Err(EventIdError::InvalidLength {
                expected: 8,
                got: bytes.len(),
            });
        }

        let mut array = [0_u8; 8];
        array.copy_from_slice(bytes);
        Self::try_from(array)
    }
}

impl redb::Value for EventId {
    type AsBytes<'a>
        = [u8; 8]
    where
        Self: 'a;
    type SelfType<'a>
        = EventId
    where
        Self: 'a;

    #[inline]
    fn fixed_width() -> Option<usize> {
        Some(8)
    }

    #[inline]
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        #[expect(
            clippy::panic,
            reason = "redb::Value trait requires panic on invalid data - no \
                      Result return type allowed"
        )]
        let Ok(event_id) = EventId::try_from(data) else {
            panic!("EventId data from database must be valid");
        };

        event_id
    }

    #[inline]
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        value.0.to_be_bytes()
    }

    #[inline]
    fn type_name() -> redb::TypeName {
        redb::TypeName::new("lithos::EventId")
    }
}

impl redb::Key for EventId {
    #[inline]
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

#[cfg(test)]
mod tests {
    use redb::{Key, Value};

    use super::*;

    mod validation {
        use super::*;

        #[test]
        fn rejects_zero_when_constructed_from_u64() {
            let result = EventId::try_from(0_u64);

            assert!(matches!(result, Err(EventIdError::ZeroNotAllowed)));
        }

        #[test]
        fn rejects_empty_bytes() {
            let result = EventId::try_from(&[][..]);

            assert!(matches!(result, Err(EventIdError::EmptyBytes)));
        }

        #[test]
        fn rejects_invalid_byte_length() {
            let result = EventId::try_from(&[0_u8; 7][..]);

            assert!(matches!(
                result,
                Err(EventIdError::InvalidLength {
                    expected: 8,
                    got: 7
                })
            ));
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn returns_event_id_for_positive_u64() {
            let result = EventId::try_from(42_u64);

            assert_eq!(result, Ok(EventId::new_unchecked(42)));
        }
    }

    mod ordering {
        use super::*;

        #[test]
        fn preserves_strict_monotonic_ordering_with_gaps_allowed() {
            let previous = EventId::try_from(1_u64).expect("valid event id");
            let next_with_gap =
                EventId::try_from(10_u64).expect("valid event id");

            assert!(next_with_gap > previous);
        }

        #[test]
        fn returns_less_when_comparing_smaller_key_bytes() {
            let smaller = EventId::try_from(2_u64).expect("valid event id");
            let larger = EventId::try_from(200_u64).expect("valid event id");

            let result = EventId::compare(
                &EventId::as_bytes(&smaller),
                &EventId::as_bytes(&larger),
            );

            assert_eq!(result, std::cmp::Ordering::Less);
        }

        #[test]
        fn does_not_panic_for_invalid_key_lengths() {
            let result = EventId::compare(&[1_u8][..], &[1_u8, 0_u8][..]);

            assert_eq!(result, std::cmp::Ordering::Less);
        }
    }

    mod serialization {
        use super::*;

        #[test]
        fn preserves_value_across_redb_roundtrip() {
            let original = EventId::try_from(99_u64).expect("valid event id");
            let bytes = EventId::as_bytes(&original);

            let decoded = EventId::from_bytes(&bytes);

            assert_eq!(decoded, original);
        }
    }

    mod integrity {
        use super::*;

        #[test]
        fn returns_fixed_width_of_eight_bytes() {
            assert_eq!(EventId::fixed_width(), Some(8));
        }

        #[test]
        fn returns_stable_redb_type_name() {
            assert_eq!(EventId::type_name().name(), "lithos::EventId");
        }
    }
}
