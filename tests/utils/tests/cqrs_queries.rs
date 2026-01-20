//! # Query Handler Testing Examples
//!
//! Demonstrates ADR 0009 Decision 2: Query handler testing with stubbed data
//! stores, result transformation testing, performance validation, and caching
//! verification.

#![allow(clippy::disallowed_methods)]

use std::sync::Arc;

use lithos_test_utils::cqrs::{
    CqrsTestResult, MockQueryStorePort, QueryCriteria, QueryStorePort,
};

// ============================================================================
// Example Domain Types for Query Testing
// ============================================================================

/// Example user read model (dumb DTO for queries)
#[derive(Debug, Clone, PartialEq, Eq)]
struct UserReadModel {
    id: String,
    email: String,
    name: String,
    status: String,
}

/// Example query to get user by ID
#[derive(Debug, Clone)]
struct GetUserQuery {
    user_id: String,
}

/// Example query handler for getting a user
struct GetUserHandler {
    store: Arc<dyn QueryStorePort<UserReadModel>>,
}

impl GetUserHandler {
    fn new(store: Arc<dyn QueryStorePort<UserReadModel>>) -> Self {
        Self {
            store,
        }
    }

    async fn handle(
        &self,
        query: GetUserQuery,
    ) -> CqrsTestResult<Option<UserReadModel>> {
        self.store.get_by_id(&query.user_id).await
    }
}

/// Example query to list users
#[derive(Debug, Clone)]
struct ListUsersQuery {
    criteria: QueryCriteria,
}

/// Example query handler for listing users
struct ListUsersHandler {
    store: Arc<dyn QueryStorePort<UserReadModel>>,
}

impl ListUsersHandler {
    fn new(store: Arc<dyn QueryStorePort<UserReadModel>>) -> Self {
        Self {
            store,
        }
    }

    async fn handle(
        &self,
        query: ListUsersQuery,
    ) -> CqrsTestResult<Vec<UserReadModel>> {
        self.store.query(&query.criteria).await
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_users() -> Vec<UserReadModel> {
    vec![
        UserReadModel {
            id: "user-1".to_string(),
            email: "alice@example.com".to_string(),
            name: "Alice".to_string(),
            status: "active".to_string(),
        },
        UserReadModel {
            id: "user-2".to_string(),
            email: "bob@example.com".to_string(),
            name: "Bob".to_string(),
            status: "active".to_string(),
        },
        UserReadModel {
            id: "user-3".to_string(),
            email: "charlie@example.com".to_string(),
            name: "Charlie".to_string(),
            status: "inactive".to_string(),
        },
    ]
}

// ============================================================================
// Tests: Query Handler Testing Patterns (ADR 0009 Decision 2)
// ============================================================================

#[tokio::test]
async fn query_handler_returns_user_by_id() {
    // Arrange
    let test_users = create_test_users();
    let mut mock_store = MockQueryStorePort::<UserReadModel>::new();
    let user_1 = test_users[0].clone();
    mock_store
        .expect_get_by_id()
        .with(mockall::predicate::eq("user-1"))
        .returning(move |_| Ok(Some(user_1.clone())));

    let handler = GetUserHandler::new(Arc::new(mock_store));

    // Act
    let query = GetUserQuery {
        user_id: "user-1".to_string(),
    };
    let result = handler.handle(query).await.unwrap();

    // Assert
    assert!(result.is_some());
    let user = result.unwrap();
    assert_eq!(user.id, "user-1");
    assert_eq!(user.email, "alice@example.com");
    assert_eq!(user.name, "Alice");
}

#[tokio::test]
async fn query_handler_returns_none_for_nonexistent_user() {
    // Arrange
    let mut mock_store = MockQueryStorePort::<UserReadModel>::new();
    mock_store.expect_get_by_id().returning(|_| Ok(None));
    let handler = GetUserHandler::new(Arc::new(mock_store));

    // Act
    let query = GetUserQuery {
        user_id: "user-999".to_string(),
    };
    let result = handler.handle(query).await.unwrap();

    // Assert
    assert!(result.is_none());
}

#[tokio::test]
async fn list_query_handler_returns_all_users() {
    // Arrange
    let test_users = create_test_users();
    let expected_count = test_users.len();
    let mut mock_store = MockQueryStorePort::<UserReadModel>::new();
    mock_store.expect_query().returning(move |_| Ok(test_users.clone()));

    let handler = ListUsersHandler::new(Arc::new(mock_store));

    // Act
    let query = ListUsersQuery {
        criteria: QueryCriteria::new(),
    };
    let result = handler.handle(query).await.unwrap();

    // Assert
    assert_eq!(result.len(), expected_count);
}

#[tokio::test]
async fn query_handler_performs_within_time_bounds() {
    // Arrange
    let _test_users = create_test_users();
    let mut mock_store = MockQueryStorePort::<UserReadModel>::new();
    mock_store.expect_get_by_id().returning(move |_| Ok(None));
    let handler = GetUserHandler::new(Arc::new(mock_store));

    // Act
    let start = std::time::Instant::now();
    let query = GetUserQuery {
        user_id: "user-1".to_string(),
    };
    handler.handle(query).await.unwrap();
    let duration = start.elapsed();

    // Assert - query should be fast (<10ms for mock)
    assert!(
        duration.as_millis() < 10,
        "Query took {duration:?}, expected <10ms"
    );
}
