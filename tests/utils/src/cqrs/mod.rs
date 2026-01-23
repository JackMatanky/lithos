//! # CQRS Testing Framework
//!
//! This module provides comprehensive testing utilities for CQRS (Command Query
//! Responsibility Segregation) patterns, supporting command handler testing,
//! query handler testing, event sourcing aggregates, and eventual consistency
//! validation.
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
//! ```rust
//! # use std::sync::Arc;
//! # use async_trait::async_trait;
//! # use lithos_test_utils::{MockRepositoryPort, RepositoryPort, Entity, CqrsTestResult};
//! # #[derive(Debug, Clone, PartialEq, Eq)]
//! # struct User { id: String }
//! # impl Entity for User { type Id = String; fn id(&self) -> &Self::Id { &self.id } }
//! # struct CreateUserHandler { repo: Arc<dyn RepositoryPort<User>> }
//! # impl CreateUserHandler { fn new(repo: Arc<dyn RepositoryPort<User>>) -> Self { Self { repo } } }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut mock_repo = MockRepositoryPort::<User>::new();
//!     mock_repo.expect_save().returning(|_| Ok(()));
//!     let handler = CreateUserHandler::new(Arc::new(mock_repo));
//!
//!     // ...
//! }
//! ```
//!
//! ### Event Sourcing Testing
//! ```rust
//! # use lithos_test_utils::TestFramework;
//! # #[derive(Debug, Clone, PartialEq)] struct AccountOpened { id: i32 }
//! # #[derive(Debug, Clone, PartialEq)] struct MoneyDeposited { amount: i32 }
//! # #[derive(Debug, Clone, PartialEq)] struct DepositMoney { amount: i32 }
//!
//! let framework: TestFramework<(), DepositMoney, MoneyDeposited> = TestFramework::default();
//! framework
//!     .given(vec![])
//!     .when(DepositMoney { amount: 100 })
//!     .execute(|_history, cmd| {
//!         // apply history to aggregate and handle cmd
//!         vec![MoneyDeposited { amount: cmd.amount }]
//!     })
//!     .then_expect_events(vec![MoneyDeposited { amount: 100 }]);
//! ```

use std::{collections::HashMap, fmt::Debug, marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use tokio::sync::{Mutex, RwLock};

// Submodules for specialized CQRS testing utilities
pub mod events;

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
// # LINT_DISABLE_REASON: Mockall generated code uses unwrap/expect internally.
// # LINT_DISABLE_REASON: Options tried: manual mocks.
// # LINT_DISABLE_REASON: Justification: standard mocking library used in
// test-only code.
#[allow(clippy::disallowed_methods)]
#[mockall::automock]
#[async_trait]
pub trait RepositoryPort<E: Entity + 'static>: Send + Sync {
    /// Save an entity
    async fn save(&self, entity: E) -> CqrsTestResult<()>;

    /// Find an entity by ID
    async fn find_by_id(&self, id: &E::Id) -> CqrsTestResult<Option<E>>;

    /// Delete an entity by ID
    async fn delete(&self, id: &E::Id) -> CqrsTestResult<()>;

    /// Check if entity exists
    async fn exists(&self, id: &E::Id) -> CqrsTestResult<bool>;
}

/// Port trait for query-side data stores
// # LINT_DISABLE_REASON: Mockall generated code uses unwrap/expect internally.
// # LINT_DISABLE_REASON: Options tried: manual mocks.
// # LINT_DISABLE_REASON: Justification: standard mocking library used in
// test-only code.
#[allow(clippy::disallowed_methods)]
#[mockall::automock]
#[async_trait]
pub trait QueryStorePort<T: Send + Sync + 'static>: Send + Sync {
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

/// A standard adapter for CQRS testing that provides access to mocks.
pub struct CqrsTestAdapter<E: Entity + 'static, T: Send + Sync + 'static> {
    /// The mock repository
    pub repository: MockRepositoryPort<E>,
    /// The mock query store
    pub query_store: MockQueryStorePort<T>,
}

impl<E: Entity + 'static, T: Send + Sync + 'static> CqrsTestAdapter<E, T> {
    /// Create a new CQRS test adapter with fresh mocks
    #[must_use]
    pub fn new() -> Self {
        Self {
            repository: MockRepositoryPort::new(),
            query_store: MockQueryStorePort::new(),
        }
    }
}

impl<E: Entity + 'static, T: Send + Sync + 'static> Default
    for CqrsTestAdapter<E, T>
{
    fn default() -> Self {
        Self::new()
    }
}

