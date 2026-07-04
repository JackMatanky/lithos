#![allow(
    missing_docs,
    clippy::missing_errors_doc,
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
