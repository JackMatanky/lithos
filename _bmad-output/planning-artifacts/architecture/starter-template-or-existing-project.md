# Starter Template or Existing Project

**Starter Template:** golang-standards/project-layout (<https://github.com/golang-standards/project-layout>)

**Rationale:** This community-standard Go project structure provides:

- Well-established directory conventions (`/cmd`, `/internal`, `/pkg`, `/api`, etc.)
- Clear separation between application entry points and library code
- Standard locations for configuration, scripts, and documentation
- Alignment with Go ecosystem best practices and community expectations

**Key Adaptations for Lithos:**

- `/cmd/lithos/` - CLI application entry point (main.go)
- `/internal/` - Private application code (domain, adapters, ports)
- `/pkg/` - Public library code (if needed for future Go module distribution)
- `/testdata/` - Test vault with golden files
- `/.lithos/` - Runtime cache directory (within vaults, not repository)

This layout directly supports the hexagonal architecture by providing clear boundaries between external-facing code (`/cmd`) and internal domain logic (`/internal`), while maintaining flexibility for future library extraction (`/pkg`) per post-MVP roadmap.
