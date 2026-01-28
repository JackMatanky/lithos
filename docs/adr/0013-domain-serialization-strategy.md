---
name: domain-serialization-strategy
status: proposed
stakeholders: [Development Team, Architects]
date_proposed: 2026-01-14
date_decided: TBD
date_implemented: TBD
---

# ADR 0013: Domain Serialization Strategy

## Context

The Lithos project requires serialization capabilities for multiple purposes:

- **Storage Persistence**: Long-term data storage in Redb database
- **API Communication**: JSON responses for REST APIs and GraphQL
- **Configuration Files**: YAML/TOML for user configuration
- **Network Protocols**: Potential future binary protocols
- **Debugging/Development**: Human-readable formats for logging and development tools
- **Interoperability**: Integration with external systems

Domain models must remain pure and focused on business logic. However, serialization needs span multiple architectural layers, creating tension between convenience and architectural purity.

## Decision

**Domain models MUST derive serde traits for JSON/YAML serialization when they represent API resources, while rkyv remains prohibited in domain.**

### Implementation Guidelines

1. **Serde Requirement**: Domain entities that represent API resources (Note, Schema) MUST derive `serde::{Serialize, Deserialize}`
2. **rkyv Prohibition**: Domain entities SHALL NOT derive rkyv traits under any circumstances
3. **Storage DTOs**: Adapters SHALL provide separate DTOs with rkyv derives for storage
4. **Required Dependencies**: Serde is required for API-facing domain models
5. **Domain Purity Guardian**: Automated tests enforce that only API models have serde derives

### Rationale

#### Why Allow Serde in Domain (Controlled)

**Architectural Benefits**:

- **API Simplicity**: Direct domain model serialization reduces DTO mapping complexity
- **Type Safety**: Compile-time guarantees that API contracts match domain models
- **Developer Experience**: Less boilerplate for simple CRUD APIs
- **Evolutionary Safety**: Domain changes automatically reflected in APIs (with proper versioning)

**Technical Benefits**:

- **Performance**: Zero-copy for JSON serialization in many cases
- **Ecosystem Maturity**: Serde is the de facto standard for Rust serialization
- **Interoperability**: Seamless integration with web frameworks, OpenAPI generators
- **Debugging**: Easy JSON serialization for logging/tracing

#### Why Prohibit rkyv in Domain

**Storage Separation**:

- **Performance Optimization Conflict**: rkyv's zero-copy requirements may constrain domain model design
- **Storage Evolution**: Storage format changes shouldn't require domain model changes
- **Adapter Encapsulation**: rkyv boilerplate belongs in SPI storage adapters only

**From ADR 0002 (Storage - Redb + rkyv)**:

> "rkyv boilerplate must be encapsulated in the adapters/spi/storage layer to protect domain ergonomics"

## Alternatives Considered

### Alternative 1: Complete Serialization Ban (Zero Dependencies)

- **Pros**: Maximum architectural purity, zero coupling
- **Cons**: Significant DTO mapping overhead, reduced developer experience
- **Verdict**: Too restrictive for practical API development

### Alternative 2: Allow Both serde and rkyv in Domain

- **Pros**: Maximum flexibility for all serialization needs
- **Cons**: Dependency bloat, architectural pollution, testing complexity
- **Verdict**: Violates domain purity principles

### Alternative 3: Custom Derives Only

- **Pros**: Domain controls serialization without external dependencies
- **Cons**: Reinventing the wheel, ecosystem isolation
- **Verdict**: Not practical for production systems

### Alternative 4: Application-Layer DTOs Only

- **Pros**: Clean separation, maximum flexibility
- **Cons**: Mapping boilerplate, maintenance overhead, potential for stale DTOs
- **Verdict**: Acceptable but verbose for simple cases

## Technical Validation

### Research Findings

- **rkyv Analysis**: Zero-copy deserialization framework suitable for storage, but creates domain coupling
- **Serde Analysis**: De facto standard for Rust serialization, excellent for APIs, minimal domain impact
- **Use Case Separation**: rkyv excels at storage performance, serde excels at API interoperability

### Compatibility & Performance

- **Hexagonal Alignment**: Serde derives maintain separation, rkyv in domain violates it
- **Performance Impact**: Appropriate tools for each context (zero-copy storage, flexible APIs)
- **Ecosystem Fit**: Serde integrates with web frameworks, rkyv optimizes storage operations

## Consequences

- **Positive**:
  - **Balanced Architecture**: Practical API development without sacrificing purity
  - **Performance**: Appropriate tools for each context (zero-copy storage, flexible APIs)
  - **Date Format Flexibility**: Supports multiple date formats (ISO 8601, Moment.js, custom) in domain
  - **Intelligent Parsing**: Best-effort typing provides type hints while allowing schema flexibility
  - **Developer Experience**: Reduced DTO mapping for simple CRUD APIs with type safety
  - **Ecosystem Integration**: Works with standard Rust web frameworks
- **Negative**:
  - **Parsing Complexity**: Multiple date format support increases domain logic
  - **Type Uncertainty**: Best-effort typing may not match schema expectations
  - **Validation Duplication**: Domain and application layer both validate types
  - **Dependency Management**: Required serde and chrono dependencies increase domain crate size
