use super::{
    global,
    global::Global,
    types::{Frontmatter, Logging, Schema, Template},
    vault,
    vault::Vault,
};

/// Merged configuration result from global and vault configurations.
///
/// This struct represents the final merged configuration after applying
/// business rules for precedence (vault overrides global). The configuration
/// is immutable once created and represents the complete runtime configuration
/// for a vault operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Config {
    /// Merged frontmatter configuration.
    pub frontmatter: Frontmatter,
    /// Global filesystem configuration.
    pub global_filesystem: global::Filesystem,
    /// Merged logging configuration.
    pub logging: Logging,
    /// Vault filesystem configuration.
    pub vault_filesystem: vault::Filesystem,
    /// Vault metadata with versioning and naming.
    pub vault_metadata: vault::Metadata,
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            vault_metadata: vault::Metadata::default(),
            global_filesystem: global::Filesystem::default(),
            vault_filesystem: vault::Filesystem::default(),
            frontmatter: Frontmatter::default(),
            logging: Logging::default(),
        }
    }
}

impl Config {
    /// Build a new Config by combining optional Global and Vault configurations with business rules.
    ///
    /// # Business Rules
    /// - Filesystems are kept separate (global library vs vault-specific).
    /// - Frontmatter and logging merge with vault precedence over global.
    /// - Vault path is required and used to set vault metadata defaults.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if `vault_path` is empty.
    /// Returns `ConfigError::InvalidEnumValue` if `log_level` is invalid.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{Config, GlobalConfig, VaultConfig};
    /// let global = GlobalConfig::default();
    /// let vault = VaultConfig::default();
    ///
    /// let config = Config::build(Some(&global), "/vault", vault).unwrap();
    /// assert_eq!(config.vault_metadata.vault_path, "/vault");
    /// ```
    #[inline]
    pub fn build(
        global: Option<&Global>,
        vault_path: &str,
        vault: Vault,
    ) -> Result<Self, crate::ConfigError> {
        // Step 1: Pre-validate required Vault Path
        vault::Metadata::validate_vault_path(vault_path)?;

        // Step 2: Set vault metadata defaults
        let vault_metadata = vault::Metadata::new(vault_path.to_owned());

        // Step 3: Set filesystem configurations with defaults applied
        let global_filesystem = global
            .map(|g| global::Filesystem {
                schema: Schema {
                    schemas_dir: if g.filesystem.schema.schemas_dir.is_empty() {
                        "schemas".to_owned()
                    } else {
                        g.filesystem.schema.schemas_dir.clone()
                    },
                    property_bank_filename: if g
                        .filesystem
                        .schema
                        .property_bank_filename
                        .is_empty()
                    {
                        "property_bank.json".to_owned()
                    } else {
                        g.filesystem.schema.property_bank_filename.clone()
                    },
                },
                template: Template {
                    templates_dir: if g
                        .filesystem
                        .template
                        .templates_dir
                        .is_empty()
                    {
                        "templates".to_owned()
                    } else {
                        g.filesystem.template.templates_dir.clone()
                    },
                },
            })
            .unwrap_or_default();

        let vault_filesystem = vault::Filesystem {
            schema: Schema {
                schemas_dir: if vault.filesystem.schema.schemas_dir.is_empty() {
                    "schemas".to_owned()
                } else {
                    vault.filesystem.schema.schemas_dir
                },
                property_bank_filename: if vault
                    .filesystem
                    .schema
                    .property_bank_filename
                    .is_empty()
                {
                    "property_bank.json".to_owned()
                } else {
                    vault.filesystem.schema.property_bank_filename
                },
            },
            template: Template {
                templates_dir: if vault
                    .filesystem
                    .template
                    .templates_dir
                    .is_empty()
                {
                    "templates".to_owned()
                } else {
                    vault.filesystem.template.templates_dir
                },
            },
            cache_dir: if vault.filesystem.cache_dir.is_empty() {
                ".cache".to_owned()
            } else {
                vault.filesystem.cache_dir
            },
        };

        // Step 4: Merge other values with precedence
        let frontmatter = Config::merge_frontmatter(
            global.map(|g| &g.frontmatter),
            vault.frontmatter.as_ref(),
        );

        let logging = Config::merge_logging(
            global.map(|g| &g.logging),
            vault.logging.as_ref(),
        );

        // Step 5: Construct the final strictly-validated aggregate
        let config = Self {
            frontmatter,
            global_filesystem,
            logging,
            vault_filesystem,
            vault_metadata,
        };

        // Step 5: Final invariant check
        config.validate()?;

        Ok(config)
    }

