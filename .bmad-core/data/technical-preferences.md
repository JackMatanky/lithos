<!-- Powered by BMAD™ Core -->

# Technical Preferences: User-Defined Preferred Patterns and Preferences

## Architectural Patterns

### Hexagonal Architecture (Ports & Adapters)

**Pattern:** Core logic is isolated from infrastructure. The core communicates via **ports** (interfaces). Implementations are **adapters**.

**Rationale:** Testability, technology independence, maintainability.

---

#### Structure

- **Domain**
  - _Models (Entities/Value Objects)_
    - Pure business data + invariants.
    - **MUST NOT** depend on frameworks, transport, storage, or application services.
  - _Services (Use Cases/Application Services)_
    - Orchestrate domain behavior; enforce policies.
    - **MAY** depend on domain models and ports.
    - **MUST NOT** depend on adapters or external libraries.
- **Ports**
  - Contracts for inbound (**API**) and outbound (**SPI**) boundaries.
  - **MUST** contain only interfaces and domain types as needed.
  - **MUST NOT** reference frameworks or concrete adapters.
- **Adapters**
  - Technology-specific implementations (HTTP/CLI/UI, DB/Queue/Search, etc.).
  - **MUST NOT** contain domain logic.
  - **MUST** isolate framework and I/O concerns.

---

#### The Dependency Rule

Dependencies **MUST** only point inwards toward the core domain: **adapter → port → domain**.

##### Domain Layer

- **Models**
  - **MUST NOT** depend on any other layer.
  - **MUST** express business concepts and invariants.
  - **MUST NOT** import technology-specific code.
- **Services**
  - **MAY** depend on domain models and ports.
  - **MUST NOT** depend on adapters or third-party libraries directly.

##### Ports Layer

- **MAY** depend on domain models for I/O contracts.
- **MUST NOT** depend on adapters.
- **MUST NOT** expose implementation details.
- **MUST** remain framework-agnostic.

##### Adapters Layer

- **MUST** implement and depend **only** on ports.
- **MUST** encapsulate framework, I/O, and external system concerns.
- **MUST NOT** contain domain or business logic.
- **MUST NOT** introduce cyclic or cross-layer imports.
- **MUST** communicate with the application core **only** through ports.
- Adapter wiring **MUST** occur at the composition root (bootstrap/main).
- **MUST NOT** depend directly on other adapters.
- **MAY** interact with adapters only through ports.
- **Adapter-Adapter Interaction**:
  - API adapters **MAY** invoke or drive SPI adapters.
  - SPI adapters **MAY** coordinate other SPI adapters.
  - API-API interactions **SHOULD** be avoided, but **MAY** be used for routing/forwarding.
  - SPI-API interactions **MUST NOT** occur.
- **Adapter-Domain Model Interaction**:
  - **MAY** reference domain **types** only as passive data in port contracts; **MUST NOT** invoke domain methods directly.
  - **MUST NOT** modify or persist domain entities; all state changes **MUST** occur through application services or use cases defined by ports.
  - **MUST NOT** expose domain models externally; boundary DTOs or schemas **MUST** be used instead.

##### Shared Utilities

- **MAY** depend only on the language standard library.
- **MUST** avoid cross-layer or circular dependencies.
- **SHOULD** remain side-effect-free and technology-neutral.

---

### CQRS (Command Query Responsibility Segregation)

**Pattern:** Separate **commands** (write) and **queries** (read) within the core.

- **Commands**
  - **MUST** have single-responsibility handlers enforcing domain invariants.
  - **MUST NOT** read from query projections directly.
  - **MAY** emit domain events and orchestrate persistence via ports.
- **Queries**
  - **MUST** be read-only.
  - **MAY** use projections/simplified read models.
  - **MUST NOT** mutate state.
- **Models**
  - Write/read models **MAY** differ.
  - Synchronization **MAY** be synchronous or event-driven.

**Rationale:** Scalability under asymmetric load, clear side-effects, event-sourcing/audit readiness.
