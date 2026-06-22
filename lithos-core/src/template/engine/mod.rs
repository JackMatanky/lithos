//! Template rendering engine port.
//!
//! The port accepts already-loaded Template domain values and already-supplied
//! render context data. It owns only engine-level source checking/loading and
//! rendering, leaving repository lookup, target resolution, conflict checks,
//! and file commits to the Template service/write pipeline.

#![allow(dead_code, reason = "template service wiring lands in a later slice")]

use std::collections::HashMap;

use super::{
    Template,
    artifact::{Rendered, TemplateArtifact},
};

mod error;
pub(crate) mod mini_jinja;

pub use error::TemplateEngineError;

/// Rendering-engine boundary for checking and rendering supplied templates.
///
/// The trait intentionally has no `Clone`, `Send`, `Sync`, or `'static` bounds;
/// concrete orchestration needs should drive future bounds.
pub(crate) trait TemplateEngine {
    /// Registers and checks the supplied template source in the engine.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateEngineError`] when the engine rejects the template
    /// source.
    fn compile(
        &mut self,
        template: &Template,
    ) -> Result<(), TemplateEngineError>;

    /// Renders a compiled template with a flat string context.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateEngineError`] when the template was not registered or
    /// rendering fails.
    fn render(
        &self,
        template: &Template,
        context: &HashMap<String, String>,
    ) -> Result<TemplateArtifact<Rendered>, TemplateEngineError>;
}
