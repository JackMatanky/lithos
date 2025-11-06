<!-- Powered by BMAD™ Core -->

# Technical Preferences: User-Defined Preferred Patterns and Preferences

## Architectural Patterns

### Hexagonal Architecture (Ports & Adapters)

**Pattern:**

- Core logic is isolated from infrastructure.
- The core communicates through **ports** (interfaces).
- Concrete **adapters** implement these ports for real technologies.
- Dependencies **MUST** always point inward toward the domain:
  - **adapter → port → application service → domain.**

**Rationale:** Enables testability, technology independence, composability, and maintainability by enforcing unidirectional dependencies toward the core.

#### Conceptual Overview

| Layer                    | Core Role                                         | Clean Architecture Equivalent |
| ------------------------ | ------------------------------------------------- | ----------------------------- |
| **Domain**               | Defines business rules and invariants             | Entities                      |
| **Application Services** | Coordinates domain behavior for use cases         | Use Cases                     |
| **Ports**                | Define boundaries between core and external world | Interfaces / Boundaries       |
| **Adapters**             | Implement boundary interfaces                     | Interface Adapters / Infra    |
| **API Ports / Adapters** | External systems call into the core               | Presentation Layer            |
| **SPI Ports / Adapters** | The core calls out to external systems            | Infrastructure Layer          |

#### Domain Layer (Entities / Value Objects)

**Purpose:** Represent and protect business rules and invariants.

**Dependency Rules**

- **MUST NOT** depend on any framework or I/O concern.
- **MUST** define all invariants and domain logic.
- **MAY** depend on pure helpers or value objects within domain.
- **MUST NOT** perform persistence or external calls.

**Prompting Questions**

- Does this express what the business allows, not how it’s done?
- Can this logic be defined using only the entity’s own data?
- Would it still make sense if frameworks or storage changed?

**Heuristics**

- Single-entity rule → Domain method.
- Multi-aggregate rule → Application service.
- I/O or persistence → Adapter.

#### Application Services Layer (Use Cases / Orchestration)

**Purpose:** Coordinate domain entities and ports to realize use cases.

**Rules**

- MAY depend on domain models and ports.
- MUST NOT depend on adapters or frameworks.
- MUST define transactional boundaries and orchestrate side effects via ports.
- MUST encapsulate workflows, not domain invariants.

**Prompting Questions**

- Does this coordinate multiple entities or repositories?
- Does it invoke ports or manage transactions?
- Would this read like a use case?

**Heuristics**

- Needs a port or transaction → Application service.
- Coordinates multiple aggregates → Application service.
- Contains pure business rule → Move to domain.

### Ports Layer (Boundaries)

**Purpose:** Define how the core interacts with external systems.

| API Ports                                     | SPI Ports                                                            |
| --------------------------------------------- | -------------------------------------------------------------------- |
| Inbound                                       | Outbound                                                             |
| Define entry points into the core (use cases) | Define dependencies the core requires (persistence, messaging, APIs) |
| Represent what the application offers         | Represent what the application needs                                 |

**Rules**

- MUST be interfaces only.
- MAY depend on domain types or DTOs.
- MUST NOT depend on adapters or frameworks.
- MUST express contracts in domain language.

**Prompting Questions**

- Is this an abstraction the core depends on? → SPI Port.
- Is this an interface invoked externally? → API Port.

**Heuristics**

- SPI → “We depend on this.”
- API → “They depend on us.”

### Adapters Layer (Implementations)

**Purpose:** Implement boundaries using real technologies.

| API Ports                                              | SPI Ports                                        |
| ------------------------------------------------------ | ------------------------------------------------ |
| Inbound                                                | Outbound                                         |
| Implement API ports for external entry (HTTP, CLI, MQ) | Implement SPI ports                              |
| Translate input → DTO → service call                   | Handle persistence, caching, or API integrations |
| Map results/errors to responses                        | Map domain ↔ infrastructure models              |

**Rules**

- MUST depend on ports, not domain logic.
- MUST isolate frameworks and I/O.
- MUST NOT contain domain logic or invoke domain methods directly.
- MUST NOT expose domain entities externally.
- MUST wire dependencies at the composition root.
- MAY interact with other adapters only through ports.

