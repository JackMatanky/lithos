# Obsidian Templater Reference

Source digest: `docs/refs/digests/obsidian_silentvoid13-templater-digest.txt`

This reference summarizes Templater behavior and syntax that impacts how
Obsidian users expect templating, automation, and metadata mutation to work.

## Core Model

- Templater is a template engine with a custom tag syntax.
- It replaces commands with values at template insert time.
- It can execute JavaScript (with access to Obsidian globals) and system
  commands (if enabled).

Security note:

- JS execution and system commands are not sandboxed; treat templates as code.

## Command Syntax

Commands are delimited by `<%` and `%>`.

- Interpolation command: `<% ... %>`
  - Evaluates and outputs the expression result.
- JavaScript execution command: `<%* ... %>`
  - Executes JS; outputs nothing unless `tR` is modified.

### Whitespace Control

Whitespace control modifiers are part of the opening/closing tag:

- `<%_` trims all whitespace before the command
- `_%>` trims all whitespace after the command
- `<%-` trims one newline before the command
- `-%>` trims one newline after the command

### Dynamic Commands

- Use `<%+ ... %>` to mark a command as dynamic.
- Dynamic commands execute when entering preview mode.
- Known issues; Dataview is recommended for most dynamic use cases.

## Function Invocation

- All functions are accessed under `tp` (e.g. `tp.date.now()`).
- Invocation uses JS call syntax with positional arguments.
- Types must match function expectations (string in quotes, booleans lowercase).

## Execution Command Output (`tR`)

In execution commands (`<%* %>`), output is constructed via `tR`:

- Append: `tR += "text"`
- Replace everything: `tR = ""` (useful for stripping frontmatter in templates)

## Internal Modules (tp.*)

Templater internal functions are grouped into modules:

- `tp.app` access to Obsidian app (useful in scripts)
- `tp.config` access to Templater config
- `tp.date` date helpers and formatting
- `tp.file` file utilities (title, path, creation/modified dates, etc.)
- `tp.frontmatter` frontmatter access by key
- `tp.hooks` lifecycle hooks
- `tp.obsidian` access to Obsidian API helpers
- `tp.system` prompts, suggesters, and system commands
- `tp.web` web utilities

Notes:

- `tp.frontmatter["key with spaces"]` is supported via bracket notation.
- `tp.obsidian` exposes API helpers like `normalizePath` and `requestUrl`.

## tp.system (internal system module)

Functions are asynchronous where noted. All return Promises.

- `tp.system.clipboard()` -> `Promise<string | null>`
  - Reads text from the system clipboard.

- `tp.system.prompt(prompt_text, default_value, throw_on_cancel?, multi_line?)`
  - `throw_on_cancel` default is false; when false, cancel returns `null`.
  - `multi_line` default is false.

- `tp.system.suggester(text_items, items, throw_on_cancel?, placeholder?, limit?)`
  - `text_items` can be `string[]` or a function `(item) => string`.
  - On cancel, returns `null` unless `throw_on_cancel` is true.

- `tp.system.multi_suggester(text_items, items, throw_on_cancel?, title?, limit?)`
  - Same semantics as `suggester` but returns `T[]` (empty array on cancel).

## tp.file (internal file module)

Static (template-time) functions:

- `tp.file.creation_date(format?)` -> string
  - Default format: `YYYY-MM-DD HH:mm` (uses `target_file.stat.ctime`).

- `tp.file.last_modified_date(format?)` -> string
  - Default format: `YYYY-MM-DD HH:mm` (uses `target_file.stat.mtime`).

- `tp.file.cursor(order?)` -> string
  - Inserts a cursor placeholder by emitting a nested cursor command string.

- `tp.file.cursor_append(content)` -> string | undefined
  - Appends content at the active editor selection. Logs error if no editor.

- `tp.file.exists(filepath)` -> Promise<boolean>
  - Uses `normalizePath` and `vault.exists`.

- `tp.file.find_tfile(filename)` -> `TFile | null`
  - Uses `metadataCache.getFirstLinkpathDest(path, "")`.

