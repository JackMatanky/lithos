# Epic List

**Architecture Version:** Based on docs/architecture/ v0.6.8 (Oct 26, 2025)

- **Epic 1: Foundational CLI with Hexagonal Architecture.**
  - **Goal:** Deliver a runnable Go application that is correctly structured with a hexagonal architecture from the start. It includes the core, non-interactive template rendering capability.
- **Epic 2: Configuration & Schema Loading.**
  - **Goal:** Introduce configuration file handling (`lithos.yaml`) and the ability to load and parse schema definition files from the vault.
- **Epic 3: Vault Indexing Engine.**
   - **Goal:** Implement the `lithos index` command, building the on-disk cache by parsing the frontmatter of all notes in the vault. This includes defining the cache and vault port interfaces (CacheWriter, CacheReader, VaultReader, VaultWriter) following CQRS pattern per architecture v0.6.8.
- **Epic 4: Interactive Input Engine.**
  - **Goal:** Implement the interactive features of the CLI, including the `lithos find` command and the `prompt()` and `suggester()` template functions.
- **Epic 5: Schema-Driven Lookups & Validation.**
  - **Goal:** Connect the template engine to the vault index to enable dynamic lookups for suggesters. Implement the core metadata validation logic.
