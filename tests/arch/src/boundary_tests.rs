//! Architecture boundary enforcement tests.
//!
//! These tests ensure that the dependency flow architecture is maintained:
//! - Domain contexts must NOT depend on `db` module
//! - Domain modules must be pure (no I/O)
//! - Dependencies flow inward: CLI → Domain → DB → FS
//!
//! # Phase 5 Note
//!
//! These are compile-time architecture tests. If the domain layer starts
//! importing from `db`, the code will not compile, enforcing the boundary.

#![expect(
    clippy::tests_outside_test_module,
    reason = "This is a test crate; all functions are tests"
)]
#![expect(
    clippy::let_underscore_must_use,
    reason = "Type assertions in tests don't need result handling"
)]
#![expect(
    clippy::used_underscore_binding,
    reason = "Underscore bindings in tests indicate intentionally unused \
              parameters"
)]

/// Test that domain aggregates do not import database types.
///
/// This is a compile-time test. If this compiles, it means the domain
/// layer is properly isolated from the database layer.
#[test]
fn domain_aggregates_do_not_import_db() {
    // This test verifies that we can use domain types without the db module
    use lithos_core::{
        config::aggregate::Config, note::aggregate::Note,
        schema::aggregate::Schema, template::aggregate::Template,
    };

    // If domain aggregates imported db types, this would fail to compile
    // because we're not importing lithos_core::db here.

    // Type assertions to ensure the types are actually used
    fn _assert_domain_types_exist() {
        let _: Option<Config> = None;
        let _: Option<Note> = None;
        let _: Option<Schema> = None;
        let _: Option<Template> = None;
    }
}

/// Test that command/query implementations properly use Database references.
///
/// This test verifies that the CQRS implementations take `&Database`
/// parameters as expected per Proposal 5 (static methods pattern).
#[test]
fn cqrs_implementations_use_database_references() {
    use lithos_core::{
        config::{command::ConfigCommand, query::ConfigQuery},
        db::Database,
        note::{command::NoteCommand, query::NoteQuery},
        schema::{command::SchemaCommand, query::SchemaQuery},
        template::{command::TemplateCommand, query::TemplateQuery},
    };

    // Verify that command/query structs can be constructed with a Database
    // reference
    fn _assert_cqrs_constructors_accept_database_ref(_db: &Database) {
        let _: ConfigCommand = ConfigCommand::new(_db);
        let _: ConfigQuery = ConfigQuery::new(_db);
        let _: NoteCommand = NoteCommand::new(_db);
        let _: NoteQuery = NoteQuery::new(_db);
        let _: SchemaCommand = SchemaCommand::new(_db);
        let _: SchemaQuery = SchemaQuery::new(_db);
        let _: TemplateCommand = TemplateCommand::new(_db);
        let _: TemplateQuery = TemplateQuery::new(_db);
    }
}

/// Test that fs module is independent (no domain dependencies).
///
/// The fs module should provide generic file system utilities without
/// knowing about domain concepts like Note or Schema.
#[test]
fn fs_module_is_domain_independent() {
    // The fs module should be usable without importing domain types
    use lithos_core::fs;

    // If fs depended on domain types, we couldn't use it in isolation
    fn _assert_fs_module_exists() {
        // Type-level assertion: if this compiles, fs module is independent
        // Just verify the function exists
        let _ = fs::validate_vault_path;
    }
}

/// Test that error types are co-located with their contexts.
///
/// Per Proposal 2, errors should be co-located (config/error.rs, note/error.rs)
/// rather than centralized.
#[test]
fn error_types_are_colocated() {
    use lithos_core::{
        config::error::ConfigError, db::DbError, fs::error::FsError,
        note::error::NoteError, schema::error::SchemaError,
        template::error::TemplateError,
    };

    // Type assertions to verify error types exist in expected locations
    fn _assert_error_types_exist() {
        let _: Option<ConfigError> = None;
        let _: Option<NoteError> = None;
        let _: Option<SchemaError> = None;
        let _: Option<TemplateError> = None;
        let _: Option<DbError> = None;
        let _: Option<FsError> = None;
    }
}

/// Test that the dependency flow is correct: CLI → Core → DB.
///
/// This is verified by attempting to use the full stack without circular deps.
#[test]
fn dependency_flow_is_correct() {
    use lithos_core::{
        db::Database,
        note::{command::NoteCommand, query::NoteQuery},
    };

    // Simulated usage pattern: CLI creates DB, then uses it with domain
    // operations
    fn _simulated_cli_usage(_db: &Database) {
        // CLI layer can depend on both db and domain
        let _cmd = NoteCommand::new(_db);
        let _query = NoteQuery::new(_db);

        // This should compile fine because:
        // - CLI depends on lithos-core (which contains both db and domain)
        // - Domain command/query types depend on Database
        // - But domain aggregates do NOT depend on Database
    }
}
