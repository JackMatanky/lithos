//! Test fixtures and factory framework for generating test data.
//!
//! This module provides standardized test fixtures using rstest, builder patterns for
//! complex object construction, fake data generation, and serialization helpers.
//!
//! # Features
//!
//! - **rstest Integration**: Parameterized testing with fixture injection
//! - **Builder Patterns**: Fluent APIs for constructing complex domain objects
//! - **Fake Data Generation**: Realistic test data with configurable scenarios
//! - **Serialization Helpers**: JSON/binary persistence testing utilities

use std::{collections::HashMap, fmt, marker::PhantomData};

use fake::{Fake, Faker};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Generic builder pattern for constructing test objects.
///
/// Provides a fluent API for building complex objects with optional fields
/// and validation. Implements the builder pattern with method chaining.
/// ```
pub struct Builder<T, F> {
    constructor: F,
    defaults: Vec<Option<Box<dyn std::any::Any>>>,
    _phantom: PhantomData<T>,
}

impl<T, F> Builder<T, F>
where
    F: Fn(&[Box<dyn std::any::Any>]) -> T,
{
    /// Creates a new builder with the given constructor function.
    pub fn new(constructor: F) -> Self {
        Self {
            constructor,
            defaults: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Adds a default value for the next parameter.
    pub fn with_default<A: 'static>(mut self, default: A) -> Self {
        self.defaults.push(Some(Box::new(default)));
        self
    }

    /// Overrides the value for the current parameter position.
    pub fn with<A: 'static>(mut self, value: A) -> Self {
        let idx = self.defaults.len() - 1;
        self.defaults[idx] = Some(Box::new(value));
        self
    }

    /// Builds the object using the configured values.
    #[allow(clippy::disallowed_methods)]
    pub fn build(self) -> T {
        let params: Vec<Box<dyn std::any::Any>> =
            self.defaults.into_iter().map(|opt| opt.unwrap()).collect();
        (self.constructor)(&params)
    }
}

/// Fake data generation utilities for realistic test scenarios.
///
/// Provides configurable fake data generation using the `fake` crate,
/// with support for different scenarios and locales.
///
/// # Example
///
/// ```rust
/// use lithos_test_utils::fixtures::{FakeData, Scenario};
///
/// let fake = FakeData::new(Scenario::Realistic);
/// let name = fake.name();
/// let email = fake.email();
/// ```
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
    pub fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            locale: None,
        }
    }

    /// Sets the locale for fake data generation.
    pub fn with_locale(mut self, locale: &str) -> Self {
        self.locale = Some(locale.to_string());
        self
    }

    /// Generates a fake name.
    pub fn name(&self) -> String {
        match self.scenario {
            Scenario::Realistic => Faker.fake::<String>(),
            Scenario::EdgeCase => "A".to_string(), // Minimal valid name
            Scenario::Invalid => "".to_string(),   // Invalid empty name
        }
    }

    /// Generates a fake email address.
    pub fn email(&self) -> String {
        match self.scenario {
            Scenario::Realistic => Faker.fake::<String>(),
            Scenario::EdgeCase => "a@b.c".to_string(), // Minimal valid email
            Scenario::Invalid => "invalid-email".to_string(),
        }
    }

    /// Generates a fake UUID.
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
    pub fn integer(&self, min: i32, max: i32) -> i32 {
        match self.scenario {
            Scenario::Realistic => rand::thread_rng().gen_range(min..=max),
            Scenario::EdgeCase => min, // Minimum value
            Scenario::Invalid => min - 1, // Below minimum
        }
    }
}

/// Serialization helpers for JSON/binary persistence testing.
///
/// Provides utilities for testing serialization and deserialization
/// of domain objects in API and persistence contexts.
///
/// # Example
///
/// ```rust
/// use lithos_test_utils::fixtures::SerializationHelper;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct TestData { value: String }
///
/// let data = TestData { value: "test".to_string() };
/// let json = SerializationHelper::to_json(&data).unwrap();
/// let deserialized: TestData = SerializationHelper::from_json(&json).unwrap();
/// ```
pub struct SerializationHelper;

impl SerializationHelper {
    /// Serializes an object to JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json<T: Serialize>(
        value: &T,
    ) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(value)
    }

    /// Deserializes an object from JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn from_json<T: for<'de> Deserialize<'de>>(
        json: &str,
    ) -> Result<T, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serializes an object to binary format using MessagePack.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_binary<T: Serialize>(
        value: &T,
    ) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(value)
    }

    /// Deserializes an object from binary format using MessagePack.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn from_binary<T: for<'de> Deserialize<'de>>(
        data: &[u8],
    ) -> Result<T, rmp_serde::decode::Error> {
        rmp_serde::from_slice(data)
    }

    /// Validates that an object can be round-tripped through serialization.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or deserialization fails, or if the
    /// round-tripped object doesn't match the original.
    pub fn validate_round_trip<
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + fmt::Debug,
    >(
        original: &T,
    ) -> Result<(), String> {
        // JSON round-trip
        let json = Self::to_json(original)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;
        let from_json: T = Self::from_json(&json)
            .map_err(|e| format!("JSON deserialization failed: {}", e))?;
        if &from_json != original {
            return Err(format!(
                "JSON round-trip failed: expected {:?}, got {:?}",
                original, from_json
            ));
        }

        // Binary round-trip
        let binary = Self::to_binary(original)
            .map_err(|e| format!("Binary serialization failed: {}", e))?;
        let from_binary: T = Self::from_binary(&binary)
            .map_err(|e| format!("Binary deserialization failed: {}", e))?;
        if &from_binary != original {
            return Err(format!(
                "Binary round-trip failed: expected {:?}, got {:?}",
                original, from_binary
            ));
        }

        Ok(())
    }
}

