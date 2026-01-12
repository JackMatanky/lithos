//! Mock event bus implementations for async testing.

use std::{
    error::Error,
    fmt,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use async_trait::async_trait;
use tokio::sync::{Mutex, broadcast, mpsc, watch};

use crate::events::{EventRecord, SequenceAssertion};

/// Event bus planes defined by ADR 0007.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPlane {
    /// Reliable data plane using MPSC channels.
    Data,
    /// Broadcast control plane for system-wide signals.
    Control,
    /// Watch-based state plane for latest state notifications.
    State,
}

/// Errors produced by the mock event bus.
#[derive(Debug, Clone)]
pub struct EventBusError {
    message: String,
}

impl EventBusError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn data_plane_closed() -> Self {
        Self::new("Data plane channel closed")
    }

    fn control_plane_closed() -> Self {
        Self::new("Control plane channel closed")
    }

    fn state_plane_closed() -> Self {
        Self::new("State plane channel closed")
    }

    fn data_subscription_taken() -> Self {
        Self::new("Data plane receiver already taken")
    }
}

impl fmt::Display for EventBusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for EventBusError {}

/// Trait for event bus ports used in tests.
#[async_trait]
pub trait EventBusPort<T>: Send + Sync
where
    T: Clone + Send + Sync + 'static,
{
    /// Publish an event to the data plane.
    async fn publish_data(&self, event: T) -> Result<(), EventBusError>;
    /// Publish an event to the control plane.
    async fn publish_control(&self, event: T) -> Result<(), EventBusError>;
    /// Publish an event to the state plane.
    async fn publish_state(&self, event: T) -> Result<(), EventBusError>;
    /// Subscribe to the data plane (single consumer).
    async fn subscribe_data(&self) -> Result<mpsc::Receiver<T>, EventBusError>;
    /// Subscribe to the control plane.
    fn subscribe_control(&self) -> broadcast::Receiver<T>;
    /// Subscribe to the state plane.
    fn subscribe_state(&self) -> watch::Receiver<Option<T>>;
    /// Access captured data plane events.
    fn captured_data(&self) -> Arc<Mutex<Vec<EventRecord<T>>>>;
    /// Access captured control plane events.
    fn captured_control(&self) -> Arc<Mutex<Vec<EventRecord<T>>>>;
    /// Access captured state plane events.
    fn captured_state(&self) -> Arc<Mutex<Vec<EventRecord<T>>>>;
}

/// Mock implementation of a hybrid event bus for testing.
#[derive(Debug)]
pub struct MockEventBus<T>
where
    T: Clone + Send + Sync + 'static,
{
    data_sender: mpsc::Sender<T>,
    data_receiver: Mutex<Option<mpsc::Receiver<T>>>,
    control_sender: broadcast::Sender<T>,
    state_sender: watch::Sender<Option<T>>,
    data_events: Arc<Mutex<Vec<EventRecord<T>>>>,
    control_events: Arc<Mutex<Vec<EventRecord<T>>>>,
    state_events: Arc<Mutex<Vec<EventRecord<T>>>>,
    data_sequence: AtomicU64,
    control_sequence: AtomicU64,
    state_sequence: AtomicU64,
}

impl<T> MockEventBus<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new mock event bus with capacity settings.
    #[must_use]
    pub fn new(data_capacity: usize, control_capacity: usize) -> Self {
        let (data_sender, data_receiver) = mpsc::channel(data_capacity);
        let (control_sender, _control_receiver) =
            broadcast::channel(control_capacity);
        let (state_sender, _state_receiver) = watch::channel(None);

        Self {
            data_sender,
            data_receiver: Mutex::new(Some(data_receiver)),
            control_sender,
            state_sender,
            data_events: Arc::new(Mutex::new(Vec::new())),
            control_events: Arc::new(Mutex::new(Vec::new())),
            state_events: Arc::new(Mutex::new(Vec::new())),
            data_sequence: AtomicU64::new(1),
            control_sequence: AtomicU64::new(1),
            state_sequence: AtomicU64::new(1),
        }
    }

    async fn record_event(
        events: &Arc<Mutex<Vec<EventRecord<T>>>>,
        sequence: &AtomicU64,
        event: T,
    ) -> EventRecord<T> {
        let next_sequence = sequence.fetch_add(1, Ordering::SeqCst);
        let record = EventRecord::new(next_sequence, event);
        let mut guard = events.lock().await;
        guard.push(record.clone());
        record
    }
}

