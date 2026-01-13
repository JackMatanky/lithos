//! # CQRS Security Testing Utilities
//!
//! This module provides testing utilities for CQRS-specific security patterns,
//! including command/query authorization testing, access control verification,
//! and audit trail validation.
//!
//! ## Architecture Compliance
//!
//! Implements ADR 0009 Decision 6: Security testing patterns for CQRS-specific
//! authorization (command execution rights, query access control).

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::cqrs::{CqrsTestError, CqrsTestResult};

/// Authorization result for CQRS operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationResult {
    /// Operation is allowed
    Allowed,
    /// Operation is denied
    Denied(String),
}

/// Mock authorization service for testing CQRS security patterns
///
/// # Architecture Compliance
/// Implements command/query access control testing from ADR 0009.
/// Allows testing authorization failures and audit trail generation.
///
/// # Usage
/// ```rust
/// # use lithos_test_utils::MockAuthorizationService;
/// # #[tokio::main]
/// # async fn main() {
/// let auth = MockAuthorizationService::new();
/// auth.grant_permission("user1", "CreateOrder").await;
///
/// assert!(auth.check_command("user1", "CreateOrder").await.is_ok());
/// assert!(auth.check_command("user1", "DeleteOrder").await.is_err());
/// # }
/// ```
pub struct MockAuthorizationService {
    /// User permissions: (user_id, operation) -> allowed
    permissions: Arc<RwLock<HashMap<(String, String), bool>>>,
    /// Audit trail of authorization checks
    audit_trail: Arc<RwLock<Vec<AuthorizationAuditEntry>>>,
}

