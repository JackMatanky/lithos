//! Test fixtures and factory framework for generating test data.
//!
//! This module provides standardized test fixtures using rstest, type-safe
//! builder patterns for complex object construction, fake data generation, and
//! serialization helpers.
//!
//! # Features
//!
//! - **Type-safe Builders**: Macro-based builders for robust object
//!   construction
//! - **Fake Data Generation**: Realistic test data with configurable scenarios
//! - **Serialization Helpers**: JSON/binary persistence testing utilities

use std::{collections::HashMap, fmt};

use fake::{Fake, Faker};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Macro to generate a type-safe test builder for a struct.
///
/// This replaces the brittle generic Builder with a robust, type-safe
/// implementation that provides fluent API and sensible defaults.
///
/// # Example
///
/// ```rust
/// use lithos_test_utils::test_builder;
///
/// struct User { id: u32, name: String }
///
/// test_builder!(UserBuilder, User, {
///     id: u32 = 1,
///     name: String = "Anonymous".to_string(),
/// });
///
/// let user = UserBuilder::new().id(42).build();
/// ```
#[macro_export]
macro_rules! test_builder {
    ($name:ident, $target:ident, { $($field:ident: $type:ty = $default:expr),* $(,)? }) => {
        #[derive(Debug, Clone)]
        #[allow(clippy::missing_docs_in_private_items)]
        pub struct $name {
            $($field: $type),*
        }
        impl Default for $name {
            fn default() -> Self {
                Self {
                    $($field: $default),*
                }
            }
        }
        impl $name {
            /// Creates a new builder with default values.
            #[must_use]
            pub fn new() -> Self {
                Self::default()
            }
            $(
                /// Sets the value for the field.
                #[must_use]
                pub fn $field(mut self, value: $type) -> Self {
                    self.$field = value;
                    self
                }
            )*
            /// Builds the target object.
            #[must_use]
            #[allow(private_interfaces)]
            pub fn build(self) -> $target {
                $target {
                    $($field: self.$field),*
                }
            }
        }
    }
}

/// Fake data generation utilities for realistic test scenarios.
///
/// Provides configurable fake data generation using the `fake` crate,
/// with support for different scenarios and locales.
#[derive(Debug, Clone, Copy)]
pub enum Scenario {
    /// Realistic data that looks like production data
    Realistic,
    /// Edge case data for boundary testing
    EdgeCase,
    /// Invalid data for error path testing
    Invalid,
}

#[derive(Debug, Clone)]
pub struct FakeData {
    scenario: Scenario,
    locale: Option<String>,
}

impl FakeData {
    /// Creates a new fake data generator with the specified scenario.
    #[must_use]
    pub fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            locale: None,
        }
    }

    /// Sets the locale for fake data generation.
    #[must_use]
    pub fn with_locale(mut self, locale: &str) -> Self {
        self.locale = Some(locale.to_string());
        self
    }

    /// Generates a fake name.
    #[must_use]
    pub fn name(&self) -> String {
        match self.scenario {
            Scenario::Realistic => Faker.fake::<String>(),
            Scenario::EdgeCase => "A".to_string(), // Minimal valid name
            Scenario::Invalid => "".to_string(),   // Invalid empty name
        }
    }

    /// Generates a fake email address.
    #[must_use]
    pub fn email(&self) -> String {
        match self.scenario {
            Scenario::Realistic => Faker.fake::<String>(),
            Scenario::EdgeCase => "a@b.c".to_string(), // Minimal valid email
            Scenario::Invalid => "invalid-email".to_string(),
        }
    }

    /// Generates a fake UUID.
    #[must_use]
    pub fn uuid(&self) -> String {
        match self.scenario {
            Scenario::Realistic => Faker.fake::<String>(),
            Scenario::EdgeCase => {
                "00000000-0000-0000-0000-000000000000".to_string()
            }
            Scenario::Invalid => "not-a-uuid".to_string(),
        }
    }

    /// Generates a fake integer within a range.
    #[must_use]
    pub fn integer(&self, min: i32, max: i32) -> i32 {
        match self.scenario {
            Scenario::Realistic => {
                let mut rng = rand::thread_rng();
                rng.gen_range(min..=max)
            }
            Scenario::EdgeCase => min, // Minimum value
            Scenario::Invalid => min - 1, // Below minimum
        }
    }
}

/// Serialization helpers for JSON/binary persistence testing.
pub struct SerializationHelper;

