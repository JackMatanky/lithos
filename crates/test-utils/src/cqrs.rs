//! # CQRS Testing Framework
//!
//! This module provides comprehensive testing utilities for CQRS (Command Query Responsibility Segregation)
//! patterns, supporting command handler testing, query handler testing, event sourcing aggregates,
//! and eventual consistency validation.
//!
//! ## Architecture Alignment
//!
//! Implements ADR 0009: CQRS Testing Patterns and Best Practices
//! - Command handlers use mock repositories (`Arc<dyn RepositoryPort>`)
//! - Query handlers use stubbed data stores
//! - Event sourcing uses Given-When-Then framework
//! - Eventual consistency testing with precise timing control
//!
//! ## Usage Examples
//!
//! ### Command Handler Testing
//! ```rust,ignore
//! use lithos_test_utils::cqrs::MockRepository;
//!
//! #[tokio::test]
//! async fn test_command_handler() {
//!     let mock_repo = Arc::new(MockRepository::new());
//!     let handler = CreateUserHandler::new(mock_repo.clone());
//!
//!     let command = CreateUserCommand { /* ... */ };
//!     handler.handle(command).await.unwrap();
//!
//!     assert_eq!(mock_repo.save_count(), 1);
//! }
//! ```
//!
//! ### Event Sourcing Testing
//! ```rust,ignore
//! use lithos_test_utils::cqrs::TestFramework;
//!
//! #[test]
//! fn test_aggregate_command() {
//!     TestFramework::default()
//!         .given(vec![AccountOpened { id: 1 }])
//!         .when(DepositMoney { amount: 100 })
//!         .execute(|history, cmd| {
//!             // apply history to aggregate and handle cmd
//!             vec![MoneyDeposited { amount: 100 }]
//!         })
//!         .then_expect_events(vec![MoneyDeposited { amount: 100 }]);
//! }
//! ```

