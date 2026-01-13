//! Mock implementations for testing.

pub mod auth;
pub mod event_bus;
pub mod obs;

pub use auth::{
    AuthorizationAuditEntry, AuthorizationResult, MockAuthorizationService,
};
pub use event_bus::{EventBusError, EventBusPort, EventPlane, MockEventBus};
pub use obs::{
    MockMetricsCollector, MockTraceCollector, OperationStats, TraceEntry,
};
