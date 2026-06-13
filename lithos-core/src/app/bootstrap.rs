//! Bootstrap orchestration seams for runtime context acquisition.

use crate::discovery::{
    context::{DiscoveryContext, DiscoveryEnv, DiscoveryFlags},
    error::DiscoveryError,
};

/// Application-owned bootstrap orchestration entry point.
#[derive(Debug, Default)]
#[allow(dead_code, reason = "Contract slice; full orchestration lands later")]
pub(crate) struct Bootstrapper;

impl Bootstrapper {
    /// Builds Discovery's input contract from app-owned runtime sources.
    #[allow(
        dead_code,
        reason = "Contract slice; full orchestration lands later"
    )]
    pub(crate) fn discovery_context<'a>(
        flags: DiscoveryFlags,
        env: DiscoveryEnv<'a>,
        anchor: &std::path::Path,
    ) -> Result<DiscoveryContext<'a>, DiscoveryError> {
        DiscoveryContext::new(flags, env, anchor)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;
    use crate::fs::path::{DirPath, FilePath};

    mod fixtures {
        use super::*;

        pub(super) struct BootstrapInputs {
            pub(super) cwd: tempfile::TempDir,
            pub(super) cli_vault: tempfile::TempDir,
            pub(super) env_vault: tempfile::TempDir,
            pub(super) cli_config: std::path::PathBuf,
            pub(super) env_config: std::path::PathBuf,
            pub(super) ceilings: &'static std::ffi::OsStr,
        }

        impl BootstrapInputs {
            pub(super) fn new() -> Self {
                let cwd = tempfile::tempdir().expect("cwd dir");
                let cli_vault = tempfile::tempdir().expect("cli vault dir");
                let env_vault = tempfile::tempdir().expect("env vault dir");
                let cli_config = cli_vault.path().join("lithos.toml");
                let env_config = env_vault.path().join("lithos.toml");
                std::fs::write(&cli_config, "").expect("write cli config");
                std::fs::write(&env_config, "").expect("write env config");
                let ceilings = OsStr::new("/work:/home");
                Self {
                    cwd,
                    cli_vault,
                    env_vault,
                    cli_config,
                    env_config,
                    ceilings,
                }
            }

            pub(super) fn build_context(
                &self,
            ) -> Result<DiscoveryContext<'_>, DiscoveryError> {
                let flags = DiscoveryFlags::new(
                    Some(self.cli_config.as_path()),
                    Some(self.cli_vault.path()),
                    true,
                )
                .expect("valid flags");
                let env = DiscoveryEnv::new(
                    Some(self.env_config.as_path()),
                    Some(self.env_vault.path()),
                    Some(self.ceilings),
                )
                .expect("valid env");
                Bootstrapper::discovery_context(flags, env, self.cwd.path())
            }
        }
    }

    mod discovery_context {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn returns_context_with_anchor_matching_cwd() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.anchor().as_path(),
                    inputs.cwd.path(),
                    "anchor should match the injected cwd"
                );
            }

            #[test]
            fn returns_context_with_flag_config_file() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.flags().config_file().map(FilePath::as_path),
                    Some(inputs.cli_config.as_path()),
                    "flag config_file should match the injected cli config \
                     path"
                );
            }

            #[test]
            fn returns_context_with_flag_vault_dir() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.flags().vault_dir().map(DirPath::as_path),
                    Some(inputs.cli_vault.path()),
                    "flag vault_dir should match the injected cli vault path"
                );
            }

            #[test]
            fn returns_context_with_suppress_global_set() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert!(
                    context.flags().suppress_global(),
                    "suppress_global should be set from the injected flags"
                );
            }

            #[test]
            fn returns_context_with_env_config_file() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.env().config_file().map(FilePath::as_path),
                    Some(inputs.env_config.as_path()),
                    "env config_file should match the injected env config path"
                );
            }

            #[test]
            fn returns_context_with_env_vault_dir() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.env().vault_dir().map(DirPath::as_path),
                    Some(inputs.env_vault.path()),
                    "env vault_dir should match the injected env vault path"
                );
            }

            #[test]
            fn returns_context_with_ceiling_dirs_raw() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.env().ceiling_dirs_raw(),
                    Some(inputs.ceilings),
                    "ceiling_dirs_raw should match the injected ceiling dirs"
                );
            }
        }
    }
}
