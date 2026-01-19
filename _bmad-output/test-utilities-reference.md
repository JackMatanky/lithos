# Lithos Test Utilities Reference (Epic 2)

This reference summarizes reusable test utilities provided by the Epic 2 test infrastructure in `tests/utils` (crate name `lithos_test_utils`). Examples are intentionally small and focus on usage patterns.

## Core async + timing helpers

### Async test macro

```rust
use lithos_test_utils::async_test;

async_test!(async fn validates_async_flow() {
    // GIVEN: async operation
    // WHEN: it completes
    // THEN: result matches expectation
    assert_eq!(1 + 1, 2);
});
```

### Virtual time for deterministic tests

```rust
use lithos_test_utils::time_test;
use tokio::time::Duration;

time_test!(async fn advances_virtual_time_for_timeouts() {
    tokio::time::advance(Duration::from_secs(1)).await;
});
```

### Timeouts and cancellation

```rust
use lithos_test_utils::{with_timeout, with_cancellation, default_test_timeout};
use tokio::time::Duration;

async fn sample() -> i32 { 42 }

async fn run() {
    let value = with_timeout(default_test_timeout(), sample()).await.unwrap();
    let _ = with_cancellation(Duration::from_secs(1), |_cancel| async move {
        Ok::<_, Box<dyn std::error::Error>>(value)
    }).await;
}
```

### Spawn blocking test helper

```rust
use lithos_test_utils::spawn_blocking_test;

async fn run() {
    let value = spawn_blocking_test(|| 7 * 6).await.unwrap();
    assert_eq!(value, 42);
}
```

### Isolated test context

```rust
use lithos_test_utils::IsolatedTestContext;

let ctx = IsolatedTestContext::new("schema_validation");
let db_path = ctx.db_path();
```

## Filesystem helpers

### Temp directories

```rust
use lithos_test_utils::TempDir;

let temp_dir = TempDir::new().unwrap();
let file_path = temp_dir.path().join("note.md");
```

### Centralized test outputs

```rust
use lithos_test_utils::TestOutput;

let output = TestOutput::new("coverage_audit").unwrap();
let report_path = output.file_path("report.json");
```

### Path utilities + unique names

```rust
use lithos_test_utils::{generate_unique_name, path_utils};

let name = generate_unique_name("fixture");
let path = path_utils::join(&["fixtures", &name]);
```

### Test vault helpers

```rust
use lithos_test_utils::TestVault;

let vault = TestVault::new()
    .with_note("Work/Project.md", "# Project")
    .with_config("lithos.toml", "[vault]\nstrict = true")
    .build();
```

## Data fixtures + factories

### Fake data scenarios

```rust
use lithos_test_utils::{FakeData, Scenario};

let realistic = FakeData::new(Scenario::Realistic).name();
let edge_case = FakeData::new(Scenario::EdgeCase).email();
```

### Fixture composition

```rust
use lithos_test_utils::{Fixture, combine};

let user = Fixture::new("user").with("name", "Ada");
let config = Fixture::new("config").with("debug", true);
let merged = combine(vec![user, config]);
```

### Serialization helpers

```rust
use lithos_test_utils::SerializationHelper;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
struct Payload { id: u32 }

let payload = Payload { id: 7 };
SerializationHelper::validate_round_trip(&payload).unwrap();
```

### Standard test fixtures

```rust
use lithos_test_utils::{test_user, test_config};

let user = test_user();
let config = test_config();
```

## Assertions and error helpers

### Detailed equality

```rust
use lithos_test_utils::assert_eq_detailed;

assert_eq_detailed!(vec![1, 2], vec![1, 2]);
```

### Async completion + eventual conditions

```rust
use lithos_test_utils::{assert_async_completed, assert_eventually};
use tokio::time::Duration;

async fn run() {
    let value = assert_async_completed!(async { 42 }, Duration::from_secs(1));
    assert_eq!(value, 42);

    assert_eventually!(|| true, Duration::from_secs(1));
}
```

### Standard error matching

```rust
use lithos_test_utils::assert_err_kind;

#[derive(Debug)]
enum MyError { Invalid }

let result: Result<(), MyError> = Err(MyError::Invalid);
assert_err_kind!(result, MyError::Invalid);
```

### Domain-specific assertions

```rust
use lithos_test_utils::domain::assert_contains_same_items;

assert_contains_same_items(&[1, 2], &[2, 1]).unwrap();
```

## CQRS + event testing

### CQRS helpers

```rust
use lithos_test_utils::{CqrsTestAdapter, EventVerifier};

let adapter = CqrsTestAdapter::default();
let verifier = EventVerifier::new();
```

### Event test framework

```rust
use lithos_test_utils::EventTestFramework;

let framework = EventTestFramework::new();
```

## Integration testing fixtures

```rust
use lithos_test_utils::{IntegrationConfig, IntegrationFixture};

let fixture = IntegrationFixture::new_with_config(IntegrationConfig::default()).await;
fixture.teardown().await;
```

## Benchmarks + performance gates

```rust
use lithos_test_utils::{create_benchmark_runtime, standard_criterion, performance_gates};

let mut criterion = standard_criterion();
let runtime = create_benchmark_runtime();
let warning = performance_gates::WARNING_THRESHOLD;
```

## Observability + tracing assertions

```rust
use lithos_test_utils::obs::tracing::init_tracing;
use tracing::{info, Level, span};

let handle = init_tracing();
info!("test");
let _span = span!(Level::INFO, "context").entered();
handle.assert_logged("test");
handle.assert_span_created("context");
```

## Mocks

### Event bus mock

```rust
use lithos_test_utils::mocks::event_bus::{EventBusPort, MockEventBus};

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
let bus = MockEventBus::<String>::new(8, 8);
let mut receiver = bus.subscribe_data().await?;
bus.publish_data("NoteCreated".to_string()).await?;
let event = receiver.recv().await.unwrap();
assert_eq!(event, "NoteCreated");
# Ok(())
# }
```