#[async_trait]
impl<T> EventBusPort<T> for MockEventBus<T>
where
    T: Clone + Send + Sync + 'static,
{
    async fn publish_data(&self, event: T) -> Result<(), EventBusError> {
        let _ = Self::record_event(
            &self.data_events,
            &self.data_sequence,
            event.clone(),
        )
        .await;
        self.data_sender
            .send(event)
            .await
            .map_err(|_| EventBusError::data_plane_closed())?;
        let records = self.data_events.lock().await;
        let _ = SequenceAssertion::verify_increasing(&records);
        Ok(())
    }

    async fn publish_control(&self, event: T) -> Result<(), EventBusError> {
        let _ = Self::record_event(
            &self.control_events,
            &self.control_sequence,
            event.clone(),
        )
        .await;
        self.control_sender
            .send(event)
            .map(|_| ())
            .map_err(|_| EventBusError::control_plane_closed())
    }

    async fn publish_state(&self, event: T) -> Result<(), EventBusError> {
        let _ = Self::record_event(
            &self.state_events,
            &self.state_sequence,
            event.clone(),
        )
        .await;
        self.state_sender
            .send(Some(event))
            .map_err(|_| EventBusError::state_plane_closed())
    }

    async fn subscribe_data(&self) -> Result<mpsc::Receiver<T>, EventBusError> {
        let mut guard = self.data_receiver.lock().await;
        guard.take().ok_or_else(EventBusError::data_subscription_taken)
    }

    fn subscribe_control(&self) -> broadcast::Receiver<T> {
        self.control_sender.subscribe()
    }

    fn subscribe_state(&self) -> watch::Receiver<Option<T>> {
        self.state_sender.subscribe()
    }

    fn captured_data(&self) -> Arc<Mutex<Vec<EventRecord<T>>>> {
        Arc::clone(&self.data_events)
    }

    fn captured_control(&self) -> Arc<Mutex<Vec<EventRecord<T>>>> {
        Arc::clone(&self.control_events)
    }

    fn captured_state(&self) -> Arc<Mutex<Vec<EventRecord<T>>>> {
        Arc::clone(&self.state_events)
    }
}

#[cfg(test)]
// # LINT_DISABLE_REASON: Assertion macros in tests trigger disallowed-method linting.
// # LINT_DISABLE_REASON: Options tried: manual Result propagation.
// # LINT_DISABLE_REASON: Justification: keep tests concise.
#[allow(clippy::disallowed_methods)]
mod tests {
    use serde::Serialize;
    use tokio::sync::mpsc::Receiver;

    use super::*;
    use crate::{async_test, events::PayloadAssertion};

    #[derive(Debug, Clone, PartialEq, Serialize)]
    struct TestEvent {
        id: u64,
    }

    async fn data_plane_fixture(
        capacity: usize,
    ) -> Result<(MockEventBus<TestEvent>, Receiver<TestEvent>, TestEvent), String>
    {
        let bus = MockEventBus::new(capacity, capacity);
        let receiver =
            bus.subscribe_data().await.map_err(|error| error.to_string())?;
        let event = TestEvent {
            id: 42,
        };

        Ok((bus, receiver, event))
    }

    async fn publish_and_receive(
        bus: &MockEventBus<TestEvent>,
        receiver: &mut Receiver<TestEvent>,
        event: TestEvent,
    ) -> Result<TestEvent, String> {
        bus.publish_data(event).await.map_err(|error| error.to_string())?;

        receiver
            .recv()
            .await
            .ok_or_else(|| "missing data plane event".to_string())
    }

    async_test!(
        async fn data_plane_delivers_event_to_subscriber() {
            let fixture = data_plane_fixture(4).await;
            assert!(fixture.is_ok(), "fixture error: {fixture:?}");
            let Ok((bus, mut receiver, event)) = fixture else {
                return;
            };
            let expected = event.clone();

            let delivery =
                publish_and_receive(&bus, &mut receiver, event).await;

            assert!(
                matches!(delivery.as_ref(), Ok(payload) if payload == &expected),
                "delivery mismatch: {delivery:?}"
            );
        }
    );

