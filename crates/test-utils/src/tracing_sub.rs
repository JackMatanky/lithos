//! Real observability testing using a custom tracing subscriber.
//!
//! This module provides a `TestTracingSubscriber` that captures actual `tracing` events
//! and spans emitted by production code, allowing for assertions on logs and traces.

use std::sync::{Arc, Mutex};

use tracing::{Event, Id, Subscriber};
use tracing_subscriber::{
    Layer, layer::Context, prelude::*, registry::LookupSpan,
};

/// A recorded tracing event.
#[derive(Debug, Clone)]
pub struct CapturedEvent {
    pub message: String,
    pub target: String,
    pub level: tracing::Level,
    pub fields: std::collections::HashMap<String, String>,
}

/// A recorded tracing span.
#[derive(Debug, Clone)]
pub struct CapturedSpan {
    pub name: String,
    pub target: String,
    pub fields: std::collections::HashMap<String, String>,
}

#[derive(Default)]
struct SharedState {
    events: Vec<CapturedEvent>,
    spans: Vec<CapturedSpan>,
}

/// A layer that captures tracing events and spans for testing.
pub struct TestLayer {
    state: Arc<Mutex<SharedState>>,
}

impl<S> Layer<S> for TestLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut fields = std::collections::HashMap::new();
        let mut visitor = FieldVisitor {
            fields: &mut fields,
        };
        event.record(&mut visitor);

        let captured = CapturedEvent {
            message: fields.get("message").cloned().unwrap_or_default(),
            target: event.metadata().target().to_string(),
            level: *event.metadata().level(),
            fields,
        };

        if let Ok(mut state) = self.state.lock() {
            state.events.push(captured);
        }
    }

    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        _id: &Id,
        _ctx: Context<'_, S>,
    ) {
        let mut fields = std::collections::HashMap::new();
        let mut visitor = FieldVisitor {
            fields: &mut fields,
        };
        attrs.record(&mut visitor);

        let captured = CapturedSpan {
            name: attrs.metadata().name().to_string(),
            target: attrs.metadata().target().to_string(),
            fields,
        };

        if let Ok(mut state) = self.state.lock() {
            state.spans.push(captured);
        }
    }
}

struct FieldVisitor<'a> {
    fields: &'a mut std::collections::HashMap<String, String>,
}

impl<'a> tracing::field::Visit for FieldVisitor<'a> {
    fn record_debug(
        &mut self,
        field: &tracing::field::Field,
        value: &dyn std::fmt::Debug,
    ) {
        self.fields.insert(field.name().to_string(), format!("{:?}", value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.fields.insert(field.name().to_string(), value.to_string());
    }
}

/// Handle to captured tracing data.
pub struct TracingHandle {
    state: Arc<Mutex<SharedState>>,
    _guard: tracing::subscriber::DefaultGuard,
}

impl TracingHandle {
    /// Returns all captured events.
    // # LINT_DISABLE_REASON: Mutex poisoning in tests is handled by unwrap as it signifies a fatal test error.
    // # LINT_DISABLE_REASON: Options tried: manual error handling.
    // # LINT_DISABLE_REASON: Justification: simplify test assertion code.
    #[allow(clippy::disallowed_methods)]
    pub fn events(&self) -> Vec<CapturedEvent> {
        self.state.lock().unwrap().events.clone()
    }

    /// Returns all captured spans.
    // # LINT_DISABLE_REASON: Mutex poisoning in tests is handled by unwrap as it signifies a fatal test error.
    // # LINT_DISABLE_REASON: Options tried: manual error handling.
    // # LINT_DISABLE_REASON: Justification: simplify test assertion code.
    #[allow(clippy::disallowed_methods)]
    pub fn spans(&self) -> Vec<CapturedSpan> {
        self.state.lock().unwrap().spans.clone()
    }

    /// Asserts that an event with the given message was logged.
    pub fn assert_logged(&self, message: &str) {
        let events = self.events();
        if !events.iter().any(|e| e.message.contains(message)) {
            panic!(
                "Expected log message '{}' not found in captured events: {:?}",
                message, events
            );
        }
    }

    /// Asserts that a span with the given name was created.
    pub fn assert_span_created(&self, name: &str) {
        let spans = self.spans();
        if !spans.iter().any(|s| s.name == name) {
            panic!(
                "Expected span '{}' not created. Found spans: {:?}",
                name, spans
            );
        }
    }
}

/// Initializes a tracing subscriber for the current test.
///
/// Returns a `TracingHandle` that can be used to query captured events and spans.
/// The subscriber is automatically uninstalled when the handle is dropped.
pub fn init_tracing() -> TracingHandle {
    let state = Arc::new(Mutex::new(SharedState::default()));
    let layer = TestLayer {
        state: Arc::clone(&state),
    };

    let subscriber = tracing_subscriber::registry().with(layer);
    let guard = tracing::subscriber::set_default(subscriber);

    TracingHandle {
        state,
        _guard: guard,
    }
}

#[cfg(test)]
mod tests {
    use tracing::{Level, info, span};

    use super::*;

    #[test]
    fn test_tracing_capture() {
        let handle = init_tracing();

        info!("Hello world");
        let _span = span!(Level::INFO, "test_span").entered();

        handle.assert_logged("Hello world");
        handle.assert_span_created("test_span");
    }
}
