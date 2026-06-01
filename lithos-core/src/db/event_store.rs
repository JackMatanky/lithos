#![allow(
    dead_code,
    reason = "Contract is consumed by upcoming repository slices"
)]

//! Shared `EventStore` contract and redb-backed implementation.

use std::{marker::PhantomData, sync::Arc};

use redb::ReadableTable as _;

use crate::db::{
    ArchivedEntity, DbError, EventId, EventIdError, EventTable, Store, Table,
};

/// Infrastructure contract for append/load/compact event-log behavior.
pub(crate) trait EventStore<E>
where
    E: ArchivedEntity,
{
    /// Append an event atomically and return the allocated event id.
    fn append(&self, event: &E) -> Result<EventId, DbError>;

    /// Load all events in deterministic ascending [`EventId`] order.
    fn load_all_events(&self) -> Result<Vec<(EventId, E)>, DbError>;

    /// Compact all events with id strictly less than `cutoff`.
    fn compact_before(&self, cutoff: EventId) -> Result<u64, DbError>;
}

/// redb-backed `EventStore` adapter scoped to repository-owned table names.
pub(crate) struct RedbEventStore<E>
where
    E: ArchivedEntity,
{
    store: Arc<Store>,
    sequence_table: Table<&'static str, u64>,
    sequence_key: &'static str,
    events_table: EventTable<&'static [u8]>,
    _event: PhantomData<E>,
}

impl<E> RedbEventStore<E>
where
    E: ArchivedEntity,
{
    /// Create a new repository-owned event store definition.
    pub(crate) const fn new(
        store: Arc<Store>,
        sequence_table: Table<&'static str, u64>,
        sequence_key: &'static str,
        events_table: EventTable<&'static [u8]>,
    ) -> Self {
        Self {
            store,
            sequence_table,
            sequence_key,
            events_table,
            _event: PhantomData,
        }
    }

    fn allocate_next_event_id(
        &self,
        tx: &mut crate::db::WriteTx,
    ) -> Result<EventId, DbError> {
        let mut sequence =
            tx.try_open_table(self.sequence_table.definition())?;
        let current = sequence.get(self.sequence_key)?.map(|v| v.value());
        let next = EventId::next_after(
            current
                .map(EventId::try_from_raw)
                .transpose()
                .map_err(|e| DbError::Deserialization(e.to_string()))?,
        )
        .map_err(|error| map_event_id_error(&error))?;
        sequence.insert(self.sequence_key, next.get())?;
        Ok(next)
    }

    fn append_with(
        &self,
        event: &E,
        before_insert: impl FnOnce(EventId) -> Result<(), DbError>,
    ) -> Result<EventId, DbError> {
        let bytes = event.to_bytes()?;
        self.store.write(|tx| {
            let next = self.allocate_next_event_id(tx)?;
            before_insert(next)?;
            let mut events =
                tx.try_open_table(self.events_table.definition())?;
            events.insert(&next, bytes.as_slice())?;
            Ok(next)
        })
    }
}

impl<E> EventStore<E> for RedbEventStore<E>
where
    E: ArchivedEntity,
{
    fn append(&self, event: &E) -> Result<EventId, DbError> {
        self.append_with(event, |_| Ok(()))
    }

    fn load_all_events(&self) -> Result<Vec<(EventId, E)>, DbError> {
        self.store.read(|tx| {
            let Some(table) =
                tx.try_open_table(self.events_table.definition())?
            else {
                return Ok(Vec::new());
            };

            let mut events = Vec::new();
            for entry in table.range(EventId::MIN..)? {
                let (id, bytes) = entry?;
                let event = E::from_bytes(bytes.value())?;
                events.push((id.value(), event));
            }

            Ok(events)
        })
    }

    fn compact_before(&self, cutoff: EventId) -> Result<u64, DbError> {
        self.store.write(|tx| {
            let mut table =
                tx.try_open_table(self.events_table.definition())?;

            let ids_to_remove = table
                .range(EventId::MIN..cutoff)?
                .map(|entry| entry.map(|(id, _)| id.value()))
                .collect::<Result<Vec<_>, _>>()?;

            for id in &ids_to_remove {
                table.remove(id)?;
            }

            let removed_count = u64::try_from(ids_to_remove.len())
                .map_err(|error| DbError::Serialization(error.to_string()))?;
            Ok(removed_count)
        })
    }
}

