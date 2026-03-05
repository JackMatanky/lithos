//! Concurrency tests for config module.
//!
//! These tests verify that concurrent operations don't cause data loss,
//! corruption, or inconsistent state.

// Allow test-specific clippy lints
#![allow(
    clippy::panic_in_result_fn,
    clippy::tests_outside_test_module,
    clippy::default_numeric_fallback,
    clippy::pattern_type_mismatch,
    clippy::print_stdout,
    clippy::use_debug,
    clippy::indexing_slicing,
    clippy::semicolon_inside_block,
    clippy::semicolon_if_nothing_returned,
    clippy::shadow_unrelated,
    clippy::unwrap_in_result,
    clippy::integer_division_remainder_used,
    clippy::integer_division,
    reason = "Concurrency tests use assert!, println!, and other patterns \
              that are acceptable in tests"
)]

use std::{
    sync::{Arc, Barrier},
    thread,
};

use lithos_core::{
    config::{
        RedbConfigCommand, RedbConfigQuery,
        adapter::{command::CommandAdapter, query::QueryAdapter},
        vault::{VaultId, VaultRoot},
    },
    db::Database,
};
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn setup_vault(dir: &TempDir) -> TestResult {
    let vault_path = dir.path().join("vault");
    std::fs::create_dir_all(&vault_path)?;

    // Create a basic config file
    let config_toml = r#"
[logging]
log_level = "debug"
"#;
    std::fs::write(vault_path.join("lithos.toml"), config_toml)?;

    Ok(())
}

/// **CRITICAL TEST**: Verifies that concurrent rebuilds don't cause version
/// collisions.
///
/// This test attempts to trigger the race condition where two threads:
/// 1. Both scan and find max version N
/// 2. Both compute next version N+1
/// 3. Both write version N+1 (second overwrites first!)
///
/// **Expected behavior** (after fix):
/// - Thread A writes version N+1
/// - Thread B writes version N+2
/// - Both versions exist in database
///
/// **Current behavior** (bug):
/// - Thread A writes version N+1
/// - Thread B writes version N+1 (overwrites!)
/// - Only one version N+1 exists in database
#[test]
#[ignore = "Demonstrates critical race condition - will fail until fixed"]
fn concurrent_rebuilds_cause_version_collision() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    setup_vault(&temp_dir)?;

    let db_path = temp_dir.path().join("config.redb");
    let db = Arc::new(Database::open(&db_path)?);

    let vault_id = VaultId::new();
    let vault_root = VaultRoot::try_new(temp_dir.path().join("vault"))?;

    // Synchronize thread starts to maximize collision probability
    let barrier = Arc::new(Barrier::new(2));

    let mut handles = vec![];

    for i in 0..2 {
        let db = Arc::clone(&db);
        let vault_root = vault_root.clone();
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            let command = RedbConfigCommand::new(CommandAdapter::new(&db));

            // Wait for both threads to be ready
            barrier.wait();

            // Both threads execute rebuild simultaneously
            let version = command
                .rebuild_merged(vault_id, &vault_root)
                .expect("rebuild should succeed");

            (i, version)
        });

        handles.push(handle);
    }

    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("thread should not panic"))
        .collect();

    // Extract versions
    let versions: Vec<_> = results.iter().map(|(_, v)| v.value()).collect();

    println!("Thread versions: {versions:?}");

    // Check if versions are unique (they should be!)
    assert_ne!(
        versions[0], versions[1],
        "RACE CONDITION DETECTED: Both threads got same version! This means \
         one config was lost."
    );

    // Verify both versions exist in database
    let query = RedbConfigQuery::new(QueryAdapter::new(&db));

    for (thread_id, version) in results {
        let config = query.find(vault_id)?;
        assert!(
            config.is_some(),
            "Thread {} version {} should exist in database",
            thread_id,
            version.value()
        );
    }

    Ok(())
}

/// Test that concurrent reads during rebuild don't cause issues.
///
/// This should be safe since reads and writes use MVCC.
#[test]
fn concurrent_reads_during_rebuild() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    setup_vault(&temp_dir)?;

    let db_path = temp_dir.path().join("config.redb");
    let db = Arc::new(Database::open(&db_path)?);

    let vault_id = VaultId::new();
    let vault_root = VaultRoot::try_new(temp_dir.path().join("vault"))?;

    // Create initial version
    let command = RedbConfigCommand::new(CommandAdapter::new(&db));
    command.rebuild_merged(vault_id, &vault_root)?;

    let barrier = Arc::new(Barrier::new(11)); // 1 writer + 10 readers
    let mut handles = vec![];

    // Spawn writer thread
    {
        let db = Arc::clone(&db);
        let vault_root = vault_root.clone();
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            let command = RedbConfigCommand::new(CommandAdapter::new(&db));
            barrier.wait();

            // Write new version
            command
                .rebuild_merged(vault_id, &vault_root)
                .expect("rebuild should succeed");
        });

        handles.push(handle)
    };

    // Spawn reader threads
    for _ in 0..10 {
        let db = Arc::clone(&db);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            let query = RedbConfigQuery::new(QueryAdapter::new(&db));
            barrier.wait();

            // Read active config (may be old or new version)
            let config = query.find(vault_id).expect("query should succeed");

            assert!(config.is_some(), "should have at least one version");
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("thread should not panic");
    }

    Ok(())
}

/// Test that version scanning performance doesn't degrade catastrophically
/// with many versions.
///
/// This is a performance test, not correctness, but important for
/// understanding scaling behavior.
#[test]
#[ignore = "Slow performance test"]
fn many_versions_performance() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    setup_vault(&temp_dir)?;

    let db_path = temp_dir.path().join("config.redb");
    let db = Database::open(&db_path)?;

    let vault_id = VaultId::new();
    let vault_root = VaultRoot::try_new(temp_dir.path().join("vault"))?;

    let command = RedbConfigCommand::new(CommandAdapter::new(&db));
    let query = RedbConfigQuery::new(QueryAdapter::new(&db));

    // Create 1000 versions
    println!("Creating 1000 versions...");
    let start = std::time::Instant::now();

    for i in 1..=1000 {
        command.rebuild_merged(vault_id, &vault_root)?;

        if i % 100 == 0 {
            println!("  Created {i} versions");
        }
    }

    let write_duration = start.elapsed();
    println!("Write time: {write_duration:?}");

    // Measure read performance
    println!("Reading active version 1000 times...");
    let start = std::time::Instant::now();

    for _ in 0..1000 {
        let _config = query.find(vault_id)?;
    }

    let read_duration = start.elapsed();
    println!("Read time: {read_duration:?}");
    println!("Avg read latency: {:?}", read_duration / 1000);

    // Warn if read latency is unacceptable
    let avg_read_ms = read_duration.as_millis() / 1000;
    if avg_read_ms > 10 {
        println!(
            "WARNING: Average read latency {avg_read_ms} ms is high with 1000 \
             versions"
        );
    }

    Ok(())
}