use std::{collections::HashMap, fmt::Debug, marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use tokio::sync::{Mutex, RwLock};

// Submodules for specialized CQRS testing utilities
pub mod observability;
pub mod security;

/// Result type for CQRS testing operations
pub type CqrsTestResult<T> = Result<T, CqrsTestError>;

/// Errors that can occur during CQRS testing
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CqrsTestError {
    /// Event verification failed
    #[error("Event verification failed: {0}")]
    EventVerificationFailed(String),

    /// Mock repository operation failed
    #[error("Mock repository error: {0}")]
    MockRepositoryError(String),

    /// Stub query store operation failed
    #[error("Stub query store error: {0}")]
    StubQueryError(String),

    /// Aggregate testing framework error
    #[error("Aggregate test error: {0}")]
    AggregateTestError(String),

    /// Eventual consistency timeout
    #[error("Eventual consistency timeout: {0}")]
    ConsistencyTimeout(String),

    /// Generic test error
    #[error("Test error: {0}")]
    TestError(String),
}

/// Trait for entities that can be persisted by repositories
#[async_trait]
pub trait Entity: Send + Sync + Clone + Debug {
    /// The type of the entity's identifier
    type Id: Send + Sync + Clone + Debug + PartialEq + Eq + std::hash::Hash;

    /// Get the entity's identifier
    fn id(&self) -> &Self::Id;
}

/// Port trait for command-side repositories
#[async_trait]
pub trait RepositoryPort<E: Entity>: Send + Sync {
    /// Save an entity
    async fn save(&self, entity: E) -> CqrsTestResult<()>;

    /// Find an entity by ID
    async fn find_by_id(&self, id: &E::Id) -> CqrsTestResult<Option<E>>;

    /// Delete an entity by ID
    async fn delete(&self, id: &E::Id) -> CqrsTestResult<()>;

    /// Check if entity exists
    async fn exists(&self, id: &E::Id) -> CqrsTestResult<bool>;
}

/// Mock repository for command handler testing
pub struct MockRepository<E: Entity> {
    /// Stored entities indexed by ID
    entities: Arc<RwLock<HashMap<E::Id, E>>>,
    /// Interaction history
    interactions: Arc<Mutex<Vec<RepositoryInteraction<E>>>>,
    /// Configured error responses
    error_config: Arc<RwLock<ErrorConfig>>,
}

/// Record of repository interaction for verification
#[derive(Debug, Clone)]
pub enum RepositoryInteraction<E: Entity> {
    /// Save operation
    Save(E),
    /// Find by ID operation
    FindById(E::Id),
    /// Delete operation
    Delete(E::Id),
    /// Exists check operation
    Exists(E::Id),
}

/// Configuration for simulating repository errors
#[derive(Debug, Clone, Default)]
pub struct ErrorConfig {
    /// Error to return on next save
    pub save_error: Option<String>,
    /// Error to return on next find
    pub find_error: Option<String>,
    /// Error to return on next delete
    pub delete_error: Option<String>,
}

impl<E: Entity> MockRepository<E> {
    /// Create a new mock repository
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: Arc::new(RwLock::new(HashMap::new())),
            interactions: Arc::new(Mutex::new(Vec::new())),
            error_config: Arc::new(RwLock::new(ErrorConfig::default())),
        }
    }

    /// Create a mock repository with pre-populated entities
    #[must_use]
    pub fn with_entities(entities: Vec<E>) -> Self {
        let mut map = HashMap::new();
        for entity in entities {
            map.insert(entity.id().clone(), entity);
        }
        Self {
            entities: Arc::new(RwLock::new(map)),
            interactions: Arc::new(Mutex::new(Vec::new())),
            error_config: Arc::new(RwLock::new(ErrorConfig::default())),
        }
    }

    /// Get the number of save operations performed
    pub async fn save_count(&self) -> usize {
        let interactions = self.interactions.lock().await;
        interactions
            .iter()
            .filter(|i| matches!(i, RepositoryInteraction::Save(_)))
            .count()
    }

    /// Get the number of find operations performed
    pub async fn find_count(&self) -> usize {
        let interactions = self.interactions.lock().await;
        interactions
            .iter()
            .filter(|i| matches!(i, RepositoryInteraction::FindById(_)))
            .count()
    }

    /// Get all interactions for verification
    pub async fn interactions(&self) -> Vec<RepositoryInteraction<E>> {
        self.interactions.lock().await.clone()
    }

    /// Configure save operation to fail with error
    pub async fn fail_next_save(&self, error: impl Into<String>) {
        let mut config = self.error_config.write().await;
        config.save_error = Some(error.into());
    }

    /// Clear all recorded interactions
    pub async fn clear_interactions(&self) {
        self.interactions.lock().await.clear();
    }

    /// Get all stored entities
    pub async fn all_entities(&self) -> Vec<E> {
        self.entities.read().await.values().cloned().collect()
    }
}

impl<E: Entity> Default for MockRepository<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<E: Entity> RepositoryPort<E> for MockRepository<E> {
    async fn save(&self, entity: E) -> CqrsTestResult<()> {
        self.interactions
            .lock()
            .await
            .push(RepositoryInteraction::Save(entity.clone()));

        let mut config = self.error_config.write().await;
        if let Some(error) = config.save_error.take() {
            return Err(CqrsTestError::MockRepositoryError(error));
        }

        self.entities.write().await.insert(entity.id().clone(), entity);
        Ok(())
    }

    async fn find_by_id(&self, id: &E::Id) -> CqrsTestResult<Option<E>> {
        self.interactions
            .lock()
            .await
            .push(RepositoryInteraction::FindById(id.clone()));

        let mut config = self.error_config.write().await;
        if let Some(error) = config.find_error.take() {
            return Err(CqrsTestError::MockRepositoryError(error));
        }

        Ok(self.entities.read().await.get(id).cloned())
    }

    async fn delete(&self, id: &E::Id) -> CqrsTestResult<()> {
        self.interactions
            .lock()
            .await
            .push(RepositoryInteraction::Delete(id.clone()));

        let mut config = self.error_config.write().await;
        if let Some(error) = config.delete_error.take() {
            return Err(CqrsTestError::MockRepositoryError(error));
        }

        self.entities.write().await.remove(id);
        Ok(())
    }

