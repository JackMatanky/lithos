//! Event identifier primitives and redb contracts.

use std::num::NonZeroU64;

use thiserror::Error;

use crate::db::{ArchivedEntity, DbError};

#[allow(dead_code, reason = "Foundation type used in upcoming slices")]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Canonical event-log identifier.
///
/// `EventId` is a strictly monotonic, positive sequence value within a single
/// context event stream. Gaps are allowed.
pub(crate) struct EventId(NonZeroU64);

#[allow(dead_code, reason = "Foundation API used in upcoming slices")]
impl EventId {
    /// Minimum possible event identifier.
    pub(crate) const MIN: Self = Self(NonZeroU64::MIN);

    #[inline]
    #[must_use]
    /// Returns the numeric event identifier.
    pub(crate) fn get(self) -> u64 {
        self.0.get()
    }

    /// Returns the next strictly monotonic [`EventId`].
    ///
    /// # Errors
    /// Returns [`EventIdError::Overflow`] if the sequence exceeds `u64::MAX`.
    pub(crate) fn next_after(
        current_max: Option<Self>,
    ) -> Result<Self, EventIdError> {
        let next_value = match current_max {
            None => 1,
            Some(id) => {
                id.get().checked_add(1).ok_or(EventIdError::Overflow)?
            }
        };

        Self::try_from_raw(next_value)
    }

    /// Parses a raw numeric value into an [`EventId`].
    ///
    /// # Errors
    /// Returns [`EventIdError::Zero`] if the value is zero.
    pub(crate) fn try_from_raw(raw: u64) -> Result<Self, EventIdError> {
        NonZeroU64::new(raw).map(Self).ok_or(EventIdError::Zero)
    }

    /// Parses and validates monotonicity in a single functional step.
    ///
    /// # Errors
    /// - [`EventIdError::Zero`] if raw is zero.
    /// - [`EventIdError::NotStrictlyMonotonic`] if candidate is not after
    ///   previous.
    pub(crate) fn try_after(
        previous: Option<Self>,
        raw: u64,
    ) -> Result<Self, EventIdError> {
        let candidate = Self::try_from_raw(raw)?;
        if let Some(prev) = previous
            && candidate <= prev
        {
            return Err(EventIdError::NotStrictlyMonotonic {
                previous: prev,
                candidate,
            });
        }
        Ok(candidate)
    }
}

impl TryFrom<u64> for EventId {
    type Error = EventIdError;

    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::try_from_raw(value)
    }
}

impl From<NonZeroU64> for EventId {
    #[inline]
    fn from(value: NonZeroU64) -> Self {
        Self(value)
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
        value.get().to_be_bytes()
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

#[allow(dead_code, reason = "Foundation error type used in upcoming slices")]
#[derive(Clone, Debug, Eq, PartialEq, Error)]
/// Parse and conversion errors for [`EventId`].
pub(crate) enum EventIdError {
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
    Zero,
    /// Sequence violation (monotonicity).
    #[error(
        "sequence violation: candidate {candidate:?} is not after {previous:?}"
    )]
    NotStrictlyMonotonic {
        /// The previous valid identifier.
        previous: EventId,
        /// The invalid candidate identifier.
        candidate: EventId,
    },
    /// Sequence value exceeded `u64::MAX`.
    #[error("event id overflow")]
    Overflow,
}

#[allow(dead_code, reason = "Foundation allocator used in upcoming slices")]
#[derive(Clone, Debug)]
/// In-memory monotonic allocator for [`EventId`].
pub(crate) struct EventIdAllocator {
    last_issued: Option<EventId>,
}

#[allow(dead_code, reason = "Foundation allocator API used in upcoming slices")]
impl EventIdAllocator {
    /// Creates a new allocator with no issued identifiers.
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            last_issued: None,
        }
    }

    /// Returns the next strictly monotonic [`EventId`].
    ///
    /// # Errors
    /// Returns [`EventIdError::Overflow`] if the sequence exceeds `u64::MAX`.
    pub(crate) fn next(&mut self) -> Result<EventId, EventIdError> {
        let id = EventId::next_after(self.last_issued)?;
        self.last_issued = Some(id);
        Ok(id)
    }

    /// Sets allocator cursor to the maximum persisted event id.
    pub(crate) fn reserve_after(&mut self, persisted_max: Option<EventId>) {
        self.last_issued = persisted_max;
    }
}

#[allow(dead_code, reason = "Contract consumed by upcoming repository slices")]
/// Infrastructure contract for append/load/compact event-log behavior.
pub(crate) trait EventStore<E>
where
    E: ArchivedEntity,
{
    /// Append an event atomically and return the allocated event id.
    fn append(&self, event: &E) -> Result<EventId, DbError>;

    /// Load all events in deterministic ascending [`EventId`] order.
    fn load_all_events(&self) -> Result<Vec<(EventId, E)>, DbError>;

    /// Compact all events with id strictly less than `cutoff`.
    fn compact_before(&self, cutoff: EventId) -> Result<u64, DbError>;
}

#[cfg(test)]
mod tests {
    use redb::{Key, Value};

    use super::*;