    async_test!(
        async fn data_plane_captures_published_record() {
            let fixture = data_plane_fixture(4).await;
            assert!(fixture.is_ok(), "fixture error: {fixture:?}");
            let Ok((bus, mut receiver, event)) = fixture else {
                return;
            };
            let expected = event.clone();

            let record_result: Result<EventRecord<TestEvent>, String> =
                match publish_and_receive(&bus, &mut receiver, event).await {
                    Ok(_) => {
                        let records = bus.captured_data();
                        let guard = records.lock().await;
                        guard
                            .first()
                            .cloned()
                            .ok_or_else(|| "missing record".to_string())
                    }
                    Err(error) => Err(error),
                };

            assert!(
                matches!(record_result.as_ref(), Ok(record) if record.payload == expected),
                "record mismatch: {record_result:?}"
            );
        }
    );

    async_test!(
        async fn data_plane_verifies_payload_contract() {
            let fixture = data_plane_fixture(4).await;
            assert!(fixture.is_ok(), "fixture error: {fixture:?}");
            let Ok((bus, mut receiver, event)) = fixture else {
                return;
            };

            let payload_result =
                match publish_and_receive(&bus, &mut receiver, event.clone())
                    .await
                {
                    Ok(_) => {
                        let records = bus.captured_data();
                        let guard = records.lock().await;
                        let record_result = guard
                            .first()
                            .ok_or_else(|| "missing record".to_string());
                        match record_result {
                            Ok(record) => PayloadAssertion::verify(
                                &event,
                                &record.payload,
                            )
                            .map_err(|error| error.to_string()),
                            Err(error) => Err(error),
                        }
                    }
                    Err(error) => Err(error),
                };

            assert!(
                payload_result.is_ok(),
                "payload verification failed: {payload_result:?}"
            );
        }
    );

    async_test!(
        async fn data_plane_maintains_sequence_ordering() {
            let fixture = data_plane_fixture(4).await;
            assert!(fixture.is_ok(), "fixture error: {fixture:?}");
            let Ok((bus, mut receiver, event)) = fixture else {
                return;
            };

            let sequence_result =
                match publish_and_receive(&bus, &mut receiver, event).await {
                    Ok(_) => {
                        let records = bus.captured_data();
                        let guard = records.lock().await;
                        SequenceAssertion::verify_increasing(&guard)
                            .map_err(|error| error.to_string())
                    }
                    Err(error) => Err(error),
                };

            assert!(
                sequence_result.is_ok(),
                "sequence validation failed: {sequence_result:?}"
            );
        }
    );

    async_test!(
        async fn data_plane_reports_closed_channel_without_receiver() {
            let bus = MockEventBus::<TestEvent>::new(1, 1);
            let receiver_result = bus.subscribe_data().await;
            assert!(
                receiver_result.is_ok(),
                "subscribe error: {receiver_result:?}"
            );
            drop(receiver_result);

            let publish_result = bus
                .publish_data(TestEvent {
                    id: 1,
                })
                .await
                .map_err(|error| error.to_string());

            assert!(
                matches!(
                    publish_result.as_ref(),
                    Err(message) if message.as_str() == "Data plane channel closed"
                ),
                "unexpected publish result: {publish_result:?}"
            );
        }
    );

    async_test!(
        async fn data_plane_rejects_second_subscription() {
            let bus = MockEventBus::<TestEvent>::new(1, 1);
            let first_result = bus.subscribe_data().await;
            assert!(first_result.is_ok(), "subscribe error: {first_result:?}");
            drop(first_result);

            let second_result =
                bus.subscribe_data().await.map_err(|error| error.to_string());

            assert!(
                matches!(
                    second_result.as_ref(),
                    Err(message) if message.as_str() == "Data plane receiver already taken"
                ),
                "unexpected subscription result: {second_result:?}"
            );
        }
    );

