# Obsidian API Reference (obsidian.d.ts)

Source: https://raw.githubusercontent.com/obsidianmd/obsidian-api/master/obsidian.d.ts

This file is a concentrated, data-focused reference for Lithos. It captures
the vault/file model, metadata cache shapes, link and subpath semantics, and
the event lifecycle that Obsidian exposes. UI-specific APIs are omitted unless
they describe data shapes or parsing context.

## Conventions and Shared Types

- `normalizePath(path: string): string` normalizes paths for adapter calls.
- `Loc` represents a position in a Markdown file: `line` (0-based), `col`, and
  `offset` (character count from file start).
- `Pos` represents a range in a Markdown file: `{ start: Loc, end: Loc }`.
- `SubpathResult` includes a range: `{ start: Loc, end: Loc | null }`.

## File and Folder Types

- `TAbstractFile`
  - `vault: Vault`
  - `path: string`
  - `name: string`
  - `parent: TFolder | null`

- `TFile` extends `TAbstractFile`
  - `stat: FileStats`
  - `basename: string`
  - `extension: string`

- `TFolder` extends `TAbstractFile`
  - `children: TAbstractFile[]`
  - `isRoot(): boolean`

- `FileStats`
  - `ctime: number` (ms since epoch)
  - `mtime: number` (ms since epoch)
  - `size: number` (bytes)

- `Stat` (used by `DataAdapter.stat`)
  - `type: "file" | "folder"`
  - `ctime: number` (seconds since epoch)
  - `mtime: number` (seconds since epoch)
  - `size: number` (bytes)

## Vault API (high-level file access)

- `Vault.adapter: DataAdapter`
- `Vault.configDir: string` (usually `.obsidian`)
- `getName(): string`
- `getFileByPath(path): TFile | null`
- `getFolderByPath(path): TFolder | null`
- `getAbstractFileByPath(path): TAbstractFile | null`
  - Path is vault-absolute with extension, case sensitive.
- `getRoot(): TFolder`
- `create(path, data, options?): Promise<TFile>`
- `createBinary(path, data, options?): Promise<TFile>` (throws if exists)
- `createFolder(path): Promise<TFolder>` (throws if exists)
- `read(file: TFile): Promise<string>` (direct read for modification)
- `cachedRead(file: TFile): Promise<string>` (read for display; cached)
- `readBinary(file: TFile): Promise<ArrayBuffer>`
- `getResourcePath(file: TFile): string` (URI for embedding)
- `modify(file, data, options?): Promise<void>`
- `modifyBinary(file, data, options?): Promise<void>`
- `append(file, data, options?): Promise<void>`
- `appendBinary(file, data, options?): Promise<void>`
- `process(file, fn, options?): Promise<string>`
  - Atomically read/modify/write; returns final text.
- `delete(file, force?): Promise<void>` (force allows hidden children)
- `trash(file, system: boolean): Promise<void>`
- `rename(file, newPath): Promise<void>`
  - Use `FileManager.renameFile` if link updates are desired.
- `copy(file, newPath): Promise<T>`
- `getAllLoadedFiles(): TAbstractFile[]`
- `getAllFolders(includeRoot?): TFolder[]`
- `getMarkdownFiles(): TFile[]`
- `getFiles(): TFile[]`
- `Vault.recurseChildren(root, cb): void`

Vault events (payload is `TAbstractFile` unless noted):

- `on("create", (file) => ...)`
  - Fires for existing files on vault load unless registered after
    `Workspace.onLayoutReady`.
- `on("modify", (file) => ...)`
- `on("delete", (file) => ...)`
- `on("rename", (file, oldPath) => ...)`

## DataAdapter (low-level filesystem interface)

Prefer `Vault` APIs where possible. Adapter methods use normalized paths.

- `getName(): string`
- `exists(normalizedPath, sensitive?): Promise<boolean>`
- `stat(normalizedPath): Promise<Stat | null>`
- `list(normalizedPath): Promise<ListedFiles>` (non-recursive)
- `read(normalizedPath): Promise<string>`
- `readBinary(normalizedPath): Promise<ArrayBuffer>`
- `write(normalizedPath, data, options?): Promise<void>`
- `writeBinary(normalizedPath, data, options?): Promise<void>`
- `append(normalizedPath, data, options?): Promise<void>`
- `appendBinary(normalizedPath, data, options?): Promise<void>`
- `process(normalizedPath, fn, options?): Promise<string>`
- `getResourcePath(normalizedPath): string`
- `mkdir(normalizedPath): Promise<void>`
- `trashSystem(normalizedPath): Promise<boolean>`
- `trashLocal(normalizedPath): Promise<void>`
- `rmdir(normalizedPath, recursive): Promise<void>`
- `remove(normalizedPath): Promise<void>`
- `rename(normalizedPath, normalizedNewPath): Promise<void>`
- `copy(normalizedPath, normalizedNewPath): Promise<void>`

- `DataWriteOptions`
  - `ctime?: number` (ms since epoch)
  - `mtime?: number` (ms since epoch)

## FileManager (link-aware file operations)

- `getNewFileParent(sourcePath, newFilePath?): TFolder`
  - Resolves new-file location based on user settings.
- `renameFile(file, newPath): Promise<void>`
  - Renames/moves a file and updates links per user settings.
- `promptForDeletion(file): Promise<boolean>`
- `trashFile(file): Promise<void>`
- `generateMarkdownLink(file, sourcePath, subpath?, alias?): string`
  - Produces link text respecting user preferences and relative paths.
- `processFrontMatter(file, fn, options?): Promise<void>`
  - Atomically mutates frontmatter; throws `YAMLParseError` on parse failure.
