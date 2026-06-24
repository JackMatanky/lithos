# Design: Decoupling Discovery and Config Modules

## Overview

This design decouples the `config` module from the `discovery` module by removing the direct dependency of `config::Builder` on `discovery::DiscoveryResult`. Instead, `Builder::new()` will take clearly defined parameters (`vault`, `global`, `repository`), and the responsibility for orchestrating this relationship will move to the `app` (or equivalent orchestrator) layer.

## Proposed Changes

### 1. `Discovery` Service Layer

-   **`DiscoveryResult`**: Add a `report` field to `traces_core::discovery::service::DiscoveryResult`.
-   **`DiscoveryService::discover`**: Update the return type to `Result<DiscoveryResult, DiscoveryError>` (consolidating `Result` and `Report`).
-   **`DiscoveryProcessor::finalize`**: Update to combine the result and report into the updated `DiscoveryResult` before returning.

### 2. `Config` Builder Layer

-   **`Builder::new()`**: Update the constructor to accept explicit parameters:
    ```rust
    pub fn new(
        vault: Box<[CandidatePath]>,
        global: Box<[CandidatePath]>,
        repository: R,
    ) -> Self { ... }
    ```
-   The `config` module will no longer import `DiscoveryResult` or `DiscoveryReport`.

### 3. Orchestration Layer (`app/bootstrapper.rs`)

-   **`Bootstrapper`**: Responsible for calling `DiscoveryService::discover` and unpacking the returned `DiscoveryResult` to supply the parameters required by `Builder::new()`.

## Impact

-   **Decoupling**: `config/` module no longer knows about the internal result/report structures of the `discovery/` module.
-   **API Clarity**: `Builder::new()` is explicit about required inputs.
-   **Orchestration**: The `Bootstrapper` layer properly owns the relationship between discovery outcomes and config construction.

## Risks and Mitigation

-   **Orchestration Burden**: The orchestrator now has to unpack `DiscoveryResult`. This is a minor cost for the significant benefit of decoupling.
-   **API Breakage**: This change requires updating all call sites of `DiscoveryService::discover` and `Builder::new()` (or `from_discovery`). This is expected and necessary.

## Testing

-   Update `discovery` service tests to expect the consolidated `DiscoveryResult`.
-   Update `config` builder tests to use `Builder::new()`.
-   Update `bootstrap` tests to account for the new orchestration flow.
