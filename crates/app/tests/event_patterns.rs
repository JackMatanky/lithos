//! Event-driven testing pattern examples for the Lithos App crate.

// # LINT_DISABLE_REASON: Integration tests do not require public documentation.
// | Options tried: Adding docs to every test function.
// | Justification: Tests are self-documenting by their names and logic.
#![allow(
    missing_docs,
    reason = "Integration tests do not require public documentation"
)]

#[cfg(test)]
// # LINT_DISABLE_REASON: Assertion macros in tests trigger disallowed-method linting.
// # LINT_DISABLE_REASON: Options tried: explicit matches/guarded Result handling.
// # LINT_DISABLE_REASON: Justification: keep tests readable without unwrap/expect.
#[expect(
    clippy::disallowed_methods,
    clippy::arbitrary_source_item_ordering,
    reason = "Test assertions use Result helpers without unwrap/expect"
)]
mod tests {
    use std::{
        sync::{
            Arc,
            atomic::{AtomicU64, Ordering},
        },
        time::Duration,
    };

    use chrono::{DateTime, Utc};
    use lithos_test_utils::{
        EventBusPort as _, EventRecord, EventTestFramework, MockEventBus,
        PayloadAssertion, SequenceAssertion, TimingAssertion, with_timeout,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};
    use tokio::sync::mpsc::Receiver;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct AccountEvent {
        account_id: String,
        version: u64,
    }

    mod event_test_framework {
        use super::*;

        /// Confirms Given/When/Then scenarios return expected events.
        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn returns_expected_events_for_given_history() {
            let expected = vec![account_event("acct-1", 2)];
            let result =
                EventTestFramework::given(vec![account_event("acct-1", 1)])
                    .when(|_history| vec![account_event("acct-1", 2)])
                    .then_expect_events(&expected);

            assert!(result.is_ok(), "unexpected result: {result:?}");
        }
    }

    mod data_plane {
        use super::*;

