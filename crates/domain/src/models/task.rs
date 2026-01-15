//! Task subentity for Note aggregate.
//!
//! Represents task items with completion status within notes.

use crate::errors::DomainError;

/// Represents the status of a task item.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "TaskStatus is the correct domain name for task statuses"
)]
pub enum TaskStatus {
    /// Task is cancelled: `- [-] Task description`.
    Cancelled,
    /// Task is complete: `- [x] Task description`.
    Complete,
    /// Task is incomplete: `- [ ] Task description`.
    Incomplete,
}

/// Represents a task item within a note.
///
/// Tasks provide todo list functionality within notes and can be
/// tracked for completion status.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Task {
    /// Character position in the source document.
    pub position: usize,
    /// Current completion status.
    pub status: TaskStatus,
    /// Task description text.
    pub text: Box<str>,
}

impl Task {
    /// Creates a new task item.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::task::{Task, TaskStatus};
    ///
    /// let task = Task::new(
    ///     "Buy milk".to_string(),
    ///     TaskStatus::Incomplete,
    ///     50
    /// ).unwrap();
    /// assert_eq!(task.text.as_ref(), "Buy milk");
    /// assert_eq!(task.status, TaskStatus::Incomplete);
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if the task text is empty or contains only whitespace.
    #[inline]
    pub fn new(
        text: String,
        status: TaskStatus,
        position: usize,
    ) -> Result<Self, DomainError> {
        if text.trim().is_empty() {
            return Err(DomainError::ValidationFailed(
                "Task text cannot be empty".to_owned(),
            ));
        }

        Ok(Self {
            position,
            status,
            text: text.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod new {
        use super::*;

        #[test]
        fn succeeds_for_valid_input() {
            // GIVEN valid task parameters
            let text = "Buy milk".to_owned();
            let status = TaskStatus::Incomplete;
            let position = 50;

            // WHEN creating a new task
            let result = Task::new(text, status, position).unwrap();

            // THEN it has the correct values
            assert_eq!(result.text.as_ref(), "Buy milk");
            assert_eq!(result.status, TaskStatus::Incomplete);
            assert_eq!(result.position, 50);
        }

        #[test]
        fn returns_error_for_empty_text() {
            // GIVEN empty task text
            let text = "   ".to_owned();

            // WHEN creating a new task
            let result = Task::new(text, TaskStatus::Complete, 0);

            // THEN it returns ValidationFailed
            assert!(matches!(result, Err(DomainError::ValidationFailed(_))));
        }
    }
}
