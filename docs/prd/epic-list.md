# Epic List

- **Epic 1: Foundational CLI & Static Template Engine.**
  - **Goal:** Deliver a working CLI that can generate a note from a static template. This includes the basic project scaffolding, a `version` command, and the core `new <template>` command with the Go `text/template` engine.
- **Epic 2: Configuration & Schema Loading.**
  - **Goal:** Introduce configuration file handling (`lithos.yaml`) and the ability to load and parse schema definition files from the vault.
- **Epic 3: Vault Indexing Engine.**
  - **Goal:** Implement the `lithos index` command, building the on-disk cache by parsing the frontmatter of all notes in the vault. This includes defining the core `Storage` interface.
- **Epic 4: Interactive Input Engine.**
  - **Goal:** Implement the interactive features of the CLI, including the `lithos find` command and the `prompt()` and `suggester()` template functions.
- **Epic 5: Schema-Driven Lookups & Validation.**
  - **Goal:** Connect the template engine to the vault index to enable dynamic lookups for suggesters. Implement the core metadata validation logic.
