# Issue Tracker

This repository uses **local markdown** as the issue tracker.

## Location

Issues live under `.scratch/<feature>/` in this repository.

## Workflow

- Create one markdown file per issue in the appropriate `.scratch/<feature>/` folder.
- Use a consistent filename prefix (for example: `001-short-title.md`).
- Keep issue state in frontmatter or in a dedicated status section inside the file.
- When a skill says "create/update an issue", it should create or update these local markdown files instead of calling remote issue APIs.

## Conventions

- Keep issue content self-contained so an AFK agent can execute it without extra chat context.
- Include acceptance criteria and relevant links/paths in each issue file.
- Use triage labels from `docs/agents/triage-labels.md` as status metadata in the issue file.
