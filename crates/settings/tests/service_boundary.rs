//! Public boundary DTO tests for settings service inputs.

use std::path::PathBuf;

use traces_fs::{DirPath, FilePath};
use traces_settings::{
    CandidatePath, ConfigBuilderOptions, DiscoveryOptions, DiscoveryOutcome,
    DiscoveryReport, Service, SettingsError, SettingsService, TrustMode,
};

mod discovery_options {
    use super::*;

    #[test]
    fn constructs_discovery_options_through_public_api() {
        let options = DiscoveryOptions::new(
            PathBuf::from("/vault"),
            Some(PathBuf::from("/vault/traces.toml")),
            Some(PathBuf::from("/vault")),
            true,
        );

        assert_eq!(options.anchor(), &PathBuf::from("/vault"));
        assert_eq!(
            options.config_file(),
            Some(&PathBuf::from("/vault/traces.toml"))
        );
        assert_eq!(options.vault_dir(), Some(&PathBuf::from("/vault")));
        assert!(options.suppress_global());
    }
}

mod config_builder_options {
    use super::*;

    #[test]
    fn constructs_config_builder_options_through_public_api() {
        let options = ConfigBuilderOptions::new(TrustMode::AcceptAll, true);

        assert!(matches!(options.trust_mode(), TrustMode::AcceptAll));
        assert!(options.auto_confirm());
    }
}

mod discovery_outcome {
    use super::*;

    #[test]
    fn constructs_through_public_api() {
        let root = tempfile::tempdir().expect("temp dir");
        let config_path = root.path().join("traces.toml");
        std::fs::write(&config_path, "").expect("write config");

        let candidate = CandidatePath::new(
            DirPath::try_new(root.path().to_path_buf()).expect("valid dir"),
            FilePath::try_new(config_path).expect("valid file"),
        );
        let outcome = DiscoveryOutcome::new(
            Box::from([candidate.clone()]),
            Box::from([]),
            DiscoveryReport::default(),
        );

        assert_eq!(outcome.vault(), [candidate].as_ref());
        assert!(outcome.global().is_empty());
        assert_eq!(outcome.report(), &DiscoveryReport::default());
    }
}

mod settings_service {
    use super::*;

    #[test]
    fn discover_returns_outcome_for_valid_options() {
        let service = Service;
        let root = tempfile::tempdir().expect("temp dir");
        let config = root.path().join("traces.toml");
        std::fs::write(&config, "").expect("config");

        let outcome = service
            .discover(DiscoveryOptions::new(
                root.path().to_path_buf(),
                None,
                None,
                true,
            ))
            .expect("discover");

        assert_eq!(
            outcome.vault().first().map(|candidate| candidate.path().as_path()),
            Some(config.as_path())
        );
        assert!(outcome.global().is_empty());
    }

    #[test]
    fn settings_service_trait_uses_borrowed_candidate_paths() {
        let service = Service;
        let discovery_options =
            DiscoveryOptions::new(PathBuf::from("/vault"), None, None, false);
        let builder_options =
            ConfigBuilderOptions::new(TrustMode::Verify, false);

        assert!(matches!(
            service.discover(discovery_options),
            Err(SettingsError::Discovery(_))
        ));
        assert!(matches!(
            service.build_config(&[], &[], builder_options),
            Err(SettingsError::PipelineNotImplemented)
        ));
    }
}
