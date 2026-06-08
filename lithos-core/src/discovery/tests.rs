use std::{os::unix::fs::symlink, path::Path};

use tempfile::TempDir;

use super::{
    engine::{DiscoveryEngine, DiscoveryInput},
    policy::{DiscoveryPolicy, VaultSourceType},
    probe::DiscoveryProbe,
};
use crate::fs::format::StructuredFileFormat;

fn engine() -> DiscoveryEngine {
    DiscoveryEngine::new(DiscoveryPolicy::default())
}

fn write_marker(root: &Path, relative: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create marker parent dir");
    }
    std::fs::write(&path, "").expect("write marker file");
}

#[test]
fn ascending_walk_resolves_marker_across_multiple_levels() {
    let root = TempDir::new().expect("root");
    write_marker(root.path(), "lithos.toml");

    let deep = root.path().join("a").join("b").join("c");
    std::fs::create_dir_all(&deep).expect("create deep dir");

    let result = engine()
        .find_vault(&DiscoveryInput {
            flag_path: None,
            env_path: None,
            cwd: &deep,
            ceiling_dirs_raw: None,
        })
        .expect("resolution succeeds");

    assert_eq!(result.source, Some(VaultSourceType::AscendingWalk));
    assert_eq!(
        result.root,
        Some(root.path().canonicalize().expect("canonical root"))
    );
    assert!(result.marker.is_some());
}

#[test]
fn ceiling_stops_above_marker() {
    let root = TempDir::new().expect("root");
    write_marker(root.path(), "lithos.toml");

    let stop = root.path().join("boundary");
    let cwd = stop.join("deep");
    std::fs::create_dir_all(&cwd).expect("create cwd");

    let result = engine()
        .find_vault(&DiscoveryInput {
            flag_path: None,
            env_path: None,
            cwd: &cwd,
            ceiling_dirs_raw: Some(stop.as_os_str()),
        })
        .expect("resolution succeeds");

    assert_eq!(result.root, None);
    assert_eq!(result.marker, None);
}

#[test]
fn symlink_cycle_probe_does_not_panic() {
    let root = TempDir::new().expect("root");
    let link_path = root.path().join("self_link");
    symlink(root.path(), &link_path).expect("create self-referential symlink");

    let probe = super::probe::VaultRootProbe;
    let result = probe.probe(root.path());

    assert!(result.is_ok(), "Probe should not panic on symlinks");
}

#[test]
fn explicit_flag_takes_precedence_over_env_and_walk() {
    let flag_root = TempDir::new().expect("flag_root");
    write_marker(flag_root.path(), "lithos.toml");
    let env_root = TempDir::new().expect("env_root");
    write_marker(env_root.path(), "lithos.toml");
    let walk_root = TempDir::new().expect("walk_root");
    write_marker(walk_root.path(), "lithos.toml");

    let result = engine()
        .find_vault(&DiscoveryInput {
            flag_path: Some(flag_root.path()),
            env_path: Some(env_root.path()),
            cwd: walk_root.path(),
            ceiling_dirs_raw: None,
        })
        .expect("resolution succeeds");

    assert_eq!(result.source, Some(VaultSourceType::ExplicitFlag));
    assert_eq!(
        result.root,
        Some(flag_root.path().canonicalize().expect("canonical flag"))
    );
}

#[test]
fn env_var_takes_precedence_over_ascending_walk() {
    let env_root = TempDir::new().expect("env_root");
    write_marker(env_root.path(), "lithos.toml");
    let walk_root = TempDir::new().expect("walk_root");
    write_marker(walk_root.path(), "lithos.toml");

    let result = engine()
        .find_vault(&DiscoveryInput {
            flag_path: None,
            env_path: Some(env_root.path()),
            cwd: walk_root.path(),
            ceiling_dirs_raw: None,
        })
        .expect("resolution succeeds");

    assert_eq!(result.source, Some(VaultSourceType::EnvVar));
    assert_eq!(
        result.root,
        Some(env_root.path().canonicalize().expect("canonical env"))
    );
}

#[test]
fn multiple_markers_return_winner_and_alternatives() {
    let root = TempDir::new().expect("root");
    write_marker(root.path(), "lithos.toml");
    write_marker(root.path(), "lithos.json");

    let result = engine()
        .find_vault(&DiscoveryInput {
            flag_path: None,
            env_path: None,
            cwd: root.path(),
            ceiling_dirs_raw: None,
        })
        .expect("resolution succeeds");

    let marker = result.marker.expect("winner should exist");
    assert_eq!(marker.format, StructuredFileFormat::Toml);

    assert!(!result.alternatives.is_empty(), "should have alternatives");
    let winner_path = marker.path.clone();
    assert!(
        !result.alternatives.iter().any(|alt| alt.path == winner_path),
        "alternatives should not include the winner"
    );
    assert!(
        result
            .alternatives
            .iter()
            .any(|alt| alt.format == StructuredFileFormat::Json),
        "JSON should be in alternatives"
    );
}