impl SerializationHelper {
    /// Serializes an object to JSON string.
    pub fn to_json<T: Serialize>(
        value: &T,
    ) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(value)
    }

    /// Deserializes an object from JSON string.
    pub fn from_json<T: for<'de> Deserialize<'de>>(
        json: &str,
    ) -> Result<T, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serializes an object to binary format using MessagePack.
    pub fn to_binary<T: Serialize>(
        value: &T,
    ) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(value)
    }

    /// Deserializes an object from binary format using MessagePack.
    pub fn from_binary<T: for<'de> Deserialize<'de>>(
        data: &[u8],
    ) -> Result<T, rmp_serde::decode::Error> {
        rmp_serde::from_slice(data)
    }

    /// Validates that an object can be round-tripped through serialization.
    pub fn validate_round_trip<
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + fmt::Debug,
    >(
        original: &T,
    ) -> Result<(), String> {
        // JSON round-trip
        let json = Self::to_json(original)
            .map_err(|e| format!("JSON serialization failed: {e}"))?;
        let from_json: T = Self::from_json(&json)
            .map_err(|e| format!("JSON deserialization failed: {e}"))?;
        if &from_json != original {
            return Err(format!(
                "JSON round-trip failed: expected {original:?}, got \
                 {from_json:?}"
            ));
        }

        // Binary round-trip
        let binary = Self::to_binary(original)
            .map_err(|e| format!("Binary serialization failed: {e}"))?;
        let from_binary: T = Self::from_binary(&binary)
            .map_err(|e| format!("Binary deserialization failed: {e}"))?;
        if &from_binary != original {
            return Err(format!(
                "Binary round-trip failed: expected {original:?}, got \
                 {from_binary:?}"
            ));
        }

        Ok(())
    }
}

/// Fixture composition utilities for combining test data.
#[derive(Debug, Clone)]
pub struct Fixture {
    #[allow(dead_code)]
    name: String,
    data: HashMap<String, serde_json::Value>,
}

impl Fixture {
    /// Creates a new fixture with the given name.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            data: HashMap::new(),
        }
    }

    /// Adds a key-value pair to the fixture.
    #[must_use]
    pub fn with<T: Serialize>(mut self, key: &str, value: T) -> Self {
        self.data.insert(
            key.to_string(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }

    /// Gets a value from the fixture.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// Merges another fixture into this one.
    #[must_use]
    pub fn merge(mut self, other: Fixture) -> Self {
        for (key, value) in other.data {
            self.data.insert(key, value);
        }
        self
    }
}

/// Combines multiple fixtures into a single fixture.
#[must_use]
pub fn combine(fixtures: Vec<Fixture>) -> Fixture {
    let mut combined = Fixture::new("combined");
    for fixture in fixtures {
        combined = combined.merge(fixture);
    }
    combined
}

/// rstest fixture functions for common test data.
#[must_use]
pub fn test_user() -> HashMap<String, String> {
    let mut user = HashMap::new();
    user.insert("name".to_string(), FakeData::new(Scenario::Realistic).name());
    user.insert(
        "email".to_string(),
        FakeData::new(Scenario::Realistic).email(),
    );
    user
}

#[must_use]
pub fn test_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("database_url".to_string(), "sqlite::memory:".to_string());
    config.insert("log_level".to_string(), "debug".to_string());
    config
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestUser {
        id: u32,
        name: String,
        email: Option<String>,
    }

    test_builder!(TestUserBuilder, TestUser, {
        id: u32 = 1,
        name: String = "Anonymous".to_string(),
        email: Option<String> = None,
    });

    #[test]
    fn builder_pattern_generates_type_safe_builders() {
        let user = TestUserBuilder::new()
            .id(42)
            .name("Alice".to_string())
            .email(Some("alice@example.com".to_string()))
            .build();

        assert_eq!(user.id, 42);
        assert_eq!(user.name, "Alice");
        assert_eq!(user.email, Some("alice@example.com".to_string()));
    }

    #[test]
    fn fake_data_generator_produces_realistic_values() {
        let fake = FakeData::new(Scenario::Realistic);
        let name = fake.name();
        assert!(!name.is_empty());

        let email = fake.email();
        assert!(!email.is_empty());

        let edge_fake = FakeData::new(Scenario::EdgeCase);
        let edge_name = edge_fake.name();
        assert_eq!(edge_name, "A");
    }

    #[test]
    fn serialization_helper_round_trips_data_correctly() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestData {
            value: String,
            number: i32,
        }

        let data = TestData {
            value: "test".to_string(),
            number: 42,
        };

        let json = SerializationHelper::to_json(&data).unwrap();
        let deserialized: TestData =
            SerializationHelper::from_json(&json).unwrap();
        assert_eq!(data, deserialized);

        SerializationHelper::validate_round_trip(&data).unwrap();
    }

    #[test]
    fn fixture_composition_merges_multiple_sources() {
        let user_fixture =
            Fixture::new("user").with("name", "Alice").with("age", 30);

        let config_fixture = Fixture::new("config").with("debug", true);

        let combined = combine(vec![user_fixture, config_fixture]);

        assert_eq!(combined.get("name").unwrap(), "Alice");
        assert_eq!(combined.get("age").unwrap(), 30);
        assert_eq!(combined.get("debug").unwrap(), true);
    }

    #[test]
    fn fixture_functions_provide_sensible_defaults() {
        let user = test_user();
        assert!(user.contains_key("name"));
        assert!(user.contains_key("email"));

        let config = test_config();
        assert!(config.contains_key("database_url"));
        assert!(config.contains_key("log_level"));
    }
}
