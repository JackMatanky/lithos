//! Template context domain types.
//!
//! Provides value objects and aggregates for the Template bounded context:
//! - [`TemplateId`] — UUID v7-based identifier
//! - [`TemplateName`] — path-derived, subdirectory-qualified template name
//! - [`TemplateBody`] — non-empty renderable source text
//! - [`RawTemplate`] — thin raw-content newtype for ingestion pipeline
//! - [`RawTemplateView`] — flat freshness/cache struct
//! - [`Template`] — primary renderable aggregate
//! - [`TemplateError`], [`TemplateNameError`], [`TemplateBodyError`],
//!   [`TemplateRepositoryError`], [`TemplateEngineError`] — domain, repository,
//!   and engine errors
//!
//! Domain and service-facing APIs stay free of `MiniJinja` imports. Rendering
//! mechanics are confined to the engine adapter, while engine errors preserve
//! their source chain for diagnostics.

pub(crate) mod aggregate;
pub(crate) mod artifact;
pub(crate) mod engine;
pub(crate) mod error;
pub(crate) mod processor;
pub(crate) mod raw;
pub(crate) mod repository;
pub(crate) mod service;
pub(crate) mod storage;
pub(crate) mod views;

pub use aggregate::{Template, TemplateBody, TemplateId, TemplateName};
pub use engine::TemplateEngineError;
pub use error::{
    TemplateBodyError, TemplateError, TemplateNameError,
    TemplateRepositoryError,
};
pub use raw::RawTemplate;
pub use repository::{ReadRepository, Repository, WriteRepository};
pub use service::TemplateService;
pub use views::RawTemplateView;

// ============================================================================
// Policy enforcement (tracer bullet 17)
// ============================================================================

#[cfg(test)]
#[allow(clippy::panic, reason = "tests use panic for assertions")]
mod policy {
    /// Verifies that no import/use of the rendering-engine crate appears
    /// anywhere in the template context module source files.
    ///
    /// This is a policy invariant: Jinja syntax validation is the engine's
    /// responsibility, not the domain's.
    #[test]
    fn no_rendering_engine_import_in_template_context() {
        let template_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("template");

        let source_files = [
            template_dir.join("mod.rs"),
            template_dir.join("artifact.rs"),
            template_dir.join("aggregate.rs"),
            template_dir.join("engine").join("mod.rs"),
            template_dir.join("error.rs"),
            template_dir.join("raw.rs"),
            template_dir.join("repository.rs"),
            template_dir.join("service.rs"),
            template_dir.join("views.rs"),
        ];

        // Build the forbidden crate name at runtime to avoid the literal
        // appearing in this source file and causing the test to falsely
        // flag itself. The crate name is: "mini" + "j" + "inja"
        let engine_crate = ["mini", "j", "inja"].concat();

        let forbidden_patterns = [
            format!("use {engine_crate}"),
            format!("extern crate {engine_crate}"),
            format!("::{engine_crate}::"),
        ];

        for path in &source_files {
            let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
                panic!("Could not read {}: {e}", path.display())
            });
            for pattern in &forbidden_patterns {
                assert!(
                    !content.contains(pattern.as_str()),
                    "Found '{}' import in {}. Template domain types must not \
                     import the rendering engine crate.",
                    engine_crate,
                    path.display()
                );
            }
        }
    }

    /// Verifies that rendering-engine imports stay inside the engine adapter
    /// rather than leaking into template domain or service-facing modules.
    #[test]
    fn rendering_engine_imports_are_confined_to_engine_adapter() {
        let template_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("template");

        let engine_crate = ["mini", "j", "inja"].concat();
        let forbidden_patterns = [
            format!("use {engine_crate}"),
            format!("extern crate {engine_crate}"),
            format!("::{engine_crate}::"),
        ];

        let forbidden_files = [
            template_dir.join("mod.rs"),
            template_dir.join("artifact.rs"),
            template_dir.join("aggregate.rs"),
            template_dir.join("engine").join("mod.rs"),
            template_dir.join("error.rs"),
            template_dir.join("raw.rs"),
            template_dir.join("repository.rs"),
            template_dir.join("service.rs"),
            template_dir.join("views.rs"),
        ];

        for path in &forbidden_files {
            let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
                panic!("Could not read {}: {e}", path.display())
            });
            for pattern in &forbidden_patterns {
                assert!(
                    !content.contains(pattern.as_str()),
                    "Found '{}' import in {}. Rendering-engine imports must \
                     stay confined to the engine adapter.",
                    engine_crate,
                    path.display()
                );
            }
        }

        let engine_adapter = template_dir.join("engine").join("mini_jinja.rs");
        let content =
            std::fs::read_to_string(&engine_adapter).unwrap_or_else(|e| {
                panic!("Could not read {}: {e}", engine_adapter.display())
            });
        assert!(
            content.contains(&format!("use {engine_crate}")),
            "Expected rendering-engine import in {}",
            engine_adapter.display()
        );
    }
}
