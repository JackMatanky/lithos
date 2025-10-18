# External APIs

Lithos operates entirely on the local file system for the MVP and does not call any external APIs or web services. Future roadmap items (e.g., a remote template registry or schema catalog) would integrate via `TemplateRepositoryPort` or `SchemaEnginePort`, allowing new adapters to be added without altering the domain core.

---
