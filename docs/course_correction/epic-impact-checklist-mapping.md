# Epic Impact Assessment - Checklist Mapping

**Purpose**: Verify understanding of what needs to be incorporated from prior sections into Epic Impact Assessment before rewriting.

---

## Group 1: Validation Architecture

### From Actionable Insights (AI-1.3)

**AI-1.3: Extract MarkdownParserPort** (Lines 5829-5891)
- [ ] Create NEW PORT: `MarkdownParserPort` in `/internal/ports/spi/markdown.go`
- [ ] Port should have `ParseMetadata(ctx, content []byte) (*domain.NoteMetadata, error)`
- [ ] Create NEW ADAPTER: `GoldmarkParserAdapter` in `/internal/adapters/spi/markdown/goldmark_parser.go`
- [ ] Adapter implements MarkdownParserPort
- [ ] **NOT** just moving to VaultReaderAdapter - it's a dedicated port/adapter
- [ ] FrontmatterService uses MarkdownParserPort (injected dependency)
- [ ] domain.NoteMetadata includes: Frontmatter, Links, Headings, Tags, Backlinks

**Question**: Should Epic Impact Assessment create:
1. Story 3.7.1: Create MarkdownParserPort and GoldmarkParserAdapter
2. Story 3.7.2: Refactor FrontmatterService to use MarkdownParserPort
3. Story 3.7.3: Enrich Frontmatter entity (from Entity Review)
4. Story 3.7.4: Enrich Note entity (from Entity Review)

---

## Group 2: Storage Architecture, CQRS & DTOs

### From Actionable Insights (AI-1.1, AI-1.2)

**AI-1.1: Eliminate FileMetadata Duplication** (Lines 5750-5788)
- [ ] VaultFile should use `fs.FileInfo` directly (not duplicate fields)
- [ ] VaultFile structure: `Path string, Info fs.FileInfo, Content []byte`
- [ ] Computed methods: `Basename()`, `Folder()`, `Ext()`, `ModTime()`, `Size()`
- [ ] Remove: Basename, Folder, Ext, ModTime, Size fields (use fs.FileInfo instead)

**AI-1.2: Adopt Vault-Relative Paths** (Lines 5791-5826)
- [ ] Store paths relative to vault root: "notes/meeting.md" not "/Users/jack/vault/notes/meeting.md"
- [ ] Add VaultID field for multi-vault support
- [ ] Helper: `AbsolutePath(vaultRoot string) string`
- [ ] Helper: `NormalizePath(absPath, vaultRoot string) (string, error)` with filepath.ToSlash

### From Gap Analysis Lines 920-930

**DTO Breakdown Strategy** (Line 927-929):
- [ ] FilePathDTO: Path, Basename, Folder, Ext
- [ ] FileDatesDTO: ModTime, CreatedTime, IndexTime
- [ ] FrontmatterDTO: All Fields, title, aliases, file_class

**Question**: Should we use:
- Option A: Single VaultFile with fs.FileInfo + computed methods (AI-1.1)?
- Option B: Decomposed FilePathDTO, FileDatesDTO, FrontmatterDTO (Line 927-929)?
- Option C: Both - VaultFile uses fs.FileInfo, but BoltDB/SQLite use decomposed DTOs?

### From Actionable Insights (AI-1.4, AI-2.1)

**AI-1.4: Implement MetadataQueryPort** (Lines 5893-5926)
- [ ] Create NEW PORT: `MetadataQueryPort` in `/internal/ports/spi/metadata_query.go`
- [ ] Methods: QueryByTag, QueryByLink, QueryByHeading, QueryByFileClass
- [ ] Enables O(1) indexed queries vs O(n) scanning
- [ ] BoltDB adapter: Use secondary index buckets "indices:by_tag", "indices:by_link"
- [ ] SQLite adapter: Use schema-driven views with indexes

**AI-2.1: Implement MetadataCache Service** (Lines 5932-6007)
- [ ] Create `MetadataCacheService` in `/internal/app/metadata/cache_service.go`
- [ ] NoteMetadataCache struct: Path, VaultID, Frontmatter, Links, Headings, Tags, LastParsed, ContentChecksum
- [ ] Method: `GetOrParse(ctx, path) (*NoteMetadataCache, error)` - check cache, verify checksum, parse if stale
- [ ] Uses MarkdownParserPort (not goldmark directly)
- [ ] Separates parsed metadata cache from file content (memory efficiency)

### From Actionable Insights (AI-2.2)

**AI-2.2: Separate File vs Content DTOs** (Lines 6009-6035)
- [ ] VaultFile for full file+content: Has Content []byte field
- [ ] VaultFileMeta for metadata only: NO Content field (faster scans)
- [ ] VaultScannerPort methods:
  - `ScanAll(ctx) ([]dto.VaultFileMeta, error)` - metadata only, fast
  - `ScanWithContent(ctx) ([]dto.VaultFile, error)` - when content needed
- [ ] Memory efficiency: Don't load 1MB files when only need Path/ModTime

**Question**: Does this replace the FilePathDTO/FileDatesDTO breakdown, or complement it?

### From Entity Review (VaultFile Lines 6681-6859)

**VaultFile DTO Redesign** (Lines 6751-6836):
- [ ] Phase 1: Leverage fs.FileInfo (Gap 1.1, 7.1 resolution)
  - VaultFileInfo struct: VaultPath string, Info fs.FileInfo, Basename/Ext/MimeType computed
  - Use filepath.ToSlash for cross-platform paths
