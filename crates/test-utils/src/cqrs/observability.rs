//! # CQRS Observability Testing Utilities
//!
//! This module provides testing utilities for CQRS-specific observability patterns,
//! including event tracing, command/query metrics validation, and execution correlation.
//!
//! ## Architecture Compliance
//!
//! Implements ADR 0009 Decision 6: Observability testing patterns for CQRS-specific
//! tracing (event publishing, command/query execution metrics).

use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::RwLock;

use crate::cqrs::{CqrsTestError, CqrsTestResult};

/// Mock metrics collector for CQRS operations
///
/// # Architecture Compliance
/// Tracks command/query execution metrics for testing observability patterns.
///
/// # Usage
/// ```rust
/// # use lithos_test_utils::MockMetricsCollector;
/// # use std::time::Duration;
/// # #[tokio::main]
/// # async fn main() {
/// let metrics = MockMetricsCollector::new();
///
/// metrics.record_command("CreateOrder", Duration::from_millis(50), true).await;
///
/// let stats = metrics.command_stats("CreateOrder").await;
/// assert_eq!(stats.total_calls, 1);
/// assert_eq!(stats.success_count, 1);
/// # }
/// ```
pub struct MockMetricsCollector {
    /// Command metrics: command name -> statistics
    command_metrics: Arc<RwLock<HashMap<String, OperationStats>>>,
    /// Query metrics: query name -> statistics
    query_metrics: Arc<RwLock<HashMap<String, OperationStats>>>,
    /// Event metrics: event type -> count
    event_metrics: Arc<RwLock<HashMap<String, usize>>>,
}

/// Statistics for a CQRS operation
#[derive(Debug, Clone, Default)]
pub struct OperationStats {
    /// Total number of calls
    pub total_calls: usize,
    /// Number of successful calls
    pub success_count: usize,
    /// Number of failed calls
    pub failure_count: usize,
    /// Total duration of all calls
    pub total_duration: Duration,
    /// Minimum duration
    pub min_duration: Option<Duration>,
    /// Maximum duration
    pub max_duration: Option<Duration>,
}

impl OperationStats {
    /// Calculate average duration
    #[must_use]
    pub fn avg_duration(&self) -> Option<Duration> {
        if self.total_calls > 0 {
            Some(self.total_duration / self.total_calls.try_into().ok()?)
        } else {
            None
        }
    }

    /// Get success rate as percentage
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_calls > 0 {
            (self.success_count as f64 / self.total_calls as f64) * 100.0
        } else {
            0.0
        }
    }
}

impl MockMetricsCollector {
    /// Create a new metrics collector
    #[must_use]
    pub fn new() -> Self {
        Self {
            command_metrics: Arc::new(RwLock::new(HashMap::new())),
            query_metrics: Arc::new(RwLock::new(HashMap::new())),
            event_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a command execution
    pub async fn record_command(
        &self,
        command: impl Into<String>,
        duration: Duration,
        success: bool,
    ) {
        let command = command.into();
        let mut metrics = self.command_metrics.write().await;
        let stats = metrics.entry(command).or_default();

        stats.total_calls += 1;
        if success {
            stats.success_count += 1;
        } else {
            stats.failure_count += 1;
        }

        stats.total_duration += duration;
        stats.min_duration =
            Some(stats.min_duration.map_or(duration, |min| min.min(duration)));
        stats.max_duration =
            Some(stats.max_duration.map_or(duration, |max| max.max(duration)));
    }

    /// Record a query execution
    pub async fn record_query(
        &self,
        query: impl Into<String>,
        duration: Duration,
        success: bool,
    ) {
        let query = query.into();
        let mut metrics = self.query_metrics.write().await;
        let stats = metrics.entry(query).or_default();

        stats.total_calls += 1;
        if success {
            stats.success_count += 1;
        } else {
            stats.failure_count += 1;
        }

        stats.total_duration += duration;
        stats.min_duration =
            Some(stats.min_duration.map_or(duration, |min| min.min(duration)));
        stats.max_duration =
            Some(stats.max_duration.map_or(duration, |max| max.max(duration)));
    }

    /// Record an event publication
    pub async fn record_event(&self, event_type: impl Into<String>) {
        let event_type = event_type.into();
        let mut metrics = self.event_metrics.write().await;
        *metrics.entry(event_type).or_insert(0) += 1;
    }

    /// Get statistics for a command
    pub async fn command_stats(&self, command: &str) -> OperationStats {
        self.command_metrics
            .read()
            .await
            .get(command)
            .cloned()
            .unwrap_or_default()
    }

    /// Get statistics for a query
    pub async fn query_stats(&self, query: &str) -> OperationStats {
        self.query_metrics.read().await.get(query).cloned().unwrap_or_default()
    }

    /// Get event publication count
    pub async fn event_count(&self, event_type: &str) -> usize {
        self.event_metrics.read().await.get(event_type).copied().unwrap_or(0)
    }

    /// Get all command names with metrics
    pub async fn all_commands(&self) -> Vec<String> {
        self.command_metrics.read().await.keys().cloned().collect()
    }

    /// Get all query names with metrics
    pub async fn all_queries(&self) -> Vec<String> {
        self.query_metrics.read().await.keys().cloned().collect()
    }

    /// Get all event types with metrics
    pub async fn all_events(&self) -> Vec<String> {
        self.event_metrics.read().await.keys().cloned().collect()
    }

    /// Clear all metrics
    pub async fn clear(&self) {
        self.command_metrics.write().await.clear();
        self.query_metrics.write().await.clear();
        self.event_metrics.write().await.clear();
    }
}

impl Default for MockMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock trace collector for CQRS operations
///
/// # Architecture Compliance
/// Tracks execution traces for correlation testing across command/query boundaries.
pub struct MockTraceCollector {
    /// Trace entries: correlation_id -> trace details
    traces: Arc<RwLock<HashMap<String, Vec<TraceEntry>>>>,
}

/// A single trace entry
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// Operation type (Command, Query, Event)
    pub operation_type: String,
    /// Operation name
    pub operation_name: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Duration
    pub duration: Option<Duration>,
    /// Additional context
    pub context: HashMap<String, String>,
}

impl MockTraceCollector {
    /// Create a new trace collector
    #[must_use]
    pub fn new() -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new trace
    pub async fn start_trace(&self, correlation_id: impl Into<String>) {
        let correlation_id = correlation_id.into();
        self.traces.write().await.insert(correlation_id, Vec::new());
    }

