//! # Command Handler Testing Examples
//!
//! Demonstrates ADR 0009 Decision 1: Command handler testing with mock repositories,
//! event verification, validation logic, and error scenario testing.

#![allow(clippy::disallowed_methods)]

use std::sync::Arc;

use lithos_test_utils::cqrs::{
    CqrsTestResult, Entity, EventVerifier, MockRepository, RepositoryPort,
};

// ============================================================================
// Example Domain Types for Command Testing
// ============================================================================

/// Example user entity for command testing
#[derive(Debug, Clone, PartialEq, Eq)]
struct User {
    id: String,
    email: String,
    name: String,
}

impl Entity for User {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

/// Example command to create a user
#[derive(Debug, Clone)]
struct CreateUserCommand {
    id: String,
    email: String,
    name: String,
}

/// Example domain event for user creation
#[derive(Debug, Clone, PartialEq, Eq)]
enum UserEvent {
    UserCreated {
        id: String,
        email: String,
        name: String,
    },
}

/// Example command handler for user creation
struct CreateUserHandler {
    repository: Arc<dyn RepositoryPort<User>>,
    event_verifier: Arc<EventVerifier<UserEvent>>,
}

impl CreateUserHandler {
    fn new(
        repository: Arc<dyn RepositoryPort<User>>,
        event_verifier: Arc<EventVerifier<UserEvent>>,
    ) -> Self {
        Self {
            repository,
            event_verifier,
        }
    }

    async fn handle(&self, command: CreateUserCommand) -> CqrsTestResult<()> {
        // Validate command
        if command.email.is_empty() {
            return Err(lithos_test_utils::cqrs::CqrsTestError::TestError(
                "Email cannot be empty".to_string(),
            ));
        }

        // Create user entity
        let user = User {
            id: command.id.clone(),
            email: command.email.clone(),
            name: command.name.clone(),
        };

        // Save to repository
        self.repository.save(user.clone()).await?;

        // Publish domain event
        let event = UserEvent::UserCreated {
            id: command.id,
            email: command.email,
            name: command.name,
        };
        self.event_verifier.record(event).await;

        Ok(())
    }
}

// ============================================================================
// Tests: Command Handler Testing Patterns (ADR 0009 Decision 1)
// ============================================================================

#[tokio::test]
async fn command_handler_saves_entity_to_repository() {
    // Arrange
    let mock_repo = Arc::new(MockRepository::new());
    let event_verifier = Arc::new(EventVerifier::new());
    let handler = CreateUserHandler::new(mock_repo.clone(), event_verifier);

    // Act
    let command = CreateUserCommand {
        id: "user-1".to_string(),
        email: "alice@example.com".to_string(),
        name: "Alice".to_string(),
    };
    handler.handle(command).await.unwrap();

    // Assert
    assert_eq!(mock_repo.save_count().await, 1);
    let saved_user = mock_repo.find_by_id(&"user-1".to_string()).await.unwrap();
    assert!(saved_user.is_some());
    assert_eq!(saved_user.unwrap().email, "alice@example.com");
}

#[tokio::test]
async fn command_handler_publishes_domain_event() {
    // Arrange
    let mock_repo = Arc::new(MockRepository::new());
    let event_verifier = Arc::new(EventVerifier::new());
    let handler = CreateUserHandler::new(mock_repo, event_verifier.clone());

    // Act
    let command = CreateUserCommand {
        id: "user-2".to_string(),
        email: "bob@example.com".to_string(),
        name: "Bob".to_string(),
    };
    handler.handle(command).await.unwrap();

    // Assert
    event_verifier.assert_event_count(1).await.unwrap();
    let expected_event = UserEvent::UserCreated {
        id: "user-2".to_string(),
        email: "bob@example.com".to_string(),
        name: "Bob".to_string(),
    };
    event_verifier.assert_event_exists(&expected_event).await.unwrap();
}

#[tokio::test]
async fn command_handler_validates_input() {
    // Arrange
    let mock_repo = Arc::new(MockRepository::new());
    let event_verifier = Arc::new(EventVerifier::new());
    let handler =
        CreateUserHandler::new(mock_repo.clone(), event_verifier.clone());

    // Act
    let command = CreateUserCommand {
        id: "user-3".to_string(),
        email: "".to_string(), // Invalid: empty email
        name: "Invalid User".to_string(),
    };
    let result = handler.handle(command).await;

    // Assert
    assert!(result.is_err());
    assert_eq!(mock_repo.save_count().await, 0); // Nothing saved
    event_verifier.assert_event_count(0).await.unwrap(); // No events published
}

#[tokio::test]
async fn command_handler_handles_repository_failure() {
    // Arrange
    let mock_repo = Arc::new(MockRepository::new());
    mock_repo.fail_next_save("Database connection failed").await;
    let event_verifier = Arc::new(EventVerifier::new());
    let handler =
        CreateUserHandler::new(mock_repo.clone(), event_verifier.clone());

    // Act
    let command = CreateUserCommand {
        id: "user-4".to_string(),
        email: "charlie@example.com".to_string(),
        name: "Charlie".to_string(),
    };
    let result = handler.handle(command).await;

    // Assert
    assert!(result.is_err());
    // Verify no events were published when save failed
    event_verifier.assert_event_count(0).await.unwrap();
}

#[tokio::test]
async fn command_handler_records_all_interactions() {
    // Arrange
    let mock_repo = Arc::new(MockRepository::new());
    let event_verifier = Arc::new(EventVerifier::new());
    let handler = CreateUserHandler::new(mock_repo.clone(), event_verifier);

    // Act
    for i in 1..=3 {
        let command = CreateUserCommand {
            id: format!("user-{i}"),
            email: format!("user{i}@example.com"),
            name: format!("User {i}"),
        };
        handler.handle(command).await.unwrap();
    }

    // Assert
    assert_eq!(mock_repo.save_count().await, 3);
    let interactions = mock_repo.interactions().await;
    assert_eq!(interactions.len(), 3);
}
