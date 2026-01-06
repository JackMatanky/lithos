# Events Package

This package contains application-level and telemetry events used for coordination and observability within Lithos.

## Event Categories

Events in Lithos are categorized into three distinct layers to maintain a clean separation of concerns:

### 1. Domain Events (`internal/domain/events.go`)
**Purpose:** Represent significant business-significant state changes in the core domain models.
**When to use:** Use for events that represent a change in the core system state that other domain services or external layers might need to react to (e.g., a note being indexed, a schema being updated).

**Events:**
- `NoteCreatedEvent`
- `NoteIndexedEvent`
- `SchemaLoadedEvent`
- `SchemasReloadedEvent`
- `SchemaUpdatedEvent`
- `FrontmatterValidatedEvent`

### 2. Workflow Events (`internal/app/events/workflow.go`)
**Purpose:** Coordinate infrastructure-level activities and application workflows.
**When to use:** Use for events that trigger or signal the completion of specific application-level tasks, such as file discovery, parsing, or cache requests. These are "moving parts" events.

**Events:**
- `CommandIssuedEvent`
- `FileDiscoveredEvent`
- `FileParseRequestedEvent`
- `NoteParsedEvent`
- `FrontmatterValidationRequestedEvent`
- `NoteCacheRequestedEvent`

### 3. Telemetry Events (`internal/app/events/telemetry.go`)
**Purpose:** Provide observability into system performance and operational health.
**When to use:** Use for events that track how the system is performing (durations, counts, result metadata) or to record operational failures that don't necessarily change domain state but are critical for monitoring.

**Events:**
- `LookupPerformedEvent`
- `QueryPerformedEvent`
- `SchemaLookupEvent`
- `ValidationPerformedEvent`
- `ValidationFailedEvent`
- `VaultIndexingCompleteEvent`

## Architecture Guidelines

- **Package Independence:** `internal/domain` events should never depend on `internal/app`.
- **Loose Coupling:** Services should publish events to the `EventBus` rather than calling other services directly whenever possible.
- **Async by Default:** Observability and workflow coordination events are typically published asynchronously via `PublishAsync` to avoid impacting core operation latency.
