# CLI `template` Command

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Add the `lithos template` command to the CLI as a thin orchestration adapter over `TemplateService`. This is the end-to-end proof that the whole vertical slice works.

Command shape:
```
lithos template --input <template-name> --output <vault-relative-path> [--var key=value]...
lithos template --input <template-name> --dry-run [--var key=value]...
```

Short forms `-i`, `-o` are accepted. `--var` may be repeated; values containing `=` are split on the first `=` only (e.g. `--var url=https://example.com/foo=bar` → key `url`, value `https://example.com/foo=bar`).

Behavior:
- Normal render: calls `TemplateService::create()`, prints the created vault-relative path to stdout on success.
- Dry-run: calls the dry-run variant, prints the rendered output to stdout without writing any file.
- The CLI adapter maps `TemplateError` variants to explicit user-facing messages — it does not forward raw `TemplateError::to_string()` output.

Deferred (out of scope for this slice):
- Declared inputs / `inputs.*` namespace
- Interactive prompt UX
- Namespaces, query helpers, custom extensions
- Multi-file pack output
- Rich conflict policies (overwrite, skip, rename, merge)

## Acceptance criteria

- [ ] `lithos template --input <name> --output <path>` renders a template and prints the created path
- [ ] `lithos template --input <name> --dry-run` prints the rendered content without creating a file
- [ ] `-i` and `-o` short flags are accepted
- [ ] `--var key=value` is accepted and repeated flags build the context map
- [ ] `--var` values containing `=` are correctly split on the first `=` only
- [ ] Missing `--input` or conflicting `--output`/`--dry-run` flags produce a clear usage error
- [ ] `TemplateError` variants are mapped to user-facing messages (not raw `Display` forwarding)
- [ ] Success output prints the vault-relative path of the created file
- [ ] No `unwrap()` or `panic!` in CLI code
- [ ] Tests cover: render command (normal path), dry-run command, repeated `--var` flags including values with `=`, output path reporting, structured failure paths (missing template, engine error, path validation error, destination exists)

## Blocked by

- `issue-07-template-service.md`