    async fn exists(&self, id: &E::Id) -> CqrsTestResult<bool> {
        self.interactions
            .lock()
            .await
            .push(RepositoryInteraction::Exists(id.clone()));

        Ok(self.entities.read().await.contains_key(id))
    }
}

/// Port trait for query-side data stores
#[async_trait]
pub trait QueryStorePort<T: Send + Sync>: Send + Sync {
    /// Query for items matching criteria
    async fn query(&self, criteria: &QueryCriteria) -> CqrsTestResult<Vec<T>>;

    /// Get a single item by ID
    async fn get_by_id(&self, id: &str) -> CqrsTestResult<Option<T>>;

    /// Count items matching criteria
    async fn count(&self, criteria: &QueryCriteria) -> CqrsTestResult<usize>;
}

/// Query criteria for filtering and pagination
#[derive(Debug, Clone, Default)]
pub struct QueryCriteria {
    /// Filters to apply
    pub filters: HashMap<String, String>,
    /// Sort field
    pub sort_by: Option<String>,
    /// Sort direction
    pub sort_desc: bool,
    /// Page number (0-indexed)
    pub page: usize,
    /// Page size
    pub page_size: usize,
}

impl QueryCriteria {
    /// Create default query criteria
    #[must_use]
    pub fn new() -> Self {
        Self {
            filters: HashMap::new(),
            sort_by: None,
            sort_desc: false,
            page: 0,
            page_size: 20,
        }
    }

    /// Add a filter
    #[must_use]
    pub fn with_filter(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.filters.insert(key.into(), value.into());
        self
    }

    /// Set sort field
    #[must_use]
    pub fn sort_by(mut self, field: impl Into<String>) -> Self {
        self.sort_by = Some(field.into());
        self
    }

    /// Set page
    #[must_use]
    pub fn page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }
}

/// Stubbed query store for query handler testing
pub struct StubQueryStore<T: Send + Sync + Clone> {
    /// Pre-configured test data
    data: Arc<RwLock<Vec<T>>>,
    /// ID extractor function
    id_extractor: Arc<dyn Fn(&T) -> String + Send + Sync>,
}

impl<T: Send + Sync + Clone + 'static> StubQueryStore<T> {
    /// Create a new stub query store with data
    #[must_use]
    pub fn with_data(
        data: Vec<T>,
        id_extractor: impl Fn(&T) -> String + Send + Sync + 'static,
    ) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            id_extractor: Arc::new(id_extractor),
        }
    }

    /// Add data to the stub store
    pub async fn add(&self, item: T) {
        self.data.write().await.push(item);
    }

    /// Clear all data
    pub async fn clear(&self) {
        self.data.write().await.clear();
    }

    /// Get all data
    pub async fn all_data(&self) -> Vec<T> {
        self.data.read().await.clone()
    }
}

#[async_trait]
impl<T: Send + Sync + Clone + 'static> QueryStorePort<T> for StubQueryStore<T> {
    async fn query(&self, _criteria: &QueryCriteria) -> CqrsTestResult<Vec<T>> {
        Ok(self.data.read().await.clone())
    }

    async fn get_by_id(&self, id: &str) -> CqrsTestResult<Option<T>> {
        let data = self.data.read().await;
        Ok(data.iter().find(|item| (self.id_extractor)(item) == id).cloned())
    }

    async fn count(&self, _criteria: &QueryCriteria) -> CqrsTestResult<usize> {
        Ok(self.data.read().await.len())
    }
}

/// Given-When-Then test framework for event sourcing aggregates
pub struct TestFramework<A, C, E> {
    /// Initial event history
    given_events: Vec<E>,
    /// Command to execute
    when_command: Option<C>,
    /// Phantom data for aggregate type
    _phantom: PhantomData<A>,
}

impl<A, C, E> Default for TestFramework<A, C, E> {
    fn default() -> Self {
        Self {
            given_events: Vec::new(),
            when_command: None,
            _phantom: PhantomData,
        }
    }
}

