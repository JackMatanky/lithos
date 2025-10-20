# Epic 2: Configuration & Schema Loading

This epic introduces the "brains" of Lithos. It enables the CLI to read configuration files and understand the structure of the user's data through schema definitions. This epic implements the SchemaRegistry and SchemaValidator domain services with proper separation of concerns per the hexagonal architecture.

**Dependencies:** Epic 1 (Foundational CLI with Hexagonal Architecture)

## Story 2.1: Enhance Configuration Management with Config Model

As a developer, I want to enhance the ConfigViperAdapter with the full Config model, so that all architectural components have access to structured configuration.

### Acceptance Criteria

- 2.1.1: The Config model from `docs/architecture/data-models.md#config` is implemented with VaultPath, TemplatesDir, SchemasDir, CacheDir, and LogLevel fields.
- 2.1.2: ConfigViperAdapter searches for `lithos.yaml` in current directory and parent directories per Viper behavior.
- 2.1.3: Configuration includes sensible defaults as specified in the data models documentation.
- 2.1.4: Config validation ensures VaultPath exists and is readable at load time.

## Story 2.2: Implement Schema Domain Model

As a developer, I want to implement the Schema model from the architecture, so that I have a foundation for schema definitions.

### Acceptance Criteria

- 2.2.1: `internal/domain/` contains Schema model with Name, Extends, Excludes, Properties, and ResolvedProperties fields per `docs/architecture/data-models.md#schema`.
- 2.2.2: Schema model includes proper JSON tags for serialization.
- 2.2.3: Schema model has unit tests for field validation and initialization.
- 2.2.4: Schema model implements proper inheritance semantics with Extends field supporting string-based parent references.
- 2.2.5: Schema model supports Excludes field for subtractive inheritance (removing parent properties).

## Story 2.3: Implement Property Domain Models

As a developer, I want to implement the Property models and PropertySpec from the architecture, so that I have robust property definitions.

### Acceptance Criteria

- 2.3.1: Property model with Name, Required, Array, and Spec fields is implemented following the architecture specification.
- 2.3.2: PropertySpec interface and concrete implementations (StringPropertySpec, NumberPropertySpec, DatePropertySpec, FilePropertySpec, BoolPropertySpec) are created.
- 2.3.3: PropertyBank model with Properties map and Location field is implemented for reusable property definitions.
- 2.3.4: All property models include unit tests for validation and behavior.

## Story 2.4: Implement Schema Engine Port and Adapter

As a developer, I want to implement the SchemaEnginePort and SchemaLoaderAdapter, so that schemas can be loaded from JSON files following the architecture.

### Acceptance Criteria

- 2.4.1: `internal/ports/spi/` contains SchemaEnginePort interface with LoadSchemas and LoadPropertyBank methods.
- 2.4.2: `internal/adapters/spi/schema/` contains SchemaLoaderAdapter implementing SchemaEnginePort.
- 2.4.3: Adapter scans for `.json` files in `schemas/` directory per `docs/architecture/tech-stack.md#json-processing`.
- 2.4.4: Property bank files are loaded from `schemas/properties/` directory with `$ref` resolution.
- 2.4.5: JSON parsing uses Go stdlib `encoding/json` with structured error handling.

## Story 2.5: Implement Basic Schema Registry Service

As a developer, I want to implement the basic SchemaRegistry domain service, so that schema loading and registry management follow the architectural patterns.

### Acceptance Criteria

- 2.5.1: `internal/app/schema/` contains SchemaRegistry implementing the interface from `docs/architecture/components.md#domain-services`.
- 2.5.2: Service loads schemas via SchemaEnginePort per `docs/architecture/components.md#schemaengineport`.
- 2.5.3: Service uses Registry package for thread-safe schema storage per `docs/architecture/components.md#shared-internal-packages`.
- 2.5.4: SchemaRegistry provides Get(name string) method for retrieving schemas by name with proper error handling.
- 2.5.5: Service integrates with ConfigPort to access schema directory configuration.

## Story 2.6: Implement Inheritance Resolution

As a developer, I want to implement inheritance resolution in the SchemaRegistry, so that schema hierarchies are properly resolved.

### Acceptance Criteria

- 2.6.1: Builder pattern resolves inheritance by: loading all schemas → building dependency graph → detecting cycles → resolving in topological order.
- 2.6.2: ResolvedProperties are computed by merging parent properties, applying Excludes, then merging child Properties.
- 2.6.3: Inheritance resolution handles multi-level chains (C extends B, B extends A) with proper property override priority.
- 2.6.4: Cycle detection provides clear error messages identifying the circular dependency path.
- 2.6.5: ResolvedProperties are immutable after inheritance resolution to ensure thread safety.

## Story 2.7: Implement Schema Validator Service

As a developer, I want to implement the SchemaValidator as a separate domain service, so that validation logic is decoupled from schema loading.

### Acceptance Criteria

- 2.7.1: `internal/app/schema/` contains SchemaValidator implementing validation interface.
- 2.7.2: Service validates Frontmatter against Schema using PropertySpec polymorphism.
- 2.7.3: Validation checks Required fields, Array constraints, and type-specific PropertySpec rules.
- 2.7.4: Returns structured ValidationError types from `internal/shared/errors/`.
