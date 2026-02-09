//! Shared domain types for the Note context.
#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive archived types"
)]

use rkyv::{Archive, Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a Note.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct NoteId(Uuid);

impl NoteId {
    /// Creates a new random `NoteId` (UUID v7).
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for NoteId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for NoteId {
    #[inline]
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<NoteId> for Uuid {
    #[inline]
    fn from(id: NoteId) -> Self {
        id.0
    }
}

/// A byte offset into the original UTF-8 source of a note.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct SourceByteOffset(u32);

impl SourceByteOffset {
    /// Creates a new `SourceByteOffset`.
    #[inline]
    #[must_use]
    pub const fn new(offset: u32) -> Self {
        Self(offset)
    }
}

impl From<u32> for SourceByteOffset {
    #[inline]
    fn from(offset: u32) -> Self {
        Self(offset)
    }
}

impl From<SourceByteOffset> for u32 {
    #[inline]
    fn from(offset: SourceByteOffset) -> Self {
        offset.0
    }
}

impl TryFrom<usize> for SourceByteOffset {
    type Error = std::num::TryFromIntError;

    #[inline]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u32::try_from(value).map(Self)
    }
}

impl From<SourceByteOffset> for usize {
    #[inline]
    fn from(offset: SourceByteOffset) -> Self {
        // Safe because u32 always fits in usize on Lithos supported platforms
        #[expect(
            clippy::as_conversions,
            reason = "Safe conversion from u32 to usize"
        )]
        {
            offset.0 as usize
        }
    }
}

/// A byte range in the original UTF-8 source.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SourceByteRange {
    /// Start of the range (inclusive).
    pub start: SourceByteOffset,
    /// End of the range (exclusive).
    pub end: SourceByteOffset,
}

impl SourceByteRange {
    /// Creates a new range from start and end offsets.
    #[inline]
    #[must_use]
    pub const fn new(start: SourceByteOffset, end: SourceByteOffset) -> Self {
        Self {
            start,
            end,
        }
    }

    /// Returns the length of the range in bytes.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.end.0.saturating_sub(self.start.0)
    }

    /// Returns true if the range is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start.0 >= self.end.0
    }
}

/// Heading level (1-6).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct HeadingLevel(u8);

impl HeadingLevel {
    /// Creates a new `HeadingLevel`, validating it is between 1 and 6.
    ///
    /// # Errors
    /// Returns an error if the level is not in the range 1..=6.
    #[inline]
    pub fn try_new(level: u8) -> Result<Self, crate::note::error::NoteError> {
        if (1..=6).contains(&level) {
            Ok(Self(level))
        } else {
            Err(crate::note::error::NoteError::Structure(format!(
                "Invalid heading level: {level}. Must be between 1 and 6."
            )))
        }
    }

    /// Returns the raw level value.
    #[inline]
    #[must_use]
    pub const fn as_u8(&self) -> u8 {
        self.0
    }
}

/// A timestamp representing task temporal data.
///
/// Wraps an `i64` Unix timestamp for semantic clarity while maintaining
/// zero-copy compatibility with rkyv serialization.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct TaskTimestamp(i64);

impl TaskTimestamp {
    /// Creates a new `TaskTimestamp` from a Unix timestamp.
    ///
    /// # Arguments
    /// * `timestamp` - Unix timestamp in seconds since epoch.
    #[inline]
    #[must_use]
    pub const fn new(timestamp: i64) -> Self {
        Self(timestamp)
    }

    /// Creates a new `TaskTimestamp` from the current time.
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        #[expect(
            clippy::cast_possible_wrap,
            clippy::as_conversions,
            reason = "Unix timestamp fits in i64 for Lithos time range"
        )]
        Self(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        )
    }

    /// Returns the raw Unix timestamp.
    #[inline]
    #[must_use]
    pub const fn as_i64(&self) -> i64 {
        self.0
    }

    /// Returns true if this timestamp represents a future time.
    ///
    /// # Arguments
    /// * `relative_to` - Optional reference time; defaults to now.
    #[inline]
    #[must_use]
    pub fn is_future(&self, relative_to: Option<Self>) -> bool {
        let reference = relative_to.unwrap_or_else(Self::now);
        self.0 > reference.0
    }

    /// Returns true if this timestamp represents a past time.
    ///
    /// # Arguments
    /// * `relative_to` - Optional reference time; defaults to now.
    #[inline]
    #[must_use]
    pub fn is_past(&self, relative_to: Option<Self>) -> bool {
        let reference = relative_to.unwrap_or_else(Self::now);
        self.0 < reference.0
    }

    /// Returns the duration from now in seconds (positive for future, negative
    /// for past).
    #[inline]
    #[must_use]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Timestamp arithmetic is safe"
    )]
    pub fn seconds_from_now(&self) -> i64 {
        self.0 - Self::now().0
    }

    /// Returns the duration between two timestamps.
    #[inline]
    #[must_use]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Timestamp arithmetic is safe"
    )]
    pub const fn duration_from(&self, other: Self) -> i64 {
        self.0 - other.0
    }

    /// Returns true if this timestamp is within the specified duration from
    /// now.
    ///
    /// # Arguments
    /// * `duration_seconds` - Duration window in seconds.
    /// * `relative_to` - Optional reference time; defaults to now.
    #[inline]
    #[must_use]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Timestamp arithmetic is safe"
    )]
    pub fn is_within(
        &self,
        duration_seconds: i64,
        relative_to: Option<Self>,
    ) -> bool {
        let reference = relative_to.unwrap_or_else(Self::now);
        let diff = (self.0 - reference.0).abs();
        diff <= duration_seconds
    }
}

