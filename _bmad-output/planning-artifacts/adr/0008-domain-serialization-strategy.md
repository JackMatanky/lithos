# ADR 0008: Domain Serialization Strategy

## Status
Accepted

## Context
The project needs to handle serialization for various purposes:
- Storage persistence (database)
- API communication (JSON)
- Configuration files (YAML/TOML)
- Network protocols
- In-memory data transfer

Domain models should remain pure and not be coupled to specific serialization formats. However, the initial architecture allowed "optional serde" in domain models for "external interface serialization" while prohibiting rkyv.

## Decision
**Domain models SHALL have ZERO external dependencies and SHALL NOT derive any serialization traits.**

All serialization, including JSON/YAML APIs and configuration files, SHALL be handled by:
1. **Application Layer DTOs** for API/external interfaces
2. **Adapter DTOs** for storage and external system integration

### Rationale

#### Against Serde in Domain
- **Architectural Purity**: Domain models should be serialization-format agnostic
- **Dependency Creep**: Allowing one serialization library opens door to others
- **Testing Complexity**: Serialization concerns pollute domain testing
- **Evolution Freedom**: Domain can evolve without breaking serialization contracts

#### Against rkyv in Domain
- **Storage Coupling**: rkyv is optimized for storage, not domain ergonomics
- **Zero-Copy Trade-offs**: Domain models shouldn't be constrained by storage format requirements
- **Adapter Encapsulation**: rkyv boilerplate belongs in SPI storage adapters only

## Consequences

### Positive
- **Pure Domain**: Domain models remain completely independent of I/O and serialization
- **Flexibility**: Can change serialization formats without affecting domain logic
- **Testability**: Domain tests focus purely on business logic
- **Performance**: Adapters can optimize serialization for specific use cases

### Implementation Requirements
- **Application DTOs**: Create separate structs for API responses/requests
- **Storage DTOs**: Adapters provide rkyv-compatible DTOs that convert to/from domain models
- **Configuration**: Use adapter DTOs for config file parsing
- **Domain Purity Guardian**: Automated tests enforce zero dependencies in domain crate

### Migration
Existing domain models with serde derives must be refactored:
1. Remove serde derives from domain entities
2. Create application DTOs with serde derives
3. Create storage DTOs with rkyv derives in adapters
4. Add conversion methods between domain and DTOs

## Alternatives Considered

### 1. Allow serde, Prohibit rkyv (Current)
- Rejected: Creates architectural inconsistency and dependency creep

### 2. Allow Both serde and rkyv in Domain
- Rejected: Maximum coupling and dependency bloat

### 3. Custom Derives Only
- Rejected: Still couples domain to serialization concerns

## References
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [Domain-Driven Design](https://domainlanguage.com/ddd/)
- Architecture.md#Domain Purity Requirements