- `tp.file.folder(absolute?)` -> string
  - Default is folder name; if `absolute` true returns full parent path.

- `tp.file.include(include_link)` -> Promise<string>
  - Accepts `TFile` or a wikilink string `"[[path#subpath]]"`.
  - Reads file content; if subpath provided, slices via `resolveSubpath`.
  - Depth-limited to 10; throws if exceeded.
  - Parsed content is re-run through the Templater parser.

- `tp.file.create_new(template, filename, open_new?, folder?)`
  - Creates a new note from a template; depth-limited to 10.

- `tp.file.move(path, file_to_move?)` -> Promise<string>
  - Ensures directories exist; renames via `fileManager.renameFile`.

- `tp.file.rename(new_title)` -> Promise<string>
  - Validates title: no `\ / :` characters; uses parent path + extension.

- `tp.file.path(relative?)` -> string
  - If `relative` true, returns vault-relative file path.
  - Otherwise returns full OS path; requires FileSystemAdapter on desktop.

- `tp.file.selection()` -> string
  - Returns current editor selection; throws if no active editor.

Dynamic (render-time) functions:

- `tp.file.content` -> string
  - Reads the target file content.

- `tp.file.tags` -> string[] | null
  - Uses `metadataCache` and `getAllTags` on the file cache.

- `tp.file.title` -> string
  - Returns `target_file.basename`.

## tp.date (internal date module)

All functions use `moment` and return formatted strings.

- `tp.date.now(format?, offset?, reference?, reference_format?)`
  - Default `format`: `YYYY-MM-DD`.
  - `offset` can be number (days) or string (moment duration).
  - If `reference` is provided it must parse via `reference_format`.

- `tp.date.tomorrow(format?)` -> string
  - Default format: `YYYY-MM-DD`.

- `tp.date.yesterday(format?)` -> string
  - Default format: `YYYY-MM-DD`.

- `tp.date.weekday(format, weekday, reference?, reference_format?)` -> string
  - `weekday` uses moment's weekday indexing.
  - `reference` must parse if provided.

## Hooks

Hooks allow delayed actions after templates run:

- `tp.hooks.on_all_templates_executed(fn)`
  - Common for post-processing (frontmatter updates, running commands).

## User Functions

Two types of user-defined functions, accessible under `tp.user.*`:

1) Script user functions
   - Loaded from a configured script folder.
   - CommonJS module exporting a function or object of functions.
   - Loaded from a folder; `.js` files only.
   - Scripts are wrapped and executed via `window.eval`.
   - Valid exports:
     - a single function (mapped to `tp.user.<file_basename>`) or
     - an object of functions (mapped to `tp.user.<file_basename>.<fn>`)
   - Any other export type throws an error.
   - Global namespace is available (`app`, `moment`), but not `tp` or `tR`
     unless passed explicitly.

2) System command user functions
   - Enabled in settings.
   - Invoked as `tp.user.<name>({arg1: value1, ...})`.
   - Args passed as environment variables to the command.
   - Internal functions are expanded before execution (e.g. `tp.file.path()`).
   - Command executes with `cwd` set to the vault base path (desktop only).
   - Uses configured `shell_path` and `command_timeout` (seconds).
   - On mobile, returns a constant unsupported message instead of running.

Limitations:

- User functions are not supported on mobile.

## Settings That Affect Behavior

- Template folder location
- Trigger templater on new file creation
  - Controlled by Folder Templates or File Regex Templates
- Template hotkeys
- Startup templates (run on app start, no output)
- User script functions folder
- User system command functions

Folder/Regex rules:

- Folder rules use the deepest match; order irrelevant.
- Regex rules are tested top-to-bottom; first match wins.
- Use catch-all rule (`/` or `.*`) if needed.

## Alignment Notes for Lithos

- Templater establishes user expectations for `tp.*` utilities and
  template-time automation.
- Inline automation can mutate files and frontmatter post-insertion via hooks.
- Execution commands can alter output by manipulating `tR`.
- Templater and Dataview overlap for dynamic data; Dataview is preferred for
  live queries while Templater is for one-time generation.