    /// Add an entry to a trace
    pub async fn add_entry(
        &self,
        correlation_id: &str,
        operation_type: impl Into<String>,
        operation_name: impl Into<String>,
        duration: Option<Duration>,
        context: HashMap<String, String>,
    ) {
        let entry = TraceEntry {
            operation_type: operation_type.into(),
            operation_name: operation_name.into(),
            timestamp: chrono::Utc::now(),
            duration,
            context,
        };

        if let Some(entries) = self.traces.write().await.get_mut(correlation_id)
        {
            entries.push(entry);
        }
    }

    /// Get trace entries for a correlation ID
    pub async fn get_trace(
        &self,
        correlation_id: &str,
    ) -> Option<Vec<TraceEntry>> {
        self.traces.read().await.get(correlation_id).cloned()
    }

    /// Verify that a command led to expected events
    ///
    /// # Errors
    /// Returns error if expected event sequence not found
    pub async fn verify_command_event_flow(
        &self,
        correlation_id: &str,
        expected_events: &[&str],
    ) -> CqrsTestResult<()> {
        let traces = self.traces.read().await;
        let entries = traces.get(correlation_id).ok_or_else(|| {
            CqrsTestError::TestError(format!(
                "No trace found for {correlation_id}"
            ))
        })?;

        let actual_events: Vec<&str> = entries
            .iter()
            .filter(|e| e.operation_type == "Event")
            .map(|e| e.operation_name.as_str())
            .collect();

        if actual_events == expected_events {
            Ok(())
        } else {
            Err(CqrsTestError::TestError(format!(
                "Event flow mismatch. Expected: {:?}, Actual: {:?}",
                expected_events, actual_events
            )))
        }
    }

    /// Clear all traces
    pub async fn clear(&self) {
        self.traces.write().await.clear();
    }
}

impl Default for MockTraceCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn metrics_collector_records_commands() {
        let metrics = MockMetricsCollector::new();

        metrics
            .record_command("CreateOrder", Duration::from_millis(50), true)
            .await;
        metrics
            .record_command("CreateOrder", Duration::from_millis(75), true)
            .await;
        metrics
            .record_command("CreateOrder", Duration::from_millis(60), false)
            .await;

        let stats = metrics.command_stats("CreateOrder").await;
        assert_eq!(stats.total_calls, 3);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.failure_count, 1);
        assert_eq!(stats.min_duration, Some(Duration::from_millis(50)));
        assert_eq!(stats.max_duration, Some(Duration::from_millis(75)));
    }

    #[tokio::test]
    async fn metrics_collector_calculates_avg_duration() {
        let metrics = MockMetricsCollector::new();

        metrics
            .record_command("CreateOrder", Duration::from_millis(50), true)
            .await;
        metrics
            .record_command("CreateOrder", Duration::from_millis(100), true)
            .await;

        let stats = metrics.command_stats("CreateOrder").await;
        assert_eq!(stats.avg_duration(), Some(Duration::from_millis(75)));
    }

    #[tokio::test]
    async fn metrics_collector_tracks_events() {
        let metrics = MockMetricsCollector::new();

        metrics.record_event("OrderCreated").await;
        metrics.record_event("OrderCreated").await;
        metrics.record_event("OrderShipped").await;

        assert_eq!(metrics.event_count("OrderCreated").await, 2);
        assert_eq!(metrics.event_count("OrderShipped").await, 1);
    }

    #[tokio::test]
    async fn trace_collector_records_traces() {
        let collector = MockTraceCollector::new();

        collector.start_trace("trace-1").await;
        collector
            .add_entry(
                "trace-1",
                "Command",
                "CreateOrder",
                Some(Duration::from_millis(50)),
                HashMap::new(),
            )
            .await;
        collector
            .add_entry("trace-1", "Event", "OrderCreated", None, HashMap::new())
            .await;

        let trace = collector.get_trace("trace-1").await.unwrap();
        assert_eq!(trace.len(), 2);
        assert_eq!(trace[0].operation_type, "Command");
        assert_eq!(trace[1].operation_type, "Event");
    }

    #[tokio::test]
    async fn trace_collector_verifies_event_flow() {
        let collector = MockTraceCollector::new();

        collector.start_trace("trace-1").await;
        collector
            .add_entry(
                "trace-1",
                "Command",
                "CreateOrder",
                Some(Duration::from_millis(50)),
                HashMap::new(),
            )
            .await;
        collector
            .add_entry("trace-1", "Event", "OrderCreated", None, HashMap::new())
            .await;
        collector
            .add_entry("trace-1", "Event", "OrderShipped", None, HashMap::new())
            .await;

        let result = collector
            .verify_command_event_flow(
                "trace-1",
                &["OrderCreated", "OrderShipped"],
            )
            .await;
        assert!(result.is_ok());

        let wrong_result = collector
            .verify_command_event_flow(
                "trace-1",
                &["OrderShipped", "OrderCreated"],
            )
            .await;
        assert!(wrong_result.is_err());
    }
}
