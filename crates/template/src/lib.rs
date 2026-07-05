//! Template rendering engine with file output.
//!
//! Two-layer architecture: [`TemplateEngine`] port abstracts rendering;
//! [`TemplateService`] orchestrates render-then-write with dry-run support.
//! Ships one implementation backed by minijinja ([`MiniJinjaEngine`]).
//!
//! Re-exports all public types at the crate root.

#![allow(
    clippy::impl_trait_in_params,
    reason = "Rust 2024 lint — crate-internal API uses impl Trait"
)]

pub(crate) mod artifact;
pub(crate) mod engine;
/// Error types for the template system.
pub mod error;
pub(crate) mod name;
pub(crate) mod service;

pub use engine::{
    RenderedTemplate, TemplateEngine, mini_jinja::MiniJinjaEngine,
};
pub use error::{
    TemplateArtifactError, TemplateEngineError, TemplateError,
    TemplateNameError,
};
pub use name::TemplateName;
pub use service::{
    CreateTemplateInput, CreateTemplateOutcome, TemplateService,
};