        /// Ensures published events reach a data-plane subscriber.
        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn delivers_published_event_to_subscriber() {
            let fixture = data_plane_fixture(8).await;
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

        /// Captures published records for assertions against stored payloads.
        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn captures_published_events_for_assertion() {
            let fixture = data_plane_fixture(8).await;
            assert!(fixture.is_ok(), "fixture error: {fixture:?}");
            let Ok((bus, mut receiver, event)) = fixture else {
                return;
            };
            let expected = event.clone();

            let record_result: Result<EventRecord<AccountEvent>, String> =
                match publish_and_receive(&bus, &mut receiver, event).await {
                    Ok(_) => {
                        let records = bus.captured_data();
                        let guard = records.lock().await;
                        guard
                            .first()
                            .cloned()
                            .ok_or_else(|| "missing record".to_owned())
                    }
                    Err(error) => Err(error),
                };

            assert!(
                matches!(record_result.as_ref(), Ok(record) if record.payload == expected),
                "record mismatch: {record_result:?}"
            );
        }

        /// Verifies recorded data plane events keep increasing sequences.
        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn maintains_event_sequence_ordering() {
            let fixture = data_plane_fixture(8).await;
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

        /// Validates payload equality with the domain event contract helper.
        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn verifies_payload_integrity_with_contract_helper() {
            let fixture = data_plane_fixture(8).await;
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
                            .ok_or_else(|| "missing record".to_owned());
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
    }

    mod control_plane {
        use super::*;

        /// Confirms control-plane broadcasts reach subscribed listeners.
        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn broadcasts_control_events_to_subscribers() {
            let bus = MockEventBus::new_with_clock(4, 4, fixed_clock());
            let mut receiver = bus.subscribe_control();
            let event = account_event("acct-3", 1);
            let publish_result = bus
                .publish_control(event.clone())
                .await
                .map_err(|error| error.to_string());
            let delivery = match publish_result {
                Ok(()) => receiver.recv().await.map_err(|err| err.to_string()),
                Err(error) => Err(error),
            };

            assert!(
                matches!(delivery.as_ref(), Ok(payload) if payload == &event),
                "control plane delivery mismatch: {delivery:?}"
            );
        }
    }

    mod subscriber_handling {
        use super::*;

        fn parse_event(value: Value) -> Result<AccountEvent, String> {
            serde_json::from_value(value)
                .map_err(|error| format!("malformed event payload: {error}"))
        }

        /// Ensures malformed events are handled without panics.
        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn reports_malformed_event_payloads() {
            let malformed = json!({"account_id": 10i64, "version": "oops"});
            let result = parse_event(malformed);

            assert!(result.is_err(), "expected malformed event error");
        }

        /// Ensures subscriber lifecycle allows only one data-plane receiver.
        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn enforces_single_data_plane_subscription() {
            let bus: MockEventBus<AccountEvent> =
                MockEventBus::new_with_clock(2, 2, fixed_clock());
            let first = bus.subscribe_data().await;
            assert!(first.is_ok(), "first subscription failed: {first:?}");

            let second = bus.subscribe_data().await;
            assert!(
                second.is_err(),
                "expected subscription rejection, got: {second:?}"
            );
        }
    }

    mod event_flow {
        use super::*;

        /// Validates end-to-end event flow across planes.
        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn relays_events_from_data_to_control_plane() -> Result<(), String>
        {
            let bus =
                Arc::new(MockEventBus::new_with_clock(4, 4, fixed_clock()));
            let mut data_receiver = bus
                .subscribe_data()
                .await
                .map_err(|error| error.to_string())?;
            let mut control_receiver = bus.subscribe_control();
            let bus_for_task = Arc::clone(&bus);
            let relay_task = tokio::spawn(async move {
                match data_receiver.recv().await {
                    Some(event) => bus_for_task
                        .publish_control(event)
                        .await
                        .map_err(|error| error.to_string()),
                    None => Err("data plane closed".to_owned()),
                }
            });

            let event = account_event("acct-flow", 1);
            bus.publish_data(event.clone())
                .await
                .map_err(|error| error.to_string())?;

            let delivered = with_timeout(Duration::from_secs(1), async {
                control_receiver.recv().await.map_err(|err| err.to_string())
            })
            .await
            .map_err(|error| error.to_string())??;

            let relay_result =
                relay_task.await.map_err(|error| error.to_string())?;
            if let Err(error) = relay_result {
                return Err(format!("relay failed: {error}"));
            }
            if delivered != event {
                return Err(format!(
                    "control plane payload mismatch: {delivered:?}"
                ));
            }

            let records = bus.captured_data();
            let guard = records.lock().await;
            TimingAssertion::verify_non_decreasing(&guard).map_err(
                |error| format!("timing verification failed: {error}"),
            )?;

            Ok(())
        }
    }

    fn fixed_clock() -> Arc<dyn Fn() -> DateTime<Utc> + Send + Sync> {
        let counter = Arc::new(AtomicU64::new(0));
        let counter_ref = Arc::clone(&counter);
        Arc::new(move || {
            let offset =
                i64::try_from(counter_ref.fetch_add(1, Ordering::SeqCst))
                    .unwrap_or_default();
            let base_timestamp = 1_700_000_000i64;
            let timestamp =
                base_timestamp.checked_add(offset).unwrap_or(base_timestamp);
            DateTime::<Utc>::from_timestamp(timestamp, 0)
                .unwrap_or_else(Utc::now)
        })
    }

    fn account_event(account_id: &str, version: u64) -> AccountEvent {
        AccountEvent {
            account_id: account_id.into(),
            version,
        }
    }

    async fn data_plane_fixture(
        capacity: usize,
    ) -> Result<
        (MockEventBus<AccountEvent>, Receiver<AccountEvent>, AccountEvent),
        String,
    > {
        let bus =
            MockEventBus::new_with_clock(capacity, capacity, fixed_clock());
        let receiver = bus
            .subscribe_data()
            .await
            .map_err(|error| format!("data plane subscribe failed: {error}"))?;
        let event = account_event("acct-2", 1);

        Ok((bus, receiver, event))
    }

    async fn publish_and_receive(
        bus: &MockEventBus<AccountEvent>,
        receiver: &mut Receiver<AccountEvent>,
        event: AccountEvent,
    ) -> Result<AccountEvent, String> {
        bus.publish_data(event).await.map_err(|error| error.to_string())?;

        receiver
            .recv()
            .await
            .ok_or_else(|| "missing data plane event".to_owned())
    }
}
