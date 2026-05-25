//! Storage implementation for Note persistence.
//!
//! This module provides the redb adapter implementing the Note repository
//! traits.

pub(crate) mod tables;

pub use tables::{LIST_VIEWS, NOTE_ID_BY_PATH, NOTES};
