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
//!   [`TemplateRepositoryError`] — domain and repository errors
//!
//! All types are free of `MiniJinja` imports — Jinja syntax validation is the
//! engine's responsibility.

pub(crate) mod aggregate;
pub(crate) mod error;
pub(crate) mod processor;
pub(crate) mod raw;
pub(crate) mod repository;
pub(crate) mod storage;
pub(crate) mod views;

pub use aggregate::{Template, TemplateBody, TemplateId, TemplateName};
pub use error::{
    TemplateBodyError, TemplateError, TemplateNameError,
    TemplateRepositoryError,
};
pub use raw::RawTemplate;
pub use repository::{ReadRepository, Repository, WriteRepository};
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
            template_dir.join("aggregate.rs"),
            template_dir.join("error.rs"),
            template_dir.join("raw.rs"),
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
}
