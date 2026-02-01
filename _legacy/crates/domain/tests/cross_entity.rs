//! Cross-entity integration tests for domain aggregates.

// # LINT_DISABLE_REASON: Integration tests use unwrap for setup and assertions.
// # LINT_DISABLE_REASON: Options tried: manual Result handling.
// # LINT_DISABLE_REASON: Justification: Test code clarity.
#![expect(
    clippy::disallowed_methods,
    reason = "Integration tests use unwrap for setup and assertions for \
              clarity"
)]

use std::collections::HashMap;

use lithos_domain::{
    Config, FieldValue, Frontmatter, GlobalConfig, VaultConfig,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// 3.6-INT-001: `frontmatter_uses_config_keys`.
    /// Priority: P1.
    #[test]
    fn frontmatter_uses_config_keys() {
        // GIVEN: a custom config with non-standard frontmatter keys
        let mut global = GlobalConfig::default();
        global.frontmatter.title_key = "subject".to_owned();
        global.frontmatter.alias_key = "other_names".to_owned();

        let vault = VaultConfig::default();
        let config = Config::build(Some(&global), "/vault", vault).unwrap();

        // AND: frontmatter that uses those custom keys
        let mut fields = HashMap::new();
        fields.insert(
            "subject".to_owned(),
            FieldValue::String("Test Note".to_owned()),
        );
        fields.insert(
            "other_names".to_owned(),
            FieldValue::Array(vec![FieldValue::String("Alias 1".to_owned())]),
        );
        let fm = Frontmatter::new(fields).unwrap();

        // WHEN: extracting title and aliases using the config
        let title = fm.title(&config);
        let aliases = fm.aliases(&config);

        // THEN: the values are correctly retrieved using the configured keys
        assert_eq!(title, "Test Note");
        assert_eq!(aliases, vec!["Alias 1".to_owned()]);
    }

    /// 3.6-INT-002:
    /// `frontmatter_falls_back_to_defaults_when_config_keys_missing`.
    /// Priority: P2.
    #[test]
    fn frontmatter_falls_back_to_defaults_when_config_keys_missing() {
        // GIVEN: a default config
        let global = GlobalConfig::default();
        let vault = VaultConfig::default();
        let config = Config::build(Some(&global), "/vault", vault).unwrap();

        // AND: frontmatter that doesn't have the expected keys
        let fm = Frontmatter::new(HashMap::new()).unwrap();

        // WHEN: extracting title and aliases
        let title = fm.title(&config);
        let aliases = fm.aliases(&config);

        // THEN: they return default (empty) values
        assert_eq!(title, "");
        assert!(aliases.is_empty());
    }

    /// 3.6-INT-003: `config_validation_fails_if_vault_path_invalid`.
    /// Priority: P1.
    #[test]
    fn config_validation_fails_if_vault_path_invalid() {
        // GIVEN: an empty vault path
        let global = GlobalConfig::default();
        let vault = VaultConfig::default();

        // WHEN: building the config
        let result = Config::build(Some(&global), "", vault);

        // THEN: it fails validation during construction
        result.unwrap_err();
    }

    /// 3.6-INT-004: `schema_registration_maintains_bank_consistency`.
    /// Priority: P1.
    #[test]
    fn schema_registration_maintains_bank_consistency() {
        // GIVEN: a property bank and a property
        let mut bank = lithos_domain::PropertyBank::new();
        let name =
            lithos_domain::PropertyName::new("status".to_owned()).unwrap();
        let property = lithos_domain::Property::new(
            uuid::Uuid::now_v7(),
            name,
            true,
            false,
            lithos_domain::PropertySpec::Bool(
                lithos_domain::BoolSpec::default(),
            ),
        )
        .unwrap();

        // WHEN: registering the property
        bank.register(property.clone()).unwrap();

        // AND: creating a schema that uses it
        let schema_name =
            lithos_domain::SchemaName::new("test".to_owned()).unwrap();
        let schema = lithos_domain::Schema::new(
            uuid::Uuid::now_v7(),
            schema_name,
            vec![property],
        )
        .unwrap();

        // THEN: the schema's property matches the bank's property
        assert_eq!(
            schema.get("status").unwrap().id(),
            bank.get_by_name("status").unwrap().id()
        );
    }
}