    mod validation {
        use super::*;

        #[test]
        fn rejects_zero_when_constructed_from_raw() {
            let result = EventId::try_from_raw(0_u64);

            assert!(matches!(result, Err(EventIdError::Zero)));
        }

        #[test]
        fn rejects_non_monotonic_candidate() {
            let previous = EventId::try_from_raw(10).expect("valid id");

            let result = EventId::try_after(Some(previous), 10);

            assert!(matches!(
                result,
                Err(EventIdError::NotStrictlyMonotonic { .. })
            ));
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
        use std::num::NonZeroU64;

        use super::*;

        #[test]
        fn returns_event_id_for_positive_u64() {
            let result = EventId::try_from_raw(42_u64);

            assert_eq!(result.map(super::super::EventId::get), Ok(42));
        }

        #[test]
        fn returns_event_id_from_non_zero_u64() {
            let non_zero = NonZeroU64::new(7).expect("non-zero literal");

            let event_id = EventId::from(non_zero);

            assert_eq!(event_id.get(), 7);
        }
    }

    mod ordering {
        use super::*;

        #[test]
        fn preserves_strict_monotonic_ordering_with_gaps_allowed() {
            let previous =
                EventId::try_from_raw(1_u64).expect("valid event id");
            let next_with_gap =
                EventId::try_from_raw(10_u64).expect("valid event id");

            assert!(next_with_gap > previous);
        }

        #[test]
        fn returns_less_when_comparing_smaller_key_bytes() {
            let smaller = EventId::try_from_raw(2_u64).expect("valid event id");
            let larger =
                EventId::try_from_raw(200_u64).expect("valid event id");

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
            let original =
                EventId::try_from_raw(99_u64).expect("valid event id");
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

    mod allocator {
        use super::*;

        mod state {
            use super::*;

            #[test]
            fn returns_first_id_when_uninitialized() {
                let mut allocator = EventIdAllocator::new();

                let first =
                    allocator.next().expect("first allocation succeeds");

                assert_eq!(first.get(), 1);
            }

            #[test]
            fn returns_next_id_after_previous() {
                let mut allocator = EventIdAllocator::new();
                let _ = allocator.next().expect("first allocation succeeds");

                let second =
                    allocator.next().expect("second allocation succeeds");

                assert_eq!(second.get(), 2);
            }

            #[test]
            fn reserves_after_persisted_max_before_next() {
                let mut allocator = EventIdAllocator::new();
                let persisted =
                    EventId::try_from_raw(10_u64).expect("valid id");
                allocator.reserve_after(Some(persisted));

                let next = allocator.next().expect("allocation succeeds");

                assert_eq!(next.get(), 11);
            }
        }

        mod validation {
            use super::*;

            #[test]
            fn returns_error_when_next_would_overflow() {
                let mut allocator = EventIdAllocator::new();
                let max_id =
                    EventId::try_from_raw(u64::MAX).expect("max id valid");
                allocator.reserve_after(Some(max_id));

                let result = allocator.next();

                assert!(matches!(result, Err(EventIdError::Overflow)));
            }
        }

        mod next_after {
            use super::*;

            #[test]
            fn returns_overflow_error_when_incrementing_max() {
                let max_id =
                    EventId::try_from_raw(u64::MAX).expect("max id valid");

                let result = EventId::next_after(Some(max_id));

                assert!(matches!(result, Err(EventIdError::Overflow)));
            }
        }
    }

    mod event_store_contract {
        use rkyv::{Archive, Deserialize, Serialize};

        use super::*;

        #[derive(
            Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq,
        )]
        #[rkyv(derive(Debug, PartialEq, Eq))]
        struct TestEvent {
            name: String,
        }

        struct FixtureStore;

        impl EventStore<TestEvent> for FixtureStore {
            fn append(&self, _event: &TestEvent) -> Result<EventId, DbError> {
                Ok(EventId::MIN)
            }

            fn load_all_events(
                &self,
            ) -> Result<Vec<(EventId, TestEvent)>, DbError> {
                Ok(vec![(EventId::MIN, TestEvent {
                    name: "fixture".to_owned(),
                })])
            }

            fn compact_before(&self, _cutoff: EventId) -> Result<u64, DbError> {
                Ok(0)
            }
        }

        #[test]
        fn accepts_archived_entity_payload_type() {
            let store = FixtureStore;
            let event = TestEvent {
                name: "append".to_owned(),
            };

            let appended = store.append(&event).expect("append");

            assert_eq!(appended.get(), 1);
        }

        #[test]
        fn returns_events_in_event_id_pairs_for_load_all_events() {
            let store = FixtureStore;

            let loaded = store.load_all_events().expect("load all");

            assert_eq!(loaded.len(), 1);
            let (id, payload) = loaded.first().expect("one fixture event");
            assert_eq!(id.get(), 1);
            assert_eq!(payload.name, "fixture");
        }

        #[test]
        fn returns_u64_count_for_compact_before() {
            let store = FixtureStore;

            let removed = store.compact_before(EventId::MIN).expect("compact");

            assert_eq!(removed, 0);
        }
    }
}
