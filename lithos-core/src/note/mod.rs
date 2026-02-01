//! Note bounded context module.
//!
//! This module contains all entities, value objects, and logic related to the
//! Note aggregate and its subentities in the domain layer.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

/// Core Note aggregate root and main entities.
pub(crate) mod aggregate;
/// Note errors.
pub mod error;
/// Note domain events.
pub(crate) mod events;
/// Frontmatter value objects and logic.
pub(crate) mod frontmatter;
/// Link subentity for Note aggregate.
pub(crate) mod link;
/// Note ports for CQRS.
pub mod ports;
/// Document structure subentities (Heading and Section) for Note aggregate.
pub(crate) mod structure;
/// Tag subentity for Note aggregate.
pub(crate) mod tag;
/// Task subentity for Note aggregate.
pub(crate) mod task;
// --- Public API & Re-exports ---
pub type Note = aggregate::Note;
pub type NoteError = error::NoteError;
pub type FrontmatterValidated = events::FrontmatterValidated;
pub type NoteCreated = events::NoteCreated;
pub type NoteEvents = events::NoteEvents;
pub type FieldValue = frontmatter::FieldValue;
pub trait FromFieldValue: frontmatter::FromFieldValue {}
impl<T> FromFieldValue for T where T: frontmatter::FromFieldValue + ?Sized {}
pub type Frontmatter = frontmatter::Frontmatter;
pub type Anchor = link::Anchor;
pub type EmbedType = link::EmbedType;
pub type Link = link::Link;
pub type Style = link::Style;
pub type Target = link::Target;

pub trait Command: ports::Command {}
impl<T> Command for T where T: ports::Command + ?Sized {}

pub trait Query: ports::Query {}
impl<T> Query for T where T: ports::Query + ?Sized {}

pub type Heading = structure::Heading;
pub type Section = structure::Section;
pub type Tag = tag::Tag;
pub type Task = task::Task;
pub type TaskStatus = task::TaskStatus;
