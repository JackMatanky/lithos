//! Task subentity for Note aggregate.
//!
//! Represents task items with completion status within notes.
#![allow(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates Archived types with public fields/variants"
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
pub struct Task {
    /// Task description text.
    text: Box<str>,
    /// Current completion status.
    status: TaskStatus,
    /// Character position in the source document.
    position: usize,
}

/// Represents the status of a task item.
///
/// # Examples
/// ```
/// # use lithos_core::note::task::TaskStatus;
/// assert_eq!(
///     TaskStatus::Complete as u8,
///     TaskStatus::Complete as u8,
///     "Enum discriminant should be stable"
/// );
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let task = Task::new("Buy milk".to_string(), TaskStatus::Incomplete, 50)?;
    /// assert_eq!(task.text(), "Buy milk", "Task text should match");
    /// assert_eq!(
    ///     task.status(),
    ///     TaskStatus::Incomplete,
    ///     "Task status should match"
    /// );
    /// # Ok(())
    /// # }
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
    reason = "Test setup uses expect for deterministic fixtures."
)]
mod tests {
    use super::*;

    mod constructor {
        use super::*;

        #[test]
        fn new_sets_text() {
            let task = Task::new("Review".to_owned(), TaskStatus::Cancelled, 5)
                .expect("Task should be created successfully");
            assert_eq!(task.text(), "Review", "Task text should be 'Review'");
        }

        #[test]
        fn new_sets_status() {
            let task = Task::new("Review".to_owned(), TaskStatus::Cancelled, 5)
                .expect("Task should be created successfully");
            assert_eq!(
                task.status(),
                TaskStatus::Cancelled,
                "Task status should be Cancelled"
            );
        }

        #[test]
        fn new_sets_position() {
            let task = Task::new("Review".to_owned(), TaskStatus::Cancelled, 5)
                .expect("Task should be created successfully");
            assert_eq!(task.position(), 5, "Task position should be 5");
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
