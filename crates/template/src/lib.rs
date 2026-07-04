#![allow(
    clippy::as_conversions,
    clippy::impl_trait_in_params,
    clippy::missing_errors_doc,
    missing_docs,
    reason = "template crate redesign"
)]

pub(crate) mod artifact;
pub(crate) mod engine;
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