impl<A, C, E> TestFramework<A, C, E>
where
    E: Clone + Debug + PartialEq,
{
    /// Create a new test framework
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set up initial event history (GIVEN phase)
    #[must_use]
    pub fn given(mut self, events: Vec<E>) -> Self {
        self.given_events = events;
        self
    }

    /// Execute command (WHEN phase)
    #[must_use]
    pub fn when(mut self, command: C) -> Self {
        self.when_command = Some(command);
        self
    }

    /// Executes the test with a provided handler and returns a result stage.
    pub fn execute<H>(self, handler: H) -> TestResultStage<E>
    where
        H: FnOnce(Vec<E>, C) -> Vec<E>,
    {
        let command = self.when_command.expect("WHEN command must be set");
        let published_events = handler(self.given_events, command);
        TestResultStage {
            published_events,
        }
    }
}

/// Result stage for asserting expected events.
pub struct TestResultStage<E> {
    published_events: Vec<E>,
}

impl<E: Clone + Debug + PartialEq> TestResultStage<E> {
    /// Assert that published events match the expected sequence.
    pub fn then_expect_events(self, expected: Vec<E>) {
        assert_eq!(
            self.published_events, expected,
            "Actual published events do not match expected events"
        );
    }

    /// Access the published events for further manual assertions.
    #[must_use]
    pub fn events(&self) -> &[E] {
        &self.published_events
    }
}

/// Event verification utilities for command handler testing
pub struct EventVerifier<E> {
    /// Captured events
    events: Arc<Mutex<Vec<E>>>,
}

impl<E: Clone + Debug + PartialEq> EventVerifier<E> {
    /// Create a new event verifier
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record an event
    pub async fn record(&self, event: E) {
        self.events.lock().await.push(event);
    }

    /// Get all recorded events
    pub async fn events(&self) -> Vec<E> {
        self.events.lock().await.clone()
    }

    /// Verify event count
    pub async fn assert_event_count(
        &self,
        expected: usize,
    ) -> CqrsTestResult<()> {
        let actual = self.events.lock().await.len();
        if actual == expected {
            Ok(())
        } else {
            Err(CqrsTestError::EventVerificationFailed(format!(
                "Expected {expected} events, got {actual}"
            )))
        }
    }

    /// Verify specific event exists
    pub async fn assert_event_exists(
        &self,
        expected: &E,
    ) -> CqrsTestResult<()> {
        let events = self.events.lock().await;
        if events.contains(expected) {
            Ok(())
        } else {
            Err(CqrsTestError::EventVerificationFailed(format!(
                "Expected event not found: {expected:?}"
            )))
        }
    }

    /// Clear recorded events
    pub async fn clear(&self) {
        self.events.lock().await.clear();
    }
}

impl<E: Clone + Debug + PartialEq> Default for EventVerifier<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Eventual consistency testing utilities for write/read model synchronization
pub struct EventualConsistencyTester {
    /// Default timeout for consistency checks
    default_timeout: tokio::time::Duration,
    /// Polling interval for condition checks
    poll_interval: tokio::time::Duration,
}