fn map_event_id_error(error: &EventIdError) -> DbError {
    match error {
        EventIdError::Overflow => DbError::Serialization(error.to_string()),
        _ => DbError::Deserialization(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use rkyv::{Archive, Deserialize, Serialize};

    use super::*;

    const SEQ_A: Table<&str, u64> = Table::new("ctx_a_sequence");
    const SEQ_B: Table<&str, u64> = Table::new("ctx_b_sequence");
    const EVENTS_A: EventTable<&[u8]> = EventTable::new("ctx_a_events");
    const EVENTS_B: EventTable<&[u8]> = EventTable::new("ctx_b_events");

    #[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    #[rkyv(derive(Debug, PartialEq, Eq))]
    struct TestEvent {
        name: String,
    }

    fn event(name: &str) -> TestEvent {
        TestEvent {
            name: name.to_owned(),
        }
    }

    fn make_store() -> (tempfile::TempDir, Arc<Store>) {
        let (temp, store) = Store::open_temp().expect("temp store");
        (temp, Arc::new(store))
    }

    mod transactions {
        use super::*;

        #[test]
        fn returns_allocated_event_id_when_append_succeeds() {
            let (_temp, store) = make_store();
            let event_store =
                RedbEventStore::<TestEvent>::new(store, SEQ_A, "a", EVENTS_A);

            let id = event_store.append(&event("first")).expect("append");

            assert_eq!(id.get(), 1);
        }

        #[test]
        fn rejects_partial_append_when_failure_occurs_after_allocation() {
            let (_temp, store) = make_store();
            let event_store =
                RedbEventStore::<TestEvent>::new(store, SEQ_A, "a", EVENTS_A);

            let result = event_store.append_with(&event("first"), |_| {
                Err(DbError::Serialization("injected failure".to_owned()))
            });

            assert!(result.is_err());

            let loaded = event_store.load_all_events().expect("load");
            assert!(loaded.is_empty());

            let next = event_store.append(&event("second")).expect("append");
            assert_eq!(next.get(), 1);
        }
    }

    mod list {
        use super::*;

        #[test]
        fn returns_events_in_ascending_event_id_order() {
            let (_temp, store) = make_store();
            let event_store =
                RedbEventStore::<TestEvent>::new(store, SEQ_A, "a", EVENTS_A);

            let _ = event_store.append(&event("first")).expect("append 1");
            let _ = event_store.append(&event("second")).expect("append 2");
            let _ = event_store.append(&event("third")).expect("append 3");

            let loaded = event_store.load_all_events().expect("load");
            let ids = loaded.iter().map(|(id, _)| id.get()).collect::<Vec<_>>();
            let names =
                loaded.iter().map(|(_, e)| e.name.clone()).collect::<Vec<_>>();

            assert_eq!(ids, vec![1, 2, 3]);
            assert_eq!(names, vec!["first", "second", "third"]);
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn returns_independent_sequences_for_distinct_context_stores() {
            let (_temp, store) = make_store();
            let a = RedbEventStore::<TestEvent>::new(
                store.clone(),
                SEQ_A,
                "a",
                EVENTS_A,
            );
            let b =
                RedbEventStore::<TestEvent>::new(store, SEQ_B, "b", EVENTS_B);

            let a1 = a.append(&event("a1")).expect("a1");
            let b1 = b.append(&event("b1")).expect("b1");
            let a2 = a.append(&event("a2")).expect("a2");
            let b2 = b.append(&event("b2")).expect("b2");

            assert_eq!((a1.get(), a2.get()), (1, 2));
            assert_eq!((b1.get(), b2.get()), (1, 2));
        }
    }

    mod compaction {
        use super::*;

        #[test]
        fn removes_prefix_events_without_reusing_event_ids() {
            let (_temp, store) = make_store();
            let event_store =
                RedbEventStore::<TestEvent>::new(store, SEQ_A, "a", EVENTS_A);

            let _ = event_store.append(&event("first")).expect("append 1");
            let _ = event_store.append(&event("second")).expect("append 2");
            let third = event_store.append(&event("third")).expect("append 3");

            let removed = event_store.compact_before(third).expect("compact");
            assert_eq!(removed, 2);

            let loaded = event_store.load_all_events().expect("load");
            assert_eq!(loaded.len(), 1);
            let (id, loaded_event) = loaded.first().expect("one event remains");
            assert_eq!(id.get(), 3);
            assert_eq!(loaded_event.name, "third");

            let next = event_store.append(&event("fourth")).expect("append 4");
            assert_eq!(next.get(), 4);
        }
    }
}