/// Entry in the authorization audit trail
#[derive(Debug, Clone)]
pub struct AuthorizationAuditEntry {
    /// User ID that requested the operation
    pub user_id: String,
    /// Operation that was requested
    pub operation: String,
    /// Whether the operation was allowed
    pub result: AuthorizationResult,
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MockAuthorizationService {
    /// Create a new mock authorization service
    #[must_use]
    pub fn new() -> Self {
        Self {
            permissions: Arc::new(RwLock::new(HashMap::new())),
            audit_trail: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Grant permission for a user to perform an operation
    pub async fn grant_permission(
        &self,
        user_id: impl Into<String>,
        operation: impl Into<String>,
    ) {
        self.permissions
            .write()
            .await
            .insert((user_id.into(), operation.into()), true);
    }

    /// Revoke permission for a user to perform an operation
    pub async fn revoke_permission(
        &self,
        user_id: impl Into<String>,
        operation: impl Into<String>,
    ) {
        self.permissions
            .write()
            .await
            .insert((user_id.into(), operation.into()), false);
    }

    /// Check if a command is authorized for a user
    ///
    /// # Errors
    /// Returns error if command is not authorized
    pub async fn check_command(
        &self,
        user_id: &str,
        command: &str,
    ) -> CqrsTestResult<()> {
        let key = (user_id.to_string(), command.to_string());
        let permissions = self.permissions.read().await;
        let allowed = permissions.get(&key).copied().unwrap_or(false);

        let result = if allowed {
            AuthorizationResult::Allowed
        } else {
            AuthorizationResult::Denied(format!(
                "User {user_id} not authorized for command {command}"
            ))
        };

        // Record in audit trail
        self.audit_trail.write().await.push(AuthorizationAuditEntry {
            user_id: user_id.to_string(),
            operation: command.to_string(),
            result: result.clone(),
            timestamp: chrono::Utc::now(),
        });

        match result {
            AuthorizationResult::Allowed => Ok(()),
            AuthorizationResult::Denied(msg) => Err(CqrsTestError::TestError(
                format!("Authorization denied: {msg}"),
            )),
        }
    }

    /// Check if a query is authorized for a user
    ///
    /// # Errors
    /// Returns error if query is not authorized
    pub async fn check_query(
        &self,
        user_id: &str,
        query: &str,
    ) -> CqrsTestResult<()> {
        let key = (user_id.to_string(), query.to_string());
        let permissions = self.permissions.read().await;
        let allowed = permissions.get(&key).copied().unwrap_or(false);

        let result = if allowed {
            AuthorizationResult::Allowed
        } else {
            AuthorizationResult::Denied(format!(
                "User {user_id} not authorized for query {query}"
            ))
        };

        // Record in audit trail
        self.audit_trail.write().await.push(AuthorizationAuditEntry {
            user_id: user_id.to_string(),
            operation: query.to_string(),
            result: result.clone(),
            timestamp: chrono::Utc::now(),
        });

        match result {
            AuthorizationResult::Allowed => Ok(()),
            AuthorizationResult::Denied(msg) => Err(CqrsTestError::TestError(
                format!("Authorization denied: {msg}"),
            )),
        }
    }

    /// Get the audit trail
    pub async fn audit_trail(&self) -> Vec<AuthorizationAuditEntry> {
        self.audit_trail.read().await.clone()
    }

    /// Get authorization attempts for a specific user
    pub async fn audit_for_user(
        &self,
        user_id: &str,
    ) -> Vec<AuthorizationAuditEntry> {
        self.audit_trail
            .read()
            .await
            .iter()
            .filter(|entry| entry.user_id == user_id)
            .cloned()
            .collect()
    }

    /// Count authorization denials
    pub async fn denial_count(&self) -> usize {
        self.audit_trail
            .read()
            .await
            .iter()
            .filter(|entry| {
                matches!(entry.result, AuthorizationResult::Denied(_))
            })
            .count()
    }

    /// Clear audit trail
    pub async fn clear_audit(&self) {
        self.audit_trail.write().await.clear();
    }
}

impl Default for MockAuthorizationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Input sanitization testing utilities
///
/// # Architecture Compliance
/// Tests input validation and sanitization for CQRS commands and queries.
pub struct InputSanitizer;

impl InputSanitizer {
    /// Check if input contains malicious patterns
    ///
    /// # Errors
    /// Returns error if malicious pattern detected
    pub fn validate_input(input: &str) -> CqrsTestResult<()> {
        // Check for SQL injection patterns
        if input.to_lowercase().contains("drop table")
            || input.to_lowercase().contains("delete from")
            || input.to_lowercase().contains("'; --")
        {
            return Err(CqrsTestError::TestError(
                "Potential SQL injection detected".to_string(),
            ));
        }

        // Check for XSS patterns
        if input.contains("<script>")
            || input.contains("javascript:")
            || input.contains("onerror=")
        {
            return Err(CqrsTestError::TestError(
                "Potential XSS attack detected".to_string(),
            ));
        }

        // Check for path traversal
        if input.contains("../") || input.contains("..\\") {
            return Err(CqrsTestError::TestError(
                "Potential path traversal detected".to_string(),
            ));
        }

        Ok(())
    }

    /// Sanitize input by removing malicious content
    #[must_use]
    pub fn sanitize(input: &str) -> String {
        input
            .replace("<script>", "")
            .replace("</script>", "")
            .replace("javascript:", "")
            .replace("../", "")
            .replace("..\\", "")
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_auth_grants_and_checks_permissions() {
        let auth = MockAuthorizationService::new();

        auth.grant_permission("user1", "CreateOrder").await;
        assert!(auth.check_command("user1", "CreateOrder").await.is_ok());
        assert!(auth.check_command("user1", "DeleteOrder").await.is_err());
    }

    #[tokio::test]
    async fn mock_auth_records_audit_trail() {
        let auth = MockAuthorizationService::new();

        auth.grant_permission("user1", "CreateOrder").await;
        auth.check_command("user1", "CreateOrder").await.ok();
        auth.check_command("user1", "DeleteOrder").await.ok();

        let trail = auth.audit_trail().await;
        assert_eq!(trail.len(), 2);
        assert_eq!(trail[0].user_id, "user1");
        assert_eq!(trail[0].operation, "CreateOrder");
        assert!(matches!(trail[0].result, AuthorizationResult::Allowed));
    }

    #[tokio::test]
    async fn mock_auth_counts_denials() {
        let auth = MockAuthorizationService::new();

        auth.grant_permission("user1", "CreateOrder").await;
        auth.check_command("user1", "CreateOrder").await.ok();
        auth.check_command("user1", "DeleteOrder").await.ok();
        auth.check_command("user1", "UpdateOrder").await.ok();

        assert_eq!(auth.denial_count().await, 2);
    }

    #[tokio::test]
    async fn mock_auth_filters_audit_by_user() {
        let auth = MockAuthorizationService::new();

        auth.grant_permission("user1", "CreateOrder").await;
        auth.grant_permission("user2", "CreateOrder").await;

        auth.check_command("user1", "CreateOrder").await.ok();
        auth.check_command("user2", "CreateOrder").await.ok();
        auth.check_command("user1", "DeleteOrder").await.ok();

        let user1_trail = auth.audit_for_user("user1").await;
        assert_eq!(user1_trail.len(), 2);
        assert!(user1_trail.iter().all(|e| e.user_id == "user1"));
    }

    #[test]
    fn input_sanitizer_detects_sql_injection() {
        let result = InputSanitizer::validate_input("'; DROP TABLE users; --");
        assert!(result.is_err());
    }

    #[test]
    fn input_sanitizer_detects_xss() {
        let result =
            InputSanitizer::validate_input("<script>alert('xss')</script>");
        assert!(result.is_err());
    }

    #[test]
    fn input_sanitizer_detects_path_traversal() {
        let result = InputSanitizer::validate_input("../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn input_sanitizer_sanitizes_input() {
        let sanitized =
            InputSanitizer::sanitize("<script>alert('xss')</script>../file");
        assert!(!sanitized.contains("<script>"));
        assert!(!sanitized.contains("../"));
    }
}
