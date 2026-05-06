# Issue tracker

This repository uses a local-markdown issue tracker.

## Location

- Issue files live under `.scratch/<feature>/` in this repo.
- Each issue is a markdown file describing one independently actionable slice.

## Workflow

- Create: add a new markdown issue file under the relevant `.scratch/<feature>/` folder.
- Update: edit the same issue file as triage progresses.
- Close: mark completion status in the issue file and link any implementation artifacts (branch, commit, PR) when applicable.

## Conventions

- Prefer one issue per vertical slice.
- Keep acceptance criteria explicit and testable.
- Preserve issue history in-file (append updates rather than overwriting prior context when possible).

## Tooling note for agent skills

Skills that normally call hosted issue APIs should read and write `.scratch/<feature>/` markdown issues instead.