    async_test!(
        async fn control_plane_broadcasts_events_to_subscribers() {
            let bus = MockEventBus::new(2, 4);
            let mut control_receiver = bus.subscribe_control();
            let event = TestEvent {
                id: 7,
            };
            let publish_result = bus
                .publish_control(event.clone())
                .await
                .map_err(|error| error.to_string());
            let delivery = match publish_result {
                Ok(()) => {
                    control_receiver.recv().await.map_err(|err| err.to_string())
                }
                Err(error) => Err(error),
            };

            assert!(
                matches!(delivery.as_ref(), Ok(payload) if payload == &event),
                "control plane delivery mismatch: {delivery:?}"
            );
        }
    );

    async_test!(
        async fn control_plane_reports_closed_channel_without_receiver() {
            let bus = MockEventBus::<TestEvent>::new(1, 1);
            let publish_result = bus
                .publish_control(TestEvent {
                    id: 2,
                })
                .await
                .map_err(|error| error.to_string());

            assert!(
                matches!(
                    publish_result.as_ref(),
                    Err(message) if message.as_str() == "Control plane channel closed"
                ),
                "unexpected publish result: {publish_result:?}"
            );
        }
    );

    async_test!(
        async fn control_plane_captures_recorded_events() {
            let bus = MockEventBus::<TestEvent>::new(1, 1);
            let _receiver = bus.subscribe_control();
            let event = TestEvent {
                id: 11,
            };

            let publish_result = bus
                .publish_control(event.clone())
                .await
                .map_err(|error| error.to_string());
            let record_result: Result<EventRecord<TestEvent>, String> =
                match publish_result {
                    Ok(()) => {
                        let records = bus.captured_control();
                        let guard = records.lock().await;
                        guard
                            .first()
                            .cloned()
                            .ok_or_else(|| "missing record".to_string())
                    }
                    Err(error) => Err(error),
                };

            assert!(
                matches!(record_result.as_ref(), Ok(record) if record.payload == event),
                "control plane record mismatch: {record_result:?}"
            );
        }
    );

    async_test!(
        async fn state_plane_updates_receivers_with_latest_event() {
            let bus = MockEventBus::new(2, 2);
            let state_receiver = bus.subscribe_state();
            let event = TestEvent {
                id: 9,
            };
            let publish_result = bus.publish_state(event.clone()).await;
            let received = state_receiver.borrow().clone();

            let outcome = publish_result
                .map_err(|error| error.to_string())
                .map(|_| received);

            assert!(
                matches!(outcome.as_ref(), Ok(Some(payload)) if payload == &event),
                "state plane update mismatch: {outcome:?}"
            );
        }
    );

    async_test!(
        async fn state_plane_reports_closed_channel_without_receiver() {
            let bus = MockEventBus::<TestEvent>::new(1, 1);
            let publish_result = bus
                .publish_state(TestEvent {
                    id: 3,
                })
                .await
                .map_err(|error| error.to_string());

            assert!(
                matches!(
                    publish_result.as_ref(),
                    Err(message) if message.as_str() == "State plane channel closed"
                ),
                "unexpected publish result: {publish_result:?}"
            );
        }
    );

    async_test!(
        async fn state_plane_captures_recorded_events() {
            let bus = MockEventBus::<TestEvent>::new(1, 1);
            let _receiver = bus.subscribe_state();
            let event = TestEvent {
                id: 12,
            };

            let publish_result = bus
                .publish_state(event.clone())
                .await
                .map_err(|error| error.to_string());
            let record_result: Result<EventRecord<TestEvent>, String> =
                match publish_result {
                    Ok(()) => {
                        let records = bus.captured_state();
                        let guard = records.lock().await;
                        guard
                            .first()
                            .cloned()
                            .ok_or_else(|| "missing record".to_string())
                    }
                    Err(error) => Err(error),
                };

            assert!(
                matches!(record_result.as_ref(), Ok(record) if record.payload == event),
                "state plane record mismatch: {record_result:?}"
            );
        }
    );

    #[test]
    fn event_plane_variants_match_expected_values() {
        let data = EventPlane::Data;
        let control = EventPlane::Control;
        let state = EventPlane::State;

        assert!(matches!(data, EventPlane::Data));
        assert!(matches!(control, EventPlane::Control));
        assert!(matches!(state, EventPlane::State));
    }
}