/// Fixture composition utilities for combining test data.
///
/// Provides combinators for creating complex test scenarios by combining
/// multiple fixtures and applying transformations.
///
/// # Example
///
/// ```rust
/// use lithos_test_utils::fixtures::{Fixture, combine};
///
/// let fixture1 = Fixture::new("user").with("name", "Alice");
/// let fixture2 = Fixture::new("profile").with("age", 30);
/// let combined = combine(vec![fixture1, fixture2]);
/// ```
#[derive(Debug, Clone)]
pub struct Fixture {
    #[allow(dead_code)]
    name: String,
    data: HashMap<String, serde_json::Value>,
}

impl Fixture {
    /// Creates a new fixture with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            data: HashMap::new(),
        }
    }

    /// Adds a key-value pair to the fixture.
    pub fn with<T: Serialize>(mut self, key: &str, value: T) -> Self {
        self.data.insert(
            key.to_string(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }

    /// Gets a value from the fixture.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// Merges another fixture into this one.
    pub fn merge(mut self, other: Fixture) -> Self {
        for (key, value) in other.data {
            self.data.insert(key, value);
        }
        self
    }
}

/// Combines multiple fixtures into a single fixture.
pub fn combine(fixtures: Vec<Fixture>) -> Fixture {
    let mut combined = Fixture::new("combined");
    for fixture in fixtures {
        combined = combined.merge(fixture);
    }
    combined
}

/// rstest fixture functions for common test data.
///
/// These functions can be used with rstest's #[fixture] attribute
/// to provide test data injection.
///
/// # Example
///
/// ```rust
/// use lithos_test_utils::fixtures::{test_user, test_config};
/// use rstest::rstest;
///
/// #[rstest]
/// fn test_something(#[fixture] test_user: TestUser, #[fixture] test_config: Config) {
///     // Test code here
/// }
/// ```
pub fn test_user() -> HashMap<String, String> {
    let mut user = HashMap::new();
    user.insert("name".to_string(), FakeData::new(Scenario::Realistic).name());
    user.insert(
        "email".to_string(),
        FakeData::new(Scenario::Realistic).email(),
    );
    user
}

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

    impl TestUser {
        fn builder() -> Builder<Self, impl Fn(&[Box<dyn std::any::Any>]) -> Self>
        {
            Builder::new(|args: &[Box<dyn std::any::Any>]| {
                let id = args[0].downcast_ref::<u32>().copied().unwrap_or(1);
                let name = args[1]
                    .downcast_ref::<String>()
                    .cloned()
                    .unwrap_or_else(|| "Anonymous".to_string());
                let email =
                    args[2].downcast_ref::<Option<String>>().cloned().flatten();
                Self {
                    id,
                    name,
                    email,
                }
            })
        }
    }

    #[test]
    fn test_builder_pattern() {
        let user = TestUser::builder()
            .with_default(1) // id default
            .with(42u32) // override id
            .with_default("Anonymous".to_string()) // name default
            .with("Alice".to_string()) // override name
            .with_default(None as Option<String>) // email default
            .with(Some("alice@example.com".to_string())) // override email
            .build();

        assert_eq!(user.id, 42);
        assert_eq!(user.name, "Alice");
        assert_eq!(user.email, Some("alice@example.com".to_string()));
    }

    #[test]
    fn test_fake_data_generation() {
        let fake = FakeData::new(Scenario::Realistic);
        let name = fake.name();
        assert!(!name.is_empty());

        let email = fake.email();
        assert!(!email.is_empty()); // Email should not be empty

        let edge_fake = FakeData::new(Scenario::EdgeCase);
        let edge_name = edge_fake.name();
        assert_eq!(edge_name, "A");
    }

    #[test]
    fn test_serialization_helper() {
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
    fn test_fixture_composition() {
        let user_fixture =
            Fixture::new("user").with("name", "Alice").with("age", 30);

        let config_fixture = Fixture::new("config").with("debug", true);

        let combined = combine(vec![user_fixture, config_fixture]);

        assert_eq!(combined.get("name").unwrap(), "Alice");
        assert_eq!(combined.get("age").unwrap(), 30);
        assert_eq!(combined.get("debug").unwrap(), true);
    }

    #[test]
    fn test_fixture_functions() {
        let user = test_user();
        assert!(user.contains_key("name"));
        assert!(user.contains_key("email"));

        let config = test_config();
        assert!(config.contains_key("database_url"));
        assert!(config.contains_key("log_level"));
    }
}
