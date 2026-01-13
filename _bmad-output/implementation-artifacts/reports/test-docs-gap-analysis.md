# Test Documentation Gap Analysis

**Date:** 2026-01-13
**Author:** Dev Agent

## Overview
This report identifies gaps and misalignments between existing testing documentation, project rules, and Rust ecosystem best practices.

## 1. Async Testing (`docs/testing/async.md`)
| Gap | Reference | Required Update |
|---|---|---|
| **Blocking Limit** | Project Context Rule 103 | Explicitly state the 10ms hard limit for blocking the executor. |
| **Concurrency Throttling** | Project Context Rule 85, 107 | Add guidance on using `tokio::sync::Semaphore` to limit concurrent I/O during tests. |
| **Shutdown Patterns** | Project Context Rule 106 | Add `tokio::select!` patterns for testing graceful shutdown. |
| **Semaphore Usage** | Project Context Rule 85 | Document usage of Semaphores for resource throttling in async tests. |

## 2. Event & CQRS Testing (`docs/testing/event.md`)
| Gap | Reference | Required Update |
|---|---|---|
| **CQRS Integration** | ADR 0009 | Add explicit section on Query Handler testing using stubs. |
| **Eventual Consistency** | ADR 0009 | Document patterns for testing consistency windows and timing control. |
| **Stub Usage** | ADR 0009 | Provide examples for `StubQueryStore` or similar read-model stubs. |

## 3. Infrastructure & Utilities (ADR 0010, 0011)
| Gap | Reference | Required Update |
|---|---|---|
| **Testcontainers Status** | ADR 0011 | Clarify that `testcontainers` is currently deferred due to RUSTSEC-2025-0134 and suggest `mockall` as the primary alternative for now. |
| **Fixture Patterns** | ADR 0010 | Add specific examples of `rstest` usage for domain fixtures. |
| **Snapshot Testing** | Story 2.9 AC 3 | Documentation on `insta` usage and redaction guidance is currently missing from ADRs. |

## 4. General Alignment
- **Naming Conventions:** Standardize on verb-first naming (e.g., `maintains_...`, `validates_...`) across all docs.
- **Project Rules:** Ensure all docs reference `mise run` as the authoritative entry point.