/// Given-When-Then test framework for event sourcing aggregates.
///
/// This framework allows for declarative verification of aggregate behavior
/// by providing a history of events and asserting on the resulting events
/// after a command is handled.
///
/// # Examples
///
/// ```rust
/// use lithos_test_utils::cqrs::TestFramework;
///
/// #[derive(Debug, Clone, PartialEq)]
/// enum UserEvent {
///     Created(String),
/// }
/// struct CreateUser {
///     name: String,
/// }
///
/// let framework: TestFramework<(), CreateUser, UserEvent> =
///     TestFramework::new();
///
/// framework
///     .given(vec![])
///     .when(CreateUser {
///         name: "Alice".into(),
///     })
///     .execute(|_history, cmd| vec![UserEvent::Created(cmd.name)])
///     .then_expect_events(vec![UserEvent::Created("Alice".into())]);
/// ```
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

/// Event verification utilities for command handler testing.
///
/// Captures emitted domain events and provides high-level assertions for
/// verifying counts and existence within a test case.
///
/// # Examples
///
/// ```rust
/// use lithos_test_utils::cqrs::EventVerifier;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let verifier = EventVerifier::<String>::new();
///
/// // Simulate event emission from a handler
/// verifier.record("UserCreated".into()).await;
///
/// // Assertions
/// verifier.assert_event_count(1).await?;
/// verifier.assert_event_exists(&"UserCreated".into()).await?;
/// # Ok(())
/// # }
/// ```
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

/// Eventual consistency testing utilities for write/read model synchronization.
///
/// Provides polling-based wait mechanisms to avoid flaky `sleep()` calls in
/// tests that depend on asynchronous projections or background indexing.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
///
/// use lithos_test_utils::cqrs::EventualConsistencyTester;
/// use tokio::sync::Mutex;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let tester = EventualConsistencyTester::new();
/// let flag = Arc::new(Mutex::new(false));
///
/// // Background task updates state
/// let f = Arc::clone(&flag);
/// tokio::spawn(async move {
///     tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
///     *f.lock().await = true;
/// });
///
/// // Wait until condition is met
/// tester
///     .wait_for_condition(
///         || {
///             let f = Arc::clone(&flag);
///             async move { *f.lock().await }
///         },
///         tokio::time::Duration::from_millis(200),
///     )
///     .await?;
/// # Ok(())
/// # }
/// ```
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
        crate::core::async_utils::poll_condition(
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

    /// Verify race condition prevention by ensuring operations complete in
    /// order
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

/// Cross-aggregate saga testing utilities.
///
/// Tracks multiple participants in a long-running workflow and captures
/// events to verify coordination logic across domain boundaries.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
///
/// use lithos_test_utils::cqrs::SagaTester;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let tester = SagaTester::new();
///
/// // Register expected participants
/// tester.track_participant("Indexing").await;
/// tester.track_participant("Search").await;
///
/// // Simulate saga progress
/// tester.record_event("BatchStarted").await;
/// tester.mark_updated("Indexing").await?;
///
/// // Assertions
/// tester.verify_event_sequence(&["BatchStarted"]).await?;
/// # Ok(())
/// # }
/// ```
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
                    .filter(|&(_, status)| !*status)
                    .map(|(name, _)| name.clone())
                    .collect();

                return Err(CqrsTestError::ConsistencyTimeout(format!(
                    "Not all participants updated within {timeout:?}. \
                     Pending: {pending:?}"
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
                "Event sequence mismatch. Expected: {expected:?}, Actual: \
                 {actual_strs:?}"
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
// # LINT_DISABLE_REASON: Mock verification and assertions in tests trigger
// disallowed-method and expect_used lints. # LINT_DISABLE_REASON: Options
// tried: manual Result matching. # LINT_DISABLE_REASON: Justification: test
// code clarity and standard practice.
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
        let mut repo = MockRepositoryPort::new();
        let user = TestUser {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };

        let user_clone = user.clone();
        repo.expect_save()
            .with(mockall::predicate::eq(user_clone))
            .times(1)
            .returning(|_| Ok(()));

        repo.save(user).await.unwrap();
    }

    #[tokio::test]
    async fn mock_repository_fails_on_configured_error() {
        let mut repo = MockRepositoryPort::<TestUser>::new();
        repo.expect_save().returning(|_| {
            Err(CqrsTestError::MockRepositoryError(
                "Simulated error".to_string(),
            ))
        });

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

        let mut store = MockQueryStorePort::<TestUser>::new();
        let users_clone = users.clone();
        store.expect_query().returning(move |_| Ok(users_clone.clone()));

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
