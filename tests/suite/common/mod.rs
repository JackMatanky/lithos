#![allow(dead_code)]

use lithos_test_utils::{
    FileTestVault,
    data::fixtures::{
        FakeData, Fixture, Scenario, combine, test_config, test_user,
    },
};

/// Shared integration-test setup helpers.
///
/// Place integration-only fixtures here to avoid compiling helper modules
/// as standalone test crates.
pub struct IntegrationFixtures {
    pub vault: FileTestVault,
    pub user: std::collections::HashMap<String, String>,
    pub config: std::collections::HashMap<String, String>,
}

impl IntegrationFixtures {
    pub fn new() -> Self {
        Self {
            vault: FileTestVault::new().expect("Should create test vault"),
            user: test_user(),
            config: test_config(),
        }
    }

    pub fn fake_user() -> std::collections::HashMap<String, String> {
        let mut data = std::collections::HashMap::new();
        let fake = FakeData::new(Scenario::Realistic);
        data.insert("name".to_string(), fake.name());
        data.insert("email".to_string(), fake.email());
        data
    }

    pub fn composite_fixture() -> Fixture {
        let user_fixture = Fixture::new("user").with("name", "Ada");
        let config_fixture = Fixture::new("config").with("debug", true);
        combine(vec![user_fixture, config_fixture])
    }
}
