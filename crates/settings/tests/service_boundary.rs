//! Public boundary DTO tests for settings service inputs.

use std::path::PathBuf;

use traces_fs::{DirPath, FilePath};
use traces_settings::{
    CacheLocation, CacheRoot, CandidatePath, ConfigBuilderOptions,
    DiscoveryOptions, DiscoveryOutcome, GlobalCacheLocation, Service,
    SettingsError, SettingsService, TrustMode,
};

#[test]
fn constructs_discovery_options_through_public_api() {
    let options = DiscoveryOptions {
        anchor: PathBuf::from("/vault"),
        config_file: Some(PathBuf::from("/vault/traces.toml")),
        vault_dir: Some(PathBuf::from("/vault")),
        suppress_global: true,
    };

    assert_eq!(options.anchor, PathBuf::from("/vault"));
    assert_eq!(options.config_file, Some(PathBuf::from("/vault/traces.toml")));
    assert_eq!(options.vault_dir, Some(PathBuf::from("/vault")));
    assert!(options.suppress_global);
}

#[test]
fn constructs_config_builder_options_through_public_api() {
    let options = ConfigBuilderOptions {
        trust_mode: TrustMode::AcceptAll,
        auto_confirm: true,
    };

    assert!(matches!(options.trust_mode, TrustMode::AcceptAll));
    assert!(options.auto_confirm);
}

#[test]
fn constructs_discovery_outcome_through_public_api() {
    let root = tempfile::tempdir().expect("temp dir");
    let config_path = root.path().join("traces.toml");
    std::fs::write(&config_path, "").expect("write config");

    let candidate = CandidatePath::new(
        DirPath::try_new(root.path().to_path_buf()).expect("valid dir"),
        FilePath::try_new(config_path).expect("valid file"),
    );
    let cache_root = CacheRoot::new(
        CacheLocation::Global(GlobalCacheLocation::PlatformUserCache),
        root.path().join("cache"),
    );

    let outcome = DiscoveryOutcome {
        vault: Box::from([candidate.clone()]),
        global: Box::from([]),
        cache_root: cache_root.clone(),
    };

    assert_eq!(outcome.vault.as_ref(), [candidate].as_ref());
    assert!(outcome.global.is_empty());
    assert_eq!(outcome.cache_root, cache_root);
}

#[test]
fn settings_service_trait_uses_owned_options_and_borrowed_candidate_paths() {
    let service = Service;
    let discovery_options = DiscoveryOptions {
        anchor: PathBuf::from("/vault"),
        config_file: None,
        vault_dir: None,
        suppress_global: false,
    };
    let builder_options = ConfigBuilderOptions {
        trust_mode: TrustMode::Verify,
        auto_confirm: false,
    };

    assert!(matches!(
        service.discover(discovery_options),
        Err(SettingsError::PipelineNotImplemented)
    ));
    assert!(matches!(
        service.build_config(&[], &[], builder_options),
        Err(SettingsError::PipelineNotImplemented)
    ));
}