impl Default for TaskTimestamp {
    #[inline]
    fn default() -> Self {
        Self::now()
    }
}

impl From<i64> for TaskTimestamp {
    #[inline]
    fn from(timestamp: i64) -> Self {
        Self(timestamp)
    }
}

impl From<TaskTimestamp> for i64 {
    #[inline]
    fn from(timestamp: TaskTimestamp) -> Self {
        timestamp.0
    }
}

impl From<std::time::SystemTime> for TaskTimestamp {
    #[inline]
    fn from(time: std::time::SystemTime) -> Self {
        #[expect(
            clippy::cast_possible_wrap,
            clippy::as_conversions,
            reason = "Unix timestamp fits in i64 for Lithos time range"
        )]
        Self(
            time.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        )
    }
}

impl From<TaskTimestamp> for std::time::SystemTime {
    #[inline]
    fn from(timestamp: TaskTimestamp) -> Self {
        #[expect(
            clippy::cast_sign_loss,
            clippy::as_conversions,
            reason = "Timestamp is non-negative for Duration conversion"
        )]
        std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(timestamp.0 as u64))
            .unwrap_or(std::time::UNIX_EPOCH)
    }
}

#[cfg(test)]
mod task_timestamp_tests {
    use super::*;

    #[test]
    fn new_creates_timestamp_from_i64() {
        let timestamp = TaskTimestamp::new(1_234_567_890);
        assert_eq!(timestamp.as_i64(), 1_234_567_890);
    }

    #[test]
    fn now_creates_current_timestamp() {
        #[expect(
            clippy::cast_possible_wrap,
            clippy::as_conversions,
            reason = "Unix timestamp fits in i64 for Lithos time range"
        )]
        let before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let timestamp = TaskTimestamp::now();

        #[expect(
            clippy::cast_possible_wrap,
            clippy::as_conversions,
            reason = "Unix timestamp fits in i64 for Lithos time range"
        )]
        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        assert!(timestamp.as_i64() >= before);
        assert!(timestamp.as_i64() <= after);
    }

    #[test]
    fn is_future_detects_future_timestamps() {
        let future = TaskTimestamp::now().as_i64() + 3600; // 1 hour from now
        let future_timestamp = TaskTimestamp::new(future);

        assert!(future_timestamp.is_future(None));
        assert!(!future_timestamp.is_past(None));
    }

    #[test]
    fn is_past_detects_past_timestamps() {
        let past = TaskTimestamp::now().as_i64() - 3600; // 1 hour ago
        let past_timestamp = TaskTimestamp::new(past);

        assert!(past_timestamp.is_past(None));
        assert!(!past_timestamp.is_future(None));
    }

    #[test]
    fn seconds_from_now_calculates_duration() {
        let future = TaskTimestamp::now().as_i64() + 1800; // 30 minutes from now
        let future_timestamp = TaskTimestamp::new(future);

        let seconds = future_timestamp.seconds_from_now();
        // Allow for small timing variations
        assert!((1790..=1810).contains(&seconds));
    }

    #[test]
    fn duration_from_calculates_difference() {
        let timestamp1 = TaskTimestamp::new(1000);
        let timestamp2 = TaskTimestamp::new(1500);

        assert_eq!(timestamp2.duration_from(timestamp1), 500);
        assert_eq!(timestamp1.duration_from(timestamp2), -500);
    }

    #[test]
    fn is_within_detects_proximity() {
        let now = TaskTimestamp::now();
        let near_future = TaskTimestamp::new(now.as_i64() + 60); // 1 minute from now
        let far_future = TaskTimestamp::new(now.as_i64() + 7200); // 2 hours from now

        assert!(near_future.is_within(120, None)); // Within 2 minutes
        assert!(!far_future.is_within(120, None)); // Not within 2 minutes
    }

    #[test]
    fn from_i64_converts_correctly() {
        let timestamp = TaskTimestamp::from(987_654_321);
        assert_eq!(timestamp.as_i64(), 987_654_321);
    }

    #[test]
    fn into_i64_converts_correctly() {
        let timestamp = TaskTimestamp::new(987_654_321);
        let i64_value: i64 = timestamp.into();
        assert_eq!(i64_value, 987_654_321);
    }

    #[test]
    fn system_time_conversions() {
        let system_time = std::time::SystemTime::now();
        let timestamp = TaskTimestamp::from(system_time);
        let converted_back: std::time::SystemTime = timestamp.into();

        // Allow for small differences due to rounding
        let duration =
            converted_back.duration_since(system_time).unwrap_or_default();
        assert!(duration.as_secs() < 2);
    }

    #[test]
    fn default_uses_now() {
        let default1 = TaskTimestamp::default();
        let default2 = TaskTimestamp::default();

        // Both should be close to now
        let now = TaskTimestamp::now();
        assert!(default1.as_i64() <= now.as_i64());
        assert!(default2.as_i64() <= now.as_i64());
    }

    #[test]
    fn ordering_works_correctly() {
        let earlier = TaskTimestamp::new(1000);
        let later = TaskTimestamp::new(2000);

        assert!(earlier < later);
        assert!(later > earlier);
        assert_eq!(earlier, TaskTimestamp::new(1000));
    }

    #[test]
    fn relative_time_comparisons() {
        let reference = TaskTimestamp::new(1000);
        let past = TaskTimestamp::new(500);
        let future = TaskTimestamp::new(1500);

        assert!(past.is_past(Some(reference)));
        assert!(future.is_future(Some(reference)));
        assert!(!reference.is_past(Some(reference)));
        assert!(!reference.is_future(Some(reference)));
    }
}
