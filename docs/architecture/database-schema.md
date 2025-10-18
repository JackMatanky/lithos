# Database Schema

Lithos persists structured metadata as JSON documents inside `.lithos/cache/`. Each note is stored in a single file containing file identity and parsed frontmatter:

```json
{
  "file": {
    "path": "/vault/projects/project-alpha.md",
    "basename": "project-alpha",
    "folder": "projects",
    "modTime": "2025-09-28T14:22:31Z"
  },
  "frontmatter": {
    "fileClass": "project",
    "fields": {
      "title": "Project Alpha",
      "status": "active",
      "owner": "jack",
      "created": "2025-07-02",
      "tags": ["project", "priority/a"]
    }
  }
}
```

- **Primary Index:** File path (absolute). Filename derived via hash to keep cache directories manageable.
- **Secondary Indices:** In-memory maps maintained by QueryService (by fileClass, directory, selected fields). Post-MVP, these can spill to persistent key-value stores (e.g., BoltDB) without changing the JSON shape.
- **Schema Storage:** Schemas and property banks live under `schemas/*.json` and `schemas/_fields/*.json`, mirroring the `Schema` and `Property` models documented earlier.
- **Hybrid Model:** Disk JSON provides durability while QueryService maintains read-optimized in-memory indices. This satisfies NFR4’s hybrid cache requirement today and keeps the door open for future BoltDB-backed adapters without touching the domain model.

**Backup & Recovery:** The Obsidian vault remains the system of record, so teams should continue their standard Git/backup routines for vault content. The `.lithos/cache/` directory is disposable—delete it and rerun `lithos index` to regenerate if corruption or drift is detected. Optional cache snapshots can be captured alongside regular vault backups, but they are not required for recovery.

---