**Adapter Interaction Rules**

- API → SPI: allowed.
- SPI → SPI: allowed.
- API → API: avoid unless routing.
- SPI → API: forbidden.

**Prompting Questions**

- Does it use frameworks, drivers, or protocols?
- Does it translate data between domain and technology?
- Would the core still work if this were replaced?

**Heuristics**

- Implements a core port → Adapter.
- Depends on framework → Adapter.
- Translates formats → Adapter.

### Shared Utilities Layer

**Purpose:** Provide neutral helpers shared across layers.

**Rules**

- MAY depend only on standard library.
- MUST remain stateless and deterministic.
- MUST NOT depend on core or adapter logic.

---

### Dependency Summary

| From \ To    | Domain            | Services           | Ports                | Adapters |
| ------------ | ----------------- | ------------------ | -------------------- | -------- |
| **Domain**   | ✅                | ❌                 | ❌                   | ❌       |
| **Services** | ✅                | ✅                 | ✅                   | ❌       |
| **Ports**    | ✅                | ✅                 | ✅                   | ❌       |
| **Adapters** | ✅ (mapping only) | ✅ (call services) | ✅ (implement ports) | ✅       |

---

### Review Checklist

| Concern                      | Question                              | Expected Layer      |
| ---------------------------- | ------------------------------------- | ------------------- |
| Business rule                | Would this rule exist without tech?   | Domain              |
| Multi-aggregate coordination | Is this a use case?                   | Application Service |
| External system I/O          | Do I need a port?                     | SPI Adapter         |
| Input/output handling        | Am I translating for external caller? | API Adapter         |
| Contract definition          | Is this a boundary interface?         | Port                |
| Serialization/parsing        | Am I converting formats?              | Adapter             |

#### Structure

- **Domain** _(Entities/Value Objects)_
  - Pure business data + invariants.
  - **MUST NOT** depend on frameworks, transport, storage, or application services.
- **Application Services** _(Use Cases/Orchestration)_
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

Dependencies **MUST** always point inward toward the domain: **adapter → port → application service → domain.**

##### Domain Layer

- **MUST NOT** depend on any other layer.
- **MUST** express business concepts and invariants.
- **MUST NOT** import technology-specific code.

##### Application Services

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

**Rationale:** Improves scalability, clarity of side effects, and event-sourcing readiness.

#### Structure

| Concern      | Definition                            |
| ------------ | ------------------------------------- |
| **Commands** | Mutate state and enforce invariants.  |
| **Queries**  | Read-only operations for projections. |
| **Models**   | Write and read models may differ.     |

#### Rules

**Commands**

- MUST have single-responsibility handlers enforcing invariants.
- MUST NOT read from projections directly.
- MAY emit domain events via ports.

**Queries**

- MUST be read-only.
- MAY use projections or read models.
- MUST NOT mutate state.

**Models**

- Write/read models MAY differ.
- Synchronization MAY be event-driven or transactional.

#### Prompting Questions

- Does this change domain state? → Command
- Is this purely data retrieval? → Query
- Does it publish events? → Command
- Is the read model optimized for access? → Query

#### Heuristics

- Mutates entities → Command Handler
- Retrieves projections → Query Handler
- Requires join/indexing → Query Adapter
- Keep command and query pipelines isolated.

#### Integration with Hexagonal Architecture

- Commands execute through **API Ports**.
- Command handlers live in **Application Services**.
- Queries execute through separate **API Ports** and read-model adapters.
- SPI Adapters handle persistence and projections.
- The domain enforces invariants but remains CQRS-agnostic.

#### Review Checklist

| Concern              | Question                          | Component           |
| -------------------- | --------------------------------- | ------------------- |
| State mutation       | Does it change aggregates?        | Command Handler     |
| Rule enforcement     | Does it apply invariants?         | Command Handler     |
| Read-only            | Does it fetch projections?        | Query Handler       |
| Different data shape | Is read model optimized?          | Query Adapter       |
| Event emission       | Should events update projections? | Command/App Service |

### CQRS Principle

> Separate what **changes state** from what **observes state**.
> Commands assert; queries observe.
