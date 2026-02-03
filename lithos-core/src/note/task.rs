//! Task subentity for Note aggregate.
//!
//! Represents task items with completion status within notes.
#![allow(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates Archived types with public fields/variants"
)]
#![expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "pub(crate) used for internal builders and tests"
)]

use super::error::NoteError;

/// Represents a task item within a note.
///
/// Tasks provide todo list functionality within notes and can be
/// tracked for completion status.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "pub(crate) used for internal builders and tests"
)]
pub struct Task {
    /// Task description text.
    pub(crate) text: Box<str>,
    /// Current completion status.
    pub(crate) status: TaskStatus,
    /// Character position in the source document.
    pub(crate) position: usize,
}

/// Represents the status of a task item.
///
/// # Examples
/// ```
/// # use lithos_core::note::task::TaskStatus;
/// assert_eq!(TaskStatus::Complete as u8, TaskStatus::Complete as u8);
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum TaskStatus {
    /// Task is cancelled: `- [-] Task description`.
    Cancelled,
    /// Task is complete: `- [x] Task description`.
    Complete,
    /// Task is incomplete: `- [ ] Task description`.
    Incomplete,
}

impl Task {
    /// Creates a new task item.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::note::task::{Task, TaskStatus};
    ///
    /// let task =
    ///     Task::new("Buy milk".to_string(), TaskStatus::Incomplete, 50).unwrap();
    /// assert_eq!(task.text(), "Buy milk");
    /// assert_eq!(task.status(), TaskStatus::Incomplete);
    /// ```
    ///
    /// # Errors
    /// Returns `NoteError::Task` if the task text is empty or contains only
    /// whitespace.
    #[inline]
    pub fn new(
        text: String,
        status: TaskStatus,
        position: usize,
    ) -> Result<Self, NoteError> {
        if text.trim().is_empty() {
            return Err(NoteError::Task(
                "Task text cannot be empty".to_owned(),
            ));
        }

        Ok(Self {
            text: text.into(),
            status,
            position,
        })
    }

    /// Returns the character position in the source document.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }

    /// Returns the current completion status.
    #[inline]
    #[must_use]
    pub const fn status(&self) -> TaskStatus {
        self.status
    }

    /// Returns the task description text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test module uses Result::unwrap() for ergonomic arrangement and \
              assertions. Acceptable in test-only code paths."
)]
mod tests {
    use super::*;

    mod new {
        use super::*;

        #[test]
        fn accessors_return_expected_values() {
            // GIVEN: a task
            let task = Task::new("Review".to_owned(), TaskStatus::Cancelled, 5)
                .unwrap();

            // THEN: accessors return expected values
            assert_eq!(task.text(), "Review", "Task text should be 'Review'");
            assert_eq!(
                task.status(),
                TaskStatus::Cancelled,
                "Task status should be Cancelled"
            );
            assert_eq!(task.position(), 5, "Task position should be 5");
        }

        #[test]
        fn succeeds_for_valid_input() {
            // GIVEN: valid task parameters
            let text = "Buy milk".to_owned();
            let status = TaskStatus::Incomplete;
            let position = 50;

            // WHEN: creating a new task
            #[expect(
                clippy::disallowed_methods,
                reason = "Setup phase - test fixture creation"
            )]
            let result = Task::new(text, status, position).unwrap();

            // THEN: it has the correct values
            assert_eq!(
                result.text(),
                "Buy milk",
                "Task text should be 'Buy milk'"
            );
            assert_eq!(
                result.status(),
                TaskStatus::Incomplete,
                "Task status should be Incomplete"
            );
            assert_eq!(result.position(), 50, "Task position should be 50");
        }

        #[test]
        fn returns_error_for_empty_text() {
            // GIVEN: empty task text
            let text = "   ".to_owned();

            // WHEN: creating a new task
            let result = Task::new(text, TaskStatus::Complete, 0);

            // THEN: it returns ValidationFailed
            assert!(
                matches!(result, Err(NoteError::Task(_))),
                "Empty task text should be rejected with Task error, got: \
                 {result:?}"
            );
        }
    }
}
