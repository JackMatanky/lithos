//! # Lithos Test Utilities
//!
//! This crate provides standardized testing utilities and patterns for the Lithos project,
//! organized into logical modules for scalability and ease of discovery.

pub mod core;
pub mod cqrs;
pub mod data;
pub mod fs;
pub mod mocks;
pub mod obs;

// --- Top-level re-exports for convenience ---

// Core async and assertion utilities
pub use crate::core::{
    arch,
    assertions::domain,
    async_utils::{
        default_test_timeout, long_test_timeout, shared_mutex, shared_rwlock,
        shared_semaphore, short_test_timeout, spawn_blocking_test,
        with_cancellation, with_timeout,
    },
    bench,
    bench::{create_benchmark_runtime, performance_gates, standard_criterion},
    integration::{IntegrationConfig, IntegrationFixture},
};
// CQRS and Event testing
pub use crate::cqrs::{
    CqrsTestAdapter, CqrsTestError, CqrsTestResult, Entity, EventVerifier,
    EventualConsistencyTester, MockQueryStorePort, MockRepositoryPort,
    QueryCriteria, QueryStorePort, RepositoryPort, SagaTester, TestFramework,
    events::{
        EventRecord, EventTestError, EventTestFramework, EventTestResult,
        EventTestScenario, PayloadAssertion, SequenceAssertion,
        TimingAssertion,
    },
};
// Data generation and snapshot testing
pub use crate::data::{
    fixtures::{
        FakeData, Fixture, Scenario, SerializationHelper, combine, test_config,
        test_user,
    },
    snapshots::with_standard_redactions,
};
// Filesystem utilities
pub use crate::fs::{
    temp::{TempDir, TestOutput, generate_unique_name, path_utils},
    vault::TestVault,
};
// Mocks and External Systems
pub use crate::mocks::{
    auth::{
        AuthorizationAuditEntry, AuthorizationResult, InputSanitizer,
        MockAuthorizationService,
    },
    event_bus::{EventBusError, EventBusPort, EventPlane, MockEventBus},
    obs::{
        MockMetricsCollector, MockTraceCollector, OperationStats, TraceEntry,
    },
};

// The async_test macro is automatically exported at crate root via #[macro_export]
// in the async_utils module.