    /// Merge frontmatter configurations applying defaults where needed.
    pub(crate) fn merge_frontmatter(
        global: Option<&Frontmatter>,
        vault: Option<&Frontmatter>,
    ) -> Frontmatter {
        let defaults = Frontmatter::default();

        Frontmatter {
            alias_key: choose_value(
                vault.map_or("", |v| &v.alias_key),
                global.map_or("", |g| &g.alias_key),
                &defaults.alias_key,
            ),
            date_created_key: choose_value(
                vault.map_or("", |v| &v.date_created_key),
                global.map_or("", |g| &g.date_created_key),
                &defaults.date_created_key,
            ),
            date_modified_key: choose_value(
                vault.map_or("", |v| &v.date_modified_key),
                global.map_or("", |g| &g.date_modified_key),
                &defaults.date_modified_key,
            ),
            file_class_key: choose_value(
                vault.map_or("", |v| &v.file_class_key),
                global.map_or("", |g| &g.file_class_key),
                &defaults.file_class_key,
            ),
            title_key: choose_value(
                vault.map_or("", |v| &v.title_key),
                global.map_or("", |g| &g.title_key),
                &defaults.title_key,
            ),
        }
    }

    /// Merge logging configurations applying defaults where needed.
    pub(crate) fn merge_logging(
        global: Option<&Logging>,
        vault: Option<&Logging>,
    ) -> Logging {
        let log_level = choose_value(
            vault.map_or("", |v| &v.log_level),
            global.map_or("", |g| &g.log_level),
            "info",
        );
        Logging {
            log_level,
        }
    }

    /// Validate configuration against critical business rules.
    ///
    /// # Validation Rules
    /// - All filesystem fields must be non-empty.
    /// - All frontmatter fields must be non-empty.
    /// - `log_level` must be one of: debug, info, warn, error.
    ///
    /// # Note
    /// This method is provided for post-construction validation if needed.
    /// The `build()` method already performs validation during construction.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if any required field is empty.
    /// Returns `ConfigError::InvalidEnumValue` if `log_level` is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lithos_domain::{Config, GlobalConfig, VaultConfig};
    /// // Create a valid config via build
    /// let global = GlobalConfig::default();
    /// let vault = VaultConfig::default();
    ///
    /// let config = Config::build(Some(&global), "/vault", vault).unwrap();
    /// assert!(config.validate().is_ok());
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        // Validate all component parts
        self.vault_metadata.validate()?;
        self.global_filesystem.validate()?;
        self.vault_filesystem.validate()?;
        self.frontmatter.validate()?;
        self.logging.validate()?;

        Ok(())
    }
}

/// Choose value with precedence: vault > global > default.
#[inline]
#[must_use]
fn choose_value(vault: &str, global: &str, default: &str) -> String {
    if !vault.is_empty() {
        vault.to_owned()
    } else if !global.is_empty() {
        global.to_owned()
    } else {
        default.to_owned()
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "Test safety boundary - panic is acceptable in test code for exhaustive match failures"
)]
mod tests {

    use super::*;

    #[cfg(test)]
    mod proptests {
        use proptest::prelude::*;

        use super::*;

