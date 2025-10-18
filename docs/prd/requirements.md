# Requirements

## Functional

### Core Engine & Templates

1. **FR1: Template Composition:** Users must be able to create templates composed of multiple, reusable sections, with or without variables. The system must prevent errors from missing sections or circular references.
2. **FR2: Non-Interactive Execution:** The CLI must be executable via a simple command (e.g., `lithos new <template>`), without requiring flags for template-defined inputs, to support simple automation.
3. **FR3: Template Engine Core Library:** The template engine must expose relevant Go standard library packages (e.g., `path`, `strings`, `time`) and implement a library of specialized PKM-specific functions. This library must include:
   - **String Formatting:** Advanced casing options and functions to generate wikilinks (e.g., `[[name|alias]]`).
   - **Interactive Functions:** Implementations for `prompt()` (free-text) and `suggester()` (list selection). The suggester must accept either a simple list of strings or a key-value map.
   - **Query Functions:** A generic `query()` function for flexible index lookups, and a convenience `lookup()` function for common queries.
4. **FR4: Template Validation (Linting):** (Stretch Goal for MVP) The system should provide a `lint` command to validate that templates do not contain fields that contradict their governing schema.

### Schema & Data

1. **FR5: Metadata Class Integration:** The system must load metadata class definitions from schema files. These classes can extend one another, inheriting fields and constraints. Schema implementation is a prerequisite for both lookup and validation functionalities.
2. **FR6: Frontmatter Handling:** The CLI must be able to read, merge, and write YAML frontmatter. Unknown fields must be preserved.
3. **FR7: Schema-Based Validation:** The CLI must validate that fields present in a template do not contradict the corresponding schema. It must enforce correct types for both single values and arrays of values. Supported types include:
   - Standard types: `boolean`, `integer`, `float`, `string`, `date`, `file`.
   - `file` type (a file path or wikilink).
   - Custom `date` types using format strings that conform to Go's standard library `time` package layouts (e.g., "2006-01-02").
   - Custom `string` types using a regex pattern.
4. **FR8: Field Value Sourcing via Custom Queries:** For `file` type fields within a schema, users can define a custom query that provides the options for that field. This simplifies the process of creating lookups within templates.
5. **FR9: Vault-Wide Lookup & Indexing:** The system must maintain an index of notes, retrievable by file path, file basename, or a schema-defined primary key. The result of a successful lookup can populate any YAML field in the frontmatter.

### User Interaction

1. **FR10: Interactive Input Engine:** The system must support interactive input during template generation, defined within the template file. This includes:
   - **Prompts:** For free-text input.
   - **Suggesters:** For selection from a list of options (sourced from schemas, lookups, or fixed values).

## Non-Functional

1. **NFR1: Platform Support (MVP):** The CLI binary must be fully supported and tested on macOS. If Linux support can be implemented without significant additional complexity, it will be included; otherwise, it will be deferred to a later phase.
2. **NFR2: Portability:** Lithos must be distributed as a single, standalone binary for each target OS with no external runtime dependencies.
3. **NFR3: Performance & Ergonomics:**
   - **Template Rendering:** The typical time to generate a single note from a template that does not require index lookups should be less than 300 milliseconds.
   - **Indexing:** The performance of the vault indexing operation must be tracked and benchmarked, but there is no strict time-to-complete requirement for the MVP.
   - **Time to First Note:** A new user, starting from a fresh install with a sample template pack, should be able to generate their first note within 2 minutes.
4. **NFR4: Index Architecture:** The system will use a hybrid indexing model. A persistent on-disk cache will be stored within the vault (e.g., in a `.lithos` directory). This will contain the full YAML frontmatter for all notes. A smaller, read-optimized in-memory index will also be used for performance-critical lookups.
5. **NFR5: Index Freshness (MVP):** The index must be updatable via a manual command (e.g., `lithos index`). Automatic incremental updates are a post-MVP goal.
