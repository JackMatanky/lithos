# Infrastructure and Deployment

- **Distribution Strategy:** Ship signed binaries via GitHub Releases using `goreleaser`. CI pipeline runs unit/integration tests before packaging macOS (amd64/arm64) and Linux binaries.
- **Infrastructure as Code:** Not required for MVP; future distribution automation lives in `.github/workflows/`.
- **Environments:**
  - *Local:* Go toolchain with `.envrc` or Just commands.
  - *CI:* GitHub Actions runners executing lint/tests and release jobs.
  - *Release:* GitHub Releases page hosting artifacts and checksums.
- **Deployment Rollback:** Re-tag previous version and re-run `goreleaser --clean`; communicate downgrade instructions in release notes.
- **Observability:** Structured logs via zerolog (JSON or pretty) feed into terminal; optional integration with system log collectors is future work.

---