        proptest! {
            #[test]
            fn merge_handles_various_path_lengths(
                vault_path in "[a-zA-Z0-9/_-]{1,200}",
                templates_dir in "[a-zA-Z0-9/_-]{0,100}"
            ) {
                // GIVEN a global config and generated vault path/template overrides
                let global = sample_global_config();
                let vault_config = Vault {
                    filesystem: vault::Filesystem {
                        schema: Schema::default(),
                        template: Template {
                            templates_dir: templates_dir.clone(),
                        },
                        cache_dir: ".cache".to_owned(),
                    },
                    frontmatter: None,
                    logging: None,
                };

                // WHEN building a config from the generated inputs
                let result = Config::build(Some(&global), &vault_path, vault_config);

                // THEN empty paths fail and valid paths preserve metadata
                if vault_path.is_empty() {
                    prop_assert!(result.is_err(), "Empty vault_path should fail");
                } else {
                    prop_assert!(result.is_ok(), "Valid paths should merge successfully");
                    if let Ok(config) = result {
                        prop_assert_eq!(config.vault_metadata.vault_path, vault_path);
                    }
                }
            }
        }
    }

    mod merge {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test expects merge to succeed, unwrap is appropriate for test clarity"
        )]
        fn vault_values_take_precedence_over_global() {
            // GIVEN a global config with default settings and a vault config with custom overrides
            let global = sample_global_config();
            let vault = sample_vault_config();

            // WHEN merging vault and global configs
            let merged = Config::build(Some(&global), "/vault", vault)
                .expect("Config build should succeed with valid sample data");

            // THEN vault values override global defaults
            assert_eq!(
                merged.vault_filesystem.template.templates_dir,
                "custom_templates",
                "Vault filesystem should have custom templates"
            );
            assert_eq!(
                merged.logging.log_level, "debug",
                "Log level should override global"
            );
            assert_eq!(
                merged.frontmatter.file_class_key, "type",
                "File class key should override global"
            );
            assert_eq!(
                merged.frontmatter.date_created_key, "created",
                "Date created key should override global"
            );
        }

        #[test]
        fn falls_back_to_defaults_when_inputs_are_empty() {
            // GIVEN configs with empty fields that should fall back to system defaults
            let global = Global {
                filesystem: global::Filesystem {
                    schema: Schema {
                        schemas_dir: String::new(), // Empty - should use default
                        property_bank_filename: String::new(),
                    },
                    template: Template {
                        templates_dir: String::new(),
                    },
                },
                frontmatter: Frontmatter {
                    alias_key: String::new(),
                    date_created_key: String::new(),
                    date_modified_key: String::new(),
                    file_class_key: String::new(),
                    title_key: String::new(),
                },
                logging: Logging {
                    log_level: String::new(), // Empty - should use default "info"
                },
                trusted_vaults: None,
            };

            let vault = Vault {
                filesystem: vault::Filesystem {
                    schema: Schema {
                        schemas_dir: String::new(),
                        property_bank_filename: String::new(),
                    },
                    template: Template {
                        templates_dir: String::new(),
                    },
                    cache_dir: String::new(),
                },
                frontmatter: None,
                logging: None,
            };

            // WHEN merging the configs
            let result = Config::build(Some(&global), "/vault", vault);

            // THEN merge should succeed and apply system defaults
            assert!(
                result.is_ok(),
                "Merge with empty values should succeed, got: {result:?}"
            );

            if let Ok(config) = result {
                // Verify defaults were applied to vault filesystem
                assert_eq!(
                    config.vault_filesystem.cache_dir, ".cache",
                    "Should fall back to default cache_dir"
                );
                assert_eq!(
                    config.vault_filesystem.template.templates_dir, "templates",
                    "Should fall back to default templates_dir"
                );
                assert_eq!(
                    config.vault_filesystem.schema.schemas_dir, "schemas",
                    "Should fall back to default schemas_dir"
                );
                assert_eq!(
                    config.vault_filesystem.schema.property_bank_filename,
                    "property_bank.json",
                    "Should fall back to default property_bank_filename"
                );
                assert_eq!(
                    config.logging.log_level, "info",
                    "Should fall back to default log_level"
                );
                assert_eq!(
                    config.frontmatter.file_class_key, "file_class",
                    "Should fall back to default file_class_key"
                );
                assert_eq!(
                    config.frontmatter.title_key, "title",
                    "Should fall back to default title_key"
                );
                assert_eq!(
                    config.frontmatter.alias_key, "aliases",
                    "Should fall back to default alias_key"
                );
            }
        }

        #[test]
        fn merge_is_idempotent() {
            // GIVEN the same global and vault configs
            let global = sample_global_config();
            let vault = sample_vault_config();

            // WHEN merging the same inputs multiple times
            let result1 = Config::build(Some(&global), "/vault", vault.clone());
            assert!(result1.is_ok(), "First merge should succeed");

            let result2 = Config::build(Some(&global), "/vault", vault);
            assert!(result2.is_ok(), "Second merge should succeed");

            // THEN results should be identical
            if let (Ok(merged1), Ok(merged2)) = (result1, result2) {
                assert_eq!(
                    merged1, merged2,
                    "Repeated merges with same input must yield identical output"
                );
            }
        }
    }

    mod validate {
        use rstest::rstest;

        use super::*;

        #[rstest]
        #[case::valid_config("/vault", "info", None)]
        #[case::empty_path("", "info", Some("vault_path"))]
        #[case::invalid_log_level("/vault", "invalid", Some("log_level"))]
        fn enforces_required_fields_and_enum_constraints(
            #[case] path: &str,
            #[case] level: &str,
            #[case] expected_error_field: Option<&str>,
        ) {
            // GIVEN a vault config with specific field values
            let global = sample_global_config();
            let vault = Vault {
                filesystem: vault::Filesystem {
                    schema: Schema::default(),
                    template: Template::default(),
                    cache_dir: ".cache".to_owned(),
                },
                frontmatter: None,
                logging: Some(Logging {
                    log_level: level.to_owned(),
                }),
            };

            // WHEN attempting to merge the configs
            let result = Config::build(Some(&global), path, vault);

            // THEN validation should succeed or fail as expected
            match expected_error_field {
                None => {
                    assert!(
                        result.is_ok(),
                        "Configuration with path='{path}' and level='{level}' should be valid, but failed: {result:?}"
                    );
                    if let Ok(config) = result {
                        assert!(
                            config.validate().is_ok(),
                            "Explicit validate() call should also pass for valid config"
                        );
                    }
                }
                Some(field_name) => {
                    let err =
                        result.expect_err("Validation should have failed");

                    // # LINT_DISABLE_REASON: Wildcard match is necessary for test resilience against new error variants. Panic is standard for test failure.
                    #[expect(
                        clippy::wildcard_enum_match_arm,
                        reason = "Test safety boundary"
                    )]
                    match err {
                        crate::ConfigError::ValidationFailed {
                            field,
                            ..
                        }
                        | crate::ConfigError::InvalidEnumValue {
                            field,
                            ..
                        } => {
                            assert_eq!(
                                field, field_name,
                                "Error reported for wrong field"
                            );
                        }
                        _ => {
                            panic!(
                                "Expected a validation-related error, found: {err:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    mod integrity {
        use super::*;

        #[test]
        fn supports_clone_debug_and_partial_eq() {
            // GIVEN a merged configuration built from valid fixtures
            let global = sample_global_config();
            let vault = sample_vault_config();

            // WHEN building the configuration
            let result1 = Config::build(Some(&global), "/vault", vault.clone());
            assert!(
                result1.is_ok(),
                "First merge for trait verification failed: {result1:?}"
            );

            // THEN debug/clone/eq traits behave as expected
            if let Ok(config) = result1 {
                let debug_str = format!("{config:?}");
                assert!(
                    !debug_str.is_empty(),
                    "Debug derivation should produce non-empty string"
                );

                let cloned = config.clone();
                assert_eq!(
                    config, cloned,
                    "Cloned config must be equal to original"
                );

                let result2 = Config::build(Some(&global), "/vault", vault);
                assert!(
                    result2.is_ok(),
                    "Second merge for trait verification failed"
                );
                if let Ok(config2) = result2 {
                    assert_eq!(
                        config, config2,
                        "Merged configs with identical input must be equal (PartialEq)"
                    );
                }
            }
        }

        #[test]
        fn merge_performance_meets_target() {
            // GIVEN valid global and vault configs
            let global = sample_global_config();
            let vault = sample_vault_config();

            // WHEN performing 1000 merge operations
            let start = std::time::Instant::now();
            for _ in 0i32..1000i32 {
                debug_assert!(
                    Config::build(Some(&global), "/vault", vault.clone())
                        .is_ok()
                );
            }
            let total_duration = start.elapsed();
            let avg_duration = total_duration / 1000;

            // THEN average merge time meets the performance target
            assert!(
                avg_duration < std::time::Duration::from_micros(100),
                "Config::build performance degraded: {}μs per operation (target: <100μs)",
                avg_duration.as_micros()
            );
        }
    }

    /// Test fixture: Create sample global configuration with system defaults.
    ///
    /// This fixture provides a complete Global configuration suitable for testing
    /// merge operations, validation logic, and default value fallback behavior.
    /// All fields are populated with realistic values that represent a typical
    /// global configuration setup.
    ///
    /// # Field Values
    /// - `filesystem.vault_path`: "." (placeholder, not used in global)
    /// - `filesystem.templates_dir`: "templates" (system default)
    /// - `filesystem.schemas_dir`: "schemas" (system default)
    /// - `frontmatter.*`: All set to system defaults (`"file_class"`, `"title"`, etc.)
    /// - `log_level`: "info" (system default)
    ///
    /// # Usage
    /// Use this fixture when you need a baseline global configuration for merge testing.
    /// ```rust
    /// let global = sample_global_config();
    /// let vault = sample_vault_config();
    /// let config = Config::build(&global, vault).unwrap();
    /// ```
    fn sample_global_config() -> Global {
        Global {
            filesystem: global::Filesystem {
                schema: Schema {
                    schemas_dir: "schemas".to_owned(),
                    property_bank_filename: "property_bank.json".to_owned(),
                },
                template: Template {
                    templates_dir: "templates".to_owned(),
                },
            },
            frontmatter: Frontmatter {
                alias_key: "aliases".to_owned(),
                date_created_key: "date_created".to_owned(),
                date_modified_key: "date_modified".to_owned(),
                file_class_key: "file_class".to_owned(),
                title_key: "title".to_owned(),
            },
            logging: Logging {
                log_level: "info".to_owned(),
            },
            trusted_vaults: None,
        }
    }

    /// Test fixture: Create sample vault configuration with user overrides.
    ///
    /// This fixture provides a complete Vault configuration with realistic overrides
    /// that demonstrate vault-level customization. Key overrides show how vault
    /// configuration takes precedence over global defaults.
    ///
    /// # Key Overrides (Vault takes precedence)
    /// - `filesystem.templates_dir`: `"custom_templates"` (vs global `"templates"`)
    /// - `filesystem.vault_path`: "/vault" (required field)
    /// - `frontmatter.file_class_key`: `"type"` (vs global `"file_class"`)
    /// - `frontmatter.date_created_key`: `"created"` (vs global `"date_created"`)
    /// - `log_level`: "debug" (vs global "info")
    ///
    /// # Usage
    /// Use this fixture to test vault-level overrides and precedence rules.
    /// ```rust
    /// let global = sample_global_config();
    /// let vault = sample_vault_config();
    /// let config = Config::build(&global, vault).unwrap();
    /// assert_eq!(config.vault_filesystem.template.templates_dir, "custom_templates"); // vault filesystem
    /// ```
    fn sample_vault_config() -> Vault {
        Vault {
            filesystem: vault::Filesystem {
                schema: Schema {
                    schemas_dir: "schemas".to_owned(), // same as global
                    property_bank_filename: "property_bank.json".to_owned(),
                },
                template: Template {
                    templates_dir: "custom_templates".to_owned(), // vault override
                },
                cache_dir: ".cache".to_owned(),
            },
            frontmatter: Some(Frontmatter {
                alias_key: "aliases".to_owned(),
                date_created_key: "created".to_owned(), // vault override
                date_modified_key: "modified".to_owned(), // vault override
                file_class_key: "type".to_owned(),      // vault override
                title_key: "title".to_owned(),
            }),
            logging: Some(Logging {
                log_level: "debug".to_owned(), // vault override
            }),
        }
    }
}