- `getAvailablePathForAttachment(filename, sourcePath?): Promise<string>`
  - Resolves a unique attachment path, ensuring parent directories exist.

## MetadataCache (parsed metadata and link resolution)

- `getFirstLinkpathDest(linkpath, sourcePath): TFile | null`
  - Returns best match for a link path.
- `getFileCache(file): CachedMetadata | null`
- `getCache(path): CachedMetadata | null`
- `fileToLinktext(file, sourcePath, omitMdExtension?): string`
  - If name is unique, uses filename; else uses full path.
- `resolvedLinks: Record<string, Record<string, number>>`
  - Maps source path -> destination path -> link count.
- `unresolvedLinks: Record<string, Record<string, number>>`
  - Maps source path -> unresolved destination -> count.

MetadataCache events:

- `on("changed", (file, data, cache) => ...)`
  - Fired when a file is indexed and cache is updated.
  - Not fired on rename; use the vault rename event.
- `on("deleted", (file, prevCache) => ...)`
  - `prevCache` can be null if not cached before deletion.
- `on("resolve", (file) => ...)`
  - Fired when `resolvedLinks`/`unresolvedLinks` updated for a file.
- `on("resolved", () => ...)`
  - Fired when all files have been resolved after modifications.

## CachedMetadata (per-file parsed data)

- `links?: LinkCache[]`
- `embeds?: EmbedCache[]`
- `tags?: TagCache[]`
- `headings?: HeadingCache[]`
- `footnotes?: FootnoteCache[]`
- `footnoteRefs?: FootnoteRefCache[]`
- `referenceLinks?: ReferenceLinkCache[]`
- `sections?: SectionCache[]`
  - Root-level Markdown blocks (paragraphs, lists, headings, etc).
- `listItems?: ListItemCache[]`
- `frontmatter?: FrontMatterCache`
- `frontmatterPosition?: Pos`
- `frontmatterLinks?: FrontmatterLinkCache[]`
- `blocks?: Record<string, BlockCache>`

## Cache Item Types

All cache items include `position: Pos` via `CacheItem` unless noted.

- `LinkCache` extends `ReferenceCache`
- `EmbedCache` extends `ReferenceCache`

- `Reference`
  - `link: string`
  - `original: string` (not available on Publish)
  - `displayText?: string`

- `ReferenceCache` = `Reference` + `CacheItem`

- `ReferenceLinkCache`
  - `id: string`
  - `link: string`

- `TagCache`
  - `tag: string`

- `HeadingCache`
  - `heading: string`
  - `level: number` (1-6)

- `BlockCache`
  - `id: string`

- `ListItemCache`
  - `id?: string` (block id for list item)
  - `task?: string`
    - `' '` is incomplete; any other character is complete.
  - `parent: number`
    - Line number of parent list item; negative value indicates root list.

- `SectionCache`
  - `id?: string` (block id for section)
  - `type: string`
    - Known examples: `blockquote`, `callout`, `code`, `heading`, `list`,
      `paragraph`, `table`, `yaml`, etc. Type is not exhaustive.

- `FrontMatterCache`
  - `[key: string]: any`

- `FrontmatterLinkCache` extends `Reference`
  - `key: string` (frontmatter key containing the link)

- `FootnoteCache`
  - `id: string`

- `FootnoteRefCache`
  - `id: string`

## Subpath Resolution

- `resolveSubpath(cache, subpath)` returns one of:
  - `HeadingSubpathResult`
    - `type: "heading"`
    - `current: HeadingCache`
    - `next: HeadingCache`
  - `BlockSubpathResult`
    - `type: "block"`
    - `block: BlockCache`
    - `list?: ListItemCache`
  - `FootnoteSubpathResult`
    - `type: "footnote"`
    - `footnote: FootnoteCache`

## Frontmatter Helpers

- `getFrontMatterInfo(content): FrontMatterInfo`
  - `exists: boolean`
  - `frontmatter: string`
  - `from: number` (start of frontmatter content, excluding `---`)
  - `to: number` (end of frontmatter content, excluding `---`)
  - `contentStart: number` (offset where frontmatter block ends, including `---`)

- `stringifyYaml(obj): string`

## Link Helpers

- `getLinkpath(linktext): string`
  - Extracts the path portion from a wikilink without `[[...]]`.
- `stripHeading(heading): string`
  - Normalizes a heading for link matching.
- `stripHeadingForLink(heading): string`
  - Prepares headings for linking by removing link-breaking characters.
- `getAllTags(cache): string[] | null`
  - Combines frontmatter tags and inline tags into a single array.

## Markdown Rendering Context (data-relevant)

- `MarkdownFileInfo`
  - `app: App`
  - `file: TFile | null`
  - `editor?: Editor`

- `MarkdownRenderer.render(app, markdown, el, sourcePath, component)`
  - `sourcePath` is required for correct relative link resolution.

- `MarkdownView`
  - `editor: Editor`
  - `currentMode: MarkdownSubView`
  - `previewMode: MarkdownPreviewView`
  - `getViewData(): string`
  - `setViewData(data, clear): void`

## Lithos Alignment Notes

- Treat `TFile.path` as the canonical vault-absolute path identity.
- Store file stats (`ctime`, `mtime`, `size`) for staleness checks.
- Align metadata storage with `CachedMetadata` to ensure link, tag, heading,
  block, list, and frontmatter parity.
- Link graph should respect `resolvedLinks`/`unresolvedLinks` semantics.
- Subpath resolution (heading/block/footnote) is a separate phase and should
  be modeled as such in Lithos pipelines.