- [ ] Phase 2: Separate Metadata from Content (Gap 1.3 resolution)
  - VaultFileMeta: metadata only
  - VaultFileWithContent: metadata + content
- [ ] Phase 3: Storage-Specific DTOs (Gap 1.1, Issue D2, A4 resolution)
  - BoltDBMetadata: Path, Basename, Aliases, FileClass, ModTime
  - SQLiteMetadata: Path, Frontmatter map, ModTime, Size
  - Conversion functions: VaultFileToBeoltDBMetadata, VaultFileToSQLiteMetadata

**Question**: Is this the final design we should use in Epic Impact Assessment?

### From Sprint Change Proposal (Lines 941-945)

**Question 4 Status**: ❌ NOT FINALIZED - multiple decomposition strategies possible
- Example Options: FilePathDTO, FileDatesDTO, FrontmatterDTO OR storage-specific structs
- Research Needed: Obsidian patterns inform final design

**Question**: Which strategy should Epic Impact Assessment stories implement?

---

## Group 6: Template System

### From Actionable Insights (AI-2.3)

**AI-2.3: Use text/template Composition** (Lines 6038-6088)
- [ ] Replace custom template caching with stdlib composition
- [ ] BEFORE: `compiled map[domain.TemplateID]cachedTemplate` with custom caching
- [ ] AFTER: Single `root *template.Template` with all templates in namespace
- [ ] Use `template.New("root").Funcs(funcMap)` then `root.New(templateID).Parse(content)`
- [ ] Rendering: `root.ExecuteTemplate(&buf, templateID, nil)`
- [ ] Simplifies code, leverages stdlib caching, enables template composition
- [ ] Gap 5.1 resolution

**From Entity Review Summary** (Line 6872):
- Entity Review says: "Template | Intentional | ✅ Good | None (intentionally anemic)"
- But AI-2.3 shows we should refactor to use stdlib composition
- **Contradiction**: Entity Review says "intentionally anemic" is GOOD, but AI-2.3 says REFACTOR

**Question**:
1. Is Template being "intentionally anemic" actually WRONG (should use stdlib composition)?
2. Should Epic Impact Assessment create story for AI-2.3 template refactoring?
3. Or defer to Epic 5 as suggested in Group 6 impact assessment I wrote?

---

## Cross-Cutting Questions

### Question 1: DTO Architecture Final Decision

We have MULTIPLE DTO strategies identified:
1. **AI-1.1**: VaultFile with fs.FileInfo + computed methods
2. **Line 927-929**: FilePathDTO, FileDatesDTO, FrontmatterDTO decomposition
3. **AI-2.2**: VaultFileMeta vs VaultFileWithContent separation
4. **Entity Review Lines 6815-6836**: BoltDBMetadata, SQLiteMetadata storage-specific DTOs

**Which combination should Epic Impact Assessment implement?**

My interpretation:
- Use AI-1.1 + AI-1.2 for VaultFile base (fs.FileInfo + vault-relative paths)
- Use AI-2.2 for VaultFileMeta vs VaultFileWithContent (content separation)
- Use Entity Review Phase 3 for BoltDBMetadata/SQLiteMetadata (storage-specific)
- FilePathDTO/FileDatesDTO breakdown may be internal to storage DTOs?

**Is this correct?**

### Question 2: MarkdownParserPort Scope

AI-1.3 shows MarkdownParserPort should return `domain.NoteMetadata` with:
- Frontmatter map[string]any
- Links []domain.Link
- Headings []domain.Heading
- Tags []string
- Backlinks []domain.Backlink

**Questions**:
1. Do domain.Link, domain.Heading, domain.Backlink entities exist yet?
2. Should Epic Impact Assessment create these domain entities?
3. Or should MarkdownParserPort initially only return frontmatter, add others later?

### Question 3: MetadataCache vs QueryService

We have:
- **AI-2.1**: MetadataCacheService (in-memory parsed metadata cache)
- **Existing**: QueryService (current implementation)
- **AI-1.4**: MetadataQueryPort (indexed queries)

**Questions**:
1. Does MetadataCacheService replace QueryService?
2. Or do they work together (MetadataCacheService feeds QueryService)?
3. Where does MetadataQueryPort fit in this architecture?

### Question 4: Story Insertion Points

Current Epic Impact Assessment has:
- Group 1 stories after 3.7 (Frontmatter Service)
- Group 2 stories after 3.6 (QueryService)

**Questions**:
1. Should MarkdownParserPort stories come BEFORE 3.7 (so FrontmatterService can use it)?
2. Should VaultFile DTO redesign come BEFORE 3.3 (VaultReaderPort uses it)?
3. What's the correct dependency-based insertion sequence?

---

## Summary: What I Need Verified

Before rewriting Epic Impact Assessment, I need answers to:

1. **DTO Architecture**: Which combination of strategies (AI-1.1, Line 927-929, AI-2.2, Entity Review Phase 3)?
2. **MarkdownParserPort**: Scope (frontmatter only or full NoteMetadata with Links/Headings/Tags)?
3. **Template System**: Should we create story for AI-2.3 refactoring or defer to Epic 5?
4. **MetadataCache**: How does AI-2.1 MetadataCacheService relate to QueryService and AI-1.4 MetadataQueryPort?
5. **Story Sequencing**: What's the correct insertion order based on dependencies?

Please review this mapping and let me know:
- What I've understood correctly
- What I've misunderstood
- Any critical details I'm still missing
- Answers to the questions above

Then I'll rewrite the Epic Impact Assessment properly incorporating all the detailed work.