impl EventualConsistencyTester {
    /// Create a new eventual consistency tester with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            default_timeout: tokio::time::Duration::from_millis(500),
            poll_interval: tokio::time::Duration::from_millis(10),
        }
    }

    /// Create a tester with custom timeout
    #[must_use]
    pub fn with_timeout(timeout: tokio::time::Duration) -> Self {
        Self {
            default_timeout: timeout,
            poll_interval: tokio::time::Duration::from_millis(10),
        }
    }

    /// Wait for a condition to become true within timeout period
    pub async fn wait_for_condition<F, Fut>(
        &self,
        condition: F,
        timeout: tokio::time::Duration,
    ) -> CqrsTestResult<()>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        crate::async_utils::poll_condition(
            condition,
            timeout,
            self.poll_interval,
        )
        .await
        .map_err(CqrsTestError::ConsistencyTimeout)
    }

    /// Wait for a condition using default timeout
    pub async fn wait_for_condition_default<F, Fut>(
        &self,
        condition: F,
    ) -> CqrsTestResult<()>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        self.wait_for_condition(condition, self.default_timeout).await
    }

    /// Wait for a value to be available within timeout
    pub async fn wait_for_value<F, Fut, T>(
        &self,
        mut getter: F,
        timeout: tokio::time::Duration,
    ) -> CqrsTestResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Option<T>>,
    {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            if let Some(value) = getter().await {
                return Ok(value);
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(CqrsTestError::ConsistencyTimeout(format!(
                    "Value not available within {timeout:?}"
                )));
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Verify race condition prevention by ensuring operations complete in order
    pub async fn verify_ordering<F1, F2, Fut1, Fut2>(
        &self,
        first_op: F1,
        second_op: F2,
        gap: tokio::time::Duration,
    ) -> CqrsTestResult<()>
    where
        F1: FnOnce() -> Fut1 + Send + 'static,
        F2: FnOnce() -> Fut2 + Send + 'static,
        Fut1: std::future::Future<Output = ()> + Send,
        Fut2: std::future::Future<Output = ()> + Send,
    {
        let first_complete = Arc::new(tokio::sync::Mutex::new(false));
        let second_complete = Arc::new(tokio::sync::Mutex::new(false));

        let first_flag = Arc::clone(&first_complete);
        let second_flag = Arc::clone(&second_complete);

        // Execute first operation
        let handle1 = tokio::spawn(async move {
            first_op().await;
            *first_flag.lock().await = true;
        });

        // Wait for gap
        tokio::time::sleep(gap).await;

        // Execute second operation
        let handle2 = tokio::spawn(async move {
            second_op().await;
            *second_flag.lock().await = true;
        });

        handle1.await.map_err(|e| {
            CqrsTestError::TestError(format!("First operation failed: {e}"))
        })?;
        handle2.await.map_err(|e| {
            CqrsTestError::TestError(format!("Second operation failed: {e}"))
        })?;

        // Verify both completed
        let first_done = *first_complete.lock().await;
        let second_done = *second_complete.lock().await;

        if !first_done || !second_done {
            return Err(CqrsTestError::TestError(
                "Operations did not complete as expected".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for EventualConsistencyTester {
    fn default() -> Self {
        Self::new()
    }
}

/// Cross-aggregate saga testing utilities
pub struct SagaTester {
    /// Participants in the saga with their update status
    participants: Arc<RwLock<HashMap<String, bool>>>,
    /// Events captured during saga execution
    events: Arc<Mutex<Vec<String>>>,
}

impl SagaTester {
    /// Create a new saga tester
    #[must_use]
    pub fn new() -> Self {
        Self {
            participants: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a saga participant
    pub async fn track_participant(&self, name: impl Into<String>) {
        self.participants.write().await.insert(name.into(), false);
    }

    /// Mark a participant as updated
    pub async fn mark_updated(&self, name: &str) -> CqrsTestResult<()> {
        let mut participants = self.participants.write().await;
        if let Some(status) = participants.get_mut(name) {
            *status = true;
            Ok(())
        } else {
            Err(CqrsTestError::TestError(format!(
                "Unknown participant: {name}"
            )))
        }
    }

    /// Check if a participant has been updated
    pub async fn is_updated(&self, name: &str) -> bool {
        self.participants.read().await.get(name).copied().unwrap_or(false)
    }

    /// Verify all participants have been updated within timeout
    pub async fn verify_all_updated(
        &self,
        timeout: tokio::time::Duration,
    ) -> CqrsTestResult<()> {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let all_updated = {
                let participants = self.participants.read().await;
                participants.values().all(|&status| status)
            };

            if all_updated {
                return Ok(());
            }

            if tokio::time::Instant::now() >= deadline {
                let participants = self.participants.read().await;
                let pending: Vec<_> = participants
                    .iter()
                    .filter(|(_, status)| !**status)
                    .map(|(name, _)| name.clone())
                    .collect();

                return Err(CqrsTestError::ConsistencyTimeout(format!(
                    "Not all participants updated within {timeout:?}. Pending: {pending:?}"
                )));
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Record an event in the saga
    pub async fn record_event(&self, event: impl Into<String>) {
        self.events.lock().await.push(event.into());
    }

    /// Get all recorded events
    pub async fn events(&self) -> Vec<String> {
        self.events.lock().await.clone()
    }

    /// Verify event sequence matches expected order
    pub async fn verify_event_sequence(
        &self,
        expected: &[&str],
    ) -> CqrsTestResult<()> {
        let actual = self.events.lock().await;
        let actual_strs: Vec<&str> =
            actual.iter().map(String::as_str).collect();

        if actual_strs == expected {
            Ok(())
        } else {
            Err(CqrsTestError::EventVerificationFailed(format!(
                "Event sequence mismatch. Expected: {expected:?}, Actual: {actual_strs:?}"
            )))
        }
    }

    /// Clear all saga state for reuse
    pub async fn clear(&self) {
        self.participants.write().await.clear();
        self.events.lock().await.clear();
    }
}

impl Default for SagaTester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::expect_used)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestUser {
        id: String,
        name: String,
    }

    impl Entity for TestUser {
        type Id = String;

        fn id(&self) -> &Self::Id {
            &self.id
        }
    }

    #[tokio::test]
    async fn mock_repository_saves_entity() {
        let repo = MockRepository::new();
        let user = TestUser {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };

        repo.save(user.clone()).await.unwrap();

        let found = repo.find_by_id(&"1".to_string()).await.unwrap();
        assert_eq!(found, Some(user));
    }

    #[tokio::test]
    async fn mock_repository_records_interactions() {
        let repo = MockRepository::new();
        let user = TestUser {
            id: "1".to_string(),
            name: "Bob".to_string(),
        };

        repo.save(user.clone()).await.unwrap();
        repo.find_by_id(&"1".to_string()).await.unwrap();

        assert_eq!(repo.save_count().await, 1);
        assert_eq!(repo.find_count().await, 1);
    }

    #[tokio::test]
    async fn mock_repository_fails_on_configured_error() {
        let repo = MockRepository::<TestUser>::new();
        repo.fail_next_save("Simulated error").await;

        let user = TestUser {
            id: "1".to_string(),
            name: "Charlie".to_string(),
        };

        let result = repo.save(user).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn stub_query_store_returns_data() {
        let users = vec![
            TestUser {
                id: "1".to_string(),
                name: "Alice".to_string(),
            },
            TestUser {
                id: "2".to_string(),
                name: "Bob".to_string(),
            },
        ];

        let store = StubQueryStore::with_data(users.clone(), |u: &TestUser| {
            u.id.clone()
        });

        let criteria = QueryCriteria::new();
        let result = store.query(&criteria).await.unwrap();

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn event_verifier_records_events() {
        let verifier = EventVerifier::new();

        verifier.record("Event1".to_string()).await;
        verifier.record("Event2".to_string()).await;

        verifier.assert_event_count(2).await.unwrap();
        verifier.assert_event_exists(&"Event1".to_string()).await.unwrap();
    }

    #[tokio::test]
    async fn eventual_consistency_tester_waits_for_condition() {
        let tester = EventualConsistencyTester::new();
        let flag = Arc::new(tokio::sync::Mutex::new(false));
        let flag_clone = Arc::clone(&flag);

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            *flag_clone.lock().await = true;
        });

        let flag_check = Arc::clone(&flag);
        let result = tester
            .wait_for_condition(
                || {
                    let flag = Arc::clone(&flag_check);
                    async move { *flag.lock().await }
                },
                tokio::time::Duration::from_millis(200),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn saga_tester_verifies_all_updated() {
        let tester = Arc::new(SagaTester::new());

        tester.track_participant("inventory").await;
        tester.track_participant("payment").await;

        let tester_clone1 = Arc::clone(&tester);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
            tester_clone1.mark_updated("inventory").await.unwrap();
        });

        let tester_clone2 = Arc::clone(&tester);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(60)).await;
            tester_clone2.mark_updated("payment").await.unwrap();
        });

        let result = tester
            .verify_all_updated(tokio::time::Duration::from_millis(200))
            .await;

        assert!(result.is_ok());
    }

    #[test]
    fn framework_verifies_event_sequence_successfully() {
        #[derive(Debug, Clone, PartialEq)]
        enum TestEvent {
            Created,
        }

        TestFramework::<(), (), TestEvent>::new()
            .given(vec![])
            .when(())
            .execute(|_history, _cmd| vec![TestEvent::Created])
            .then_expect_events(vec![TestEvent::Created]);
    }
}
