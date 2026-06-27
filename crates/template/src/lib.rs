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
//!   [`TemplateRepositoryError`], [`TemplateArtifactError`],
//!   [`TemplateEngineError`] — domain, repository, artifact, and engine errors
//!
//! Service and engine surface:
//! - [`TemplateService`] — use-case orchestrator for ingestion and rendering
//! - [`CreateInput`], [`CreateTemplateOutcome`] — render-to-commit request and
//!   outcome
//! - [`ProcessSummary`] — counts returned by [`TemplateService::process_all`]
//! - [`TemplateEngine`], [`MiniJinjaEngine`] — rendering port and adapter
//! - [`RenderedTemplate`] — load-bearing rendered-output newtype
//! - [`ReadRepository`], [`WriteRepository`], [`Repository`] — persistence
//!   ports
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
pub(crate) mod service;
pub(crate) mod storage;
pub(crate) mod views;

pub use aggregate::{Template, TemplateBody, TemplateId, TemplateName};
pub use engine::{
    RenderedTemplate, TemplateEngine, TemplateEngineError,
    mini_jinja::MiniJinjaEngine,
};
pub use error::{
    TemplateArtifactError, TemplateBodyError, TemplateError, TemplateNameError,
    TemplateRepositoryError,
};
pub use raw::RawTemplate;
pub use service::{
    CreateInput, CreateTemplateOutcome, ProcessSummary, TemplateService,
};
pub use storage::{ReadRepository, Repository, WriteRepository};
pub use views::RawTemplateView;

// ============================================================================
// Policy enforcement (tracer bullet 17)
// ============================================================================

#[cfg(test)]
#[expect(clippy::panic, reason = "tests use panic for assertions")]
mod policy {
    use std::path::{Path, PathBuf};

    use walkdir::WalkDir;

    /// Files allowed to reference the rendering engine.
    ///
    /// `engine/mini_jinja.rs` is the adapter (it *must* import the crate).
    /// The port (`engine.rs`) uses a boxed `dyn Error` source and never imports
    /// the rendering engine crate directly.
    const ENGINE_BOUNDARY_FILES: [&str; 1] = ["engine/mini_jinja.rs"];

    /// The forbidden crate name, assembled at runtime so the literal never
    /// appears in this file and trips the policy test against itself.
    fn engine_crate() -> String {
        ["mini", "j", "inja"].concat()
    }

    fn src_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
    }

    /// Discovers every `*.rs` file under `src/`, returning each as a
    /// `(path, relative-string)` pair so callers can apply boundary
    /// exclusions. Replaces the previously hand-maintained file list so new
    /// source files are covered automatically.
    fn rust_source_files(src: &Path) -> Vec<(PathBuf, String)> {
        WalkDir::new(src)
            .into_iter()
            .map(|entry| {
                entry.unwrap_or_else(|e| panic!("walk {}: {e}", src.display()))
            })
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry.path().extension().is_some_and(|ext| ext == "rs")
            })
            .map(|entry| {
                let rel = entry
                    .path()
                    .strip_prefix(src)
                    .unwrap_or_else(|e| {
                        panic!("strip {}: {e}", entry.path().display())
                    })
                    .to_string_lossy()
                    .replace('\\', "/");
                (entry.path().to_path_buf(), rel)
            })
            .collect()
    }

    fn forbidden_patterns(engine_crate: &str) -> [String; 3] {
        [
            format!("use {engine_crate}"),
            format!("extern crate {engine_crate}"),
            format!("::{engine_crate}::"),
        ]
    }

    /// Verifies that no import/use of the rendering-engine crate appears in any
    /// template-context source file outside the engine boundary.
    ///
    /// This is a policy invariant: Jinja syntax validation is the engine's
    /// responsibility, not the domain's. The file set is discovered by
    /// walking `src/`, so a new `*.rs` file cannot silently slip a forbidden
    /// import past the check.
    #[test]
    fn no_rendering_engine_import_in_template_context() {
        let src = src_dir();
        let engine_crate = engine_crate();
        let forbidden = forbidden_patterns(&engine_crate);

        for (path, rel) in rust_source_files(&src) {
            if ENGINE_BOUNDARY_FILES.contains(&rel.as_str()) {
                continue;
            }
            let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!("Could not read {}: {e}", path.display())
            });
            for pattern in &forbidden {
                assert!(
                    !content.contains(pattern.as_str()),
                    "Found '{engine_crate}' import in {rel}. Template domain \
                     types must not import the rendering engine crate."
                );
            }
        }
    }

    /// Verifies that rendering-engine imports stay confined to the engine
    /// boundary and that the adapter actually imports the crate.
    #[test]
    fn rendering_engine_imports_are_confined_to_engine_adapter() {
        let src = src_dir();
        let engine_crate = engine_crate();
        let forbidden = forbidden_patterns(&engine_crate);

        for (path, rel) in rust_source_files(&src) {
            if ENGINE_BOUNDARY_FILES.contains(&rel.as_str()) {
                continue;
            }
            let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!("Could not read {}: {e}", path.display())
            });
            for pattern in &forbidden {
                assert!(
                    !content.contains(pattern.as_str()),
                    "Found '{engine_crate}' import in {rel}. Rendering-engine \
                     imports must stay confined to the engine adapter."
                );
            }
        }

        let engine_adapter = src.join("engine").join("mini_jinja.rs");
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
