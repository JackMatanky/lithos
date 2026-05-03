# Parser Capability Matrix

Maps all parser features to behavior across policy combinations, validates contract compliance.

## Feature Matrix

| Feature | CmarkExtensionsPolicy State | EventRetentionPolicy State | BreakPolicy State | Stream IR (ParserEvent) Behavior | Structure Builder Behavior | Scanner/Semantic Expectation | Unsupported Fallback/Error Path |
|---------|-----------------------------|---------------------------|-------------------|----------------------------------|---------------------------|-----------------------------|--------------------------------|
| `math` (inline/display math) | Enabled/Disabled | Retain/Strip/Fail | N/A | Enabled: Emits `ParserEvent::InlineMath` / `ParserEvent::DisplayMath`; Disabled + Retain: Preserves raw syntax as text; Disabled + Strip: Drops all math events | Enabled: Adds `BlockNode::Math` with `inline`/`display` flag; Disabled: No math nodes created | Enabled: `extractor::scan_math` returns all math blocks with ranges; Disabled: Math content skipped during scanning | Disabled + Fail: Returns `ParserError::UnsupportedFeature("math")`; Disabled + Strip: Silent drop of events |
| `tables` | Enabled/Disabled | Retain/Strip/Fail | N/A | Enabled: Emits `ParserEvent::TableStart` / `TableEnd` / `TableRow` / `TableCell`; Disabled + Retain: Preserves raw table syntax; Disabled + Strip: Drops table events | Enabled: Builds `BlockNode::Table` with `TableRow` and `TableCell` children; Disabled: No table nodes | Enabled: `extractor::scan_tables` returns parsed table data with headers/rows; Disabled: Tables skipped | Disabled + Fail: `ParserError::UnsupportedFeature("tables")`; Disabled + Strip: Drops events |
| `footnotes` | Enabled/Disabled | Retain/Strip/Fail | N/A | Enabled: Emits `ParserEvent::FootnoteStart` / `FootnoteEnd` / `FootnoteRef`; Disabled + Retain: Preserves raw footnote syntax; Disabled + Strip: Drops events | Enabled: Adds `BlockNode::Footnote` with reference ID and content; Disabled: No footnote nodes | Enabled: `extractor::scan_footnotes` returns footnote definitions and references; Disabled: Skipped | Disabled + Fail: `ParserError::UnsupportedFeature("footnotes")`; Disabled + Strip: Drops events |
| `definition lists` | Enabled/Disabled | Retain/Strip/Fail | N/A | Enabled: Emits `ParserEvent::DefListStart` / `DefListEnd` / `DefTerm` / `DefDesc`; Disabled + Retain: Preserves raw syntax; Disabled + Strip: Drops events | Enabled: Builds `BlockNode::DefinitionList` with term/description children; Disabled: No nodes | Enabled: `extractor::scan_definition_lists` returns term-description pairs; Disabled: Skipped | Disabled + Fail: `ParserError::UnsupportedFeature("definition_lists")`; Disabled + Strip: Drops events |
| `metadata blocks` (YAML frontmatter) | Enabled/Disabled | Retain/Strip/Fail | N/A | Enabled: Emits `ParserEvent::MetadataBlock` with key-value pairs; Disabled + Retain: Preserves raw frontmatter; Disabled + Strip: Drops events | Enabled: Adds `BlockNode::Metadata` with parsed key-value pairs; Disabled: No metadata nodes | Enabled: `extractor::scan_metadata` returns frontmatter data; Disabled: Skipped | Disabled + Fail: `ParserError::UnsupportedFeature("metadata_blocks")`; Disabled + Strip: Drops events |
| `strikethrough` | Enabled/Disabled | Retain/Strip/Fail | N/A | Enabled: Emits `ParserEvent::Inline(InlineToken::Strikethrough)`; Disabled + Retain: Preserves `~~` syntax as text; Disabled + Strip: Drops events | Enabled: Applies `TextStyle::Strikethrough` to spanned text; Disabled: No strikethrough style | Enabled: Scanner includes strikethrough ranges in scannable output; Disabled: Skipped | Disabled + Fail: `ParserError::UnsupportedFeature("strikethrough")`; Disabled + Strip: Drops events |
| `task lists` | Enabled/Disabled | Retain/Strip/Fail | N/A | Enabled: Emits `ParserEvent::TaskListMarker` with checked state; Disabled + Retain: Preserves `[ ]`/`[x]` syntax; Disabled + Strip: Drops events | Enabled: Adds `BlockNode::TaskItem` with `checked` flag; Disabled: No task list nodes | Enabled: `extractor::scan_task_lists` returns task items with checked state; Disabled: Skipped | Disabled + Fail: `ParserError::UnsupportedFeature("task_lists")`; Disabled + Strip: Drops events |
| `wikilinks` | Enabled/Disabled | Retain/Strip/Fail | N/A | Enabled: Emits `ParserEvent::Inline(InlineToken::Wikilink)` with target; Disabled + Retain: Preserves `[[ ]]` syntax; Disabled + Strip: Drops events | Enabled: Applies `TextStyle::Wikilink` to spanned text; Disabled: No wikilink style | Enabled: Scanner includes wikilink targets in scannable output; Disabled: Skipped | Disabled + Fail: `ParserError::UnsupportedFeature("wikilinks")`; Disabled + Strip: Drops events |

## Policy Cross-Reference

- **CmarkExtensionsPolicy**: Controls parsing of extended Markdown features. States: `Enabled` (parse extensions), `Disabled` (ignore extensions).
- **EventRetentionPolicy**: Controls handling of disabled/unsupported features. States: `Retain` (preserve raw syntax as plain text), `Strip` (silently drop events), `Fail` (return `ParserError::UnsupportedFeature`).
- **BreakPolicy**: Controls line break emission. States: `Soft` (emits `ParserEvent::SoftBreak`), `Hard` (emits `ParserEvent::HardBreak`). Only affects inline text parsing, not block-level features.

## Edge Cases

1. **Nested features**: Math inside table cells, strikethrough wrapping wikilinks, task list items containing footnotes.
2. **Malformed syntax**: Unclosed math delimiters (`$...`), mismatched table cell counts, invalid YAML frontmatter.
3. **Policy conflicts**: `CmarkExtensionsPolicy::Disabled` + `EventRetentionPolicy::Fail` must return error before emitting any events.
4. **Empty content**: Empty math blocks, tables with no rows, metadata blocks with no key-value pairs.
5. **Mixed content**: Block and inline features interleaved (e.g., table containing math, paragraph with strikethrough and wikilinks).
