//! Markdown ingestion entry points (File → `RawNote`).

use std::time::SystemTime;

use super::{obsidian_options, parse_markdown};
use crate::note::{
    error::{NoteError, NoteIngestError},
    parser::{
        ast::{
            InlineLink, LinkStyle, ListStyle, Node, NodeKind, Text, TextOrigin,
        },
        frontmatter::MetadataBlock,
        note::ReferenceLinkDefinition,
    },
    paths::NotePath,
    position::SourceByteOffset,
    raw::{
        RawBlockRef, RawFrontmatter, RawHeading, RawInlineField, RawLink,
        RawLinkStyle, RawListDepth, RawListItem, RawListType, RawNote,
        RawReferenceLink, RawSection, RawSectionKind, RawTag, RawTask,
        RawTaskKind,
    },
    task_tokens::RawTaskTokens,
};

struct RawCollector<'source> {
    source: &'source str,
    list_stack: Vec<RawListType>,
    open_item_by_depth: Vec<SourceByteOffset>,
    headings: Vec<RawHeading>,
    sections: Vec<RawSection>,
    links: Vec<RawLink>,
    tags: Vec<RawTag>,
    list_items: Vec<RawListItem>,
    tasks: Vec<RawTask>,
    inline_fields: Vec<RawInlineField>,
}

impl<'source> RawCollector<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
            list_stack: Vec::new(),
            open_item_by_depth: Vec::new(),
            headings: Vec::new(),
            sections: Vec::new(),
            links: Vec::new(),
            tags: Vec::new(),
            list_items: Vec::new(),
            tasks: Vec::new(),
            inline_fields: Vec::new(),
        }
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &NodeKind"
    )]
    fn collect_nodes(
        &mut self,
        nodes: &[Node],
        depth: u32,
    ) -> Result<(), NoteError> {
        for node in nodes {
            match node.kind() {
                NodeKind::Heading {
                    level,
                    text,
                    links: inline_links,
                } => {
                    self.sections.push(RawSection::new(
                        RawSectionKind::Heading,
                        node.range(),
                        depth,
                    ));
                    let raw = RawHeading::new(
                        level.value(),
                        text.to_boxed_str(),
                        node.range(),
                        node.range().start(),
                    );
                    self.headings.push(raw);
                    self.collect_text_nodes(text)?;
                    self.collect_inline_links(inline_links);
                }
                NodeKind::Paragraph {
                    text,
                    links: inline_links,
                } => {
                    self.sections.push(RawSection::new(
                        RawSectionKind::Paragraph,
                        node.range(),
                        depth,
                    ));
                    self.collect_text_nodes(text)?;
                    self.collect_inline_links(inline_links);
                }
                NodeKind::List {
                    list_type,
                    items,
                } => {
                    let list_type = match list_type {
                        ListStyle::Ordered {
                            start,
                        } => RawListType::Ordered {
                            start: *start,
                        },
                        ListStyle::Unordered => RawListType::Unordered,
                    };
                    self.list_stack.push(list_type);
                    self.collect_nodes(items, depth.saturating_add(1))?;
                    self.list_stack.pop();
                }
                NodeKind::ListItem {
                    text,
                    task_marker,
                    links: inline_links,
                    children,
                } => {
                    self.sections.push(RawSection::new(
                        RawSectionKind::List,
                        node.range(),
                        depth,
                    ));
                    let list_item = ListItemContext {
                        position: node.range().start(),
                        text,
                        task_marker: *task_marker,
                        inline_links,
                        children,
                    };
                    self.collect_list_item(&list_item, depth)?;
                }
                NodeKind::BlockQuote {
                    nodes: quote_nodes,
                    ..
                } => {
                    self.sections.push(RawSection::new(
                        RawSectionKind::BlockQuote,
                        node.range(),
                        depth,
                    ));
                    self.collect_nodes(quote_nodes, depth.saturating_add(1))?;
                }
                NodeKind::CodeBlock {
                    ..
                } => {
                    self.sections.push(RawSection::new(
                        RawSectionKind::CodeBlock,
                        node.range(),
                        depth,
                    ));
                }
            }
        }
        Ok(())
    }

    fn collect_list_item(
        &mut self,
        list_item: &ListItemContext<'_>,
        depth: u32,
    ) -> Result<(), NoteError> {
        let position = list_item.position;
        let depth_value =
            u8::try_from(self.list_stack.len()).unwrap_or(u8::MAX);
        let depth_index = usize::from(depth_value);
        if self.open_item_by_depth.len() <= depth_index {
            self.open_item_by_depth
                .resize(depth_index.saturating_add(1), position);
        }
        if let Some(slot) = self.open_item_by_depth.get_mut(depth_index) {
            *slot = position;
        }
        self.open_item_by_depth.truncate(depth_index.saturating_add(1));
        let parent = self.parent_for_depth(depth_value);

        let list_type =
            self.list_stack.last().copied().unwrap_or(RawListType::Unordered);
        let list_depth = if depth_value == 0 {
            RawListDepth::Root
        } else {
            RawListDepth::Nested(depth_value)
        };

        let task_kind = match list_item.task_marker {
            Some(checked) => {
                let fallback = if checked {
                    'x'
                } else {
                    ' '
                };
                let marker =
                    self.task_marker_from_source(position).unwrap_or(fallback);
                Some(TaskMarkerScanner::raw_task_kind_from_marker(marker))
            }
            None => None,
        };
        let raw_text = list_item.text_content();
        if let Some(task_kind) = task_kind {
            self.list_items.push(RawListItem::new(
                list_type,
                list_depth,
                raw_text.clone(),
                Some(task_kind),
                position,
                parent,
            ));
            let raw_tags = TagScanner::scan(raw_text.as_ref(), position)?;
            let tags_for_task =
                raw_tags.into_iter().map(|tag| tag.value().into()).collect();
            let tokens = RawTaskTokens::parse(raw_text.as_ref(), &[]);
            self.tasks.push(RawTask::new(
                task_kind,
                raw_text,
                tags_for_task,
                tokens.inline_fields().to_vec(),
                tokens.emoji_dates().to_vec(),
                position,
            ));
        } else {
            self.list_items.push(RawListItem::new(
                list_type, list_depth, raw_text, None, position, parent,
            ));
        }

        self.collect_text_nodes(list_item.text)?;
        self.collect_inline_links(list_item.inline_links);
        self.collect_nodes(list_item.children, depth.saturating_add(1))?;
        Ok(())
    }

    fn collect_text_nodes(&mut self, text: &Text) -> Result<(), NoteError> {
        for node in text.nodes() {
            if matches!(node.origin(), TextOrigin::LinkAlias) {
                continue;
            }
            let raw = TagScanner::scan(node.content(), node.range().start())?;
            self.tags.extend(raw);
        }
        InlineFieldScanner::scan(text, &mut self.inline_fields)?;
        Ok(())
    }

    fn collect_inline_links(&mut self, inline_links: &[InlineLink]) {
        for link in inline_links {
            let raw_style = match link.style() {
                LinkStyle::Wiki => RawLinkStyle::Wiki,
                LinkStyle::Markdown => RawLinkStyle::Markdown,
            };
            let alias = if link.alias().is_empty() {
                None
            } else {
                let alias_text = link.alias().to_boxed_str();
                if alias_text.trim().is_empty() {
                    None
                } else {
                    Some(alias_text)
                }
            };
            let (target_raw, anchor) = LinkTarget::new(link.target()).split();
            self.links.push(RawLink::new(
                raw_style,
                link.is_embed(),
                target_raw.into(),
                alias,
                anchor.map(Into::into),
                link.range().start(),
            ));
        }
    }

    fn task_marker_from_source(
        &self,
        position: SourceByteOffset,
    ) -> Option<char> {
        let start = usize::try_from(u32::from(position)).ok()?;
        let tail = self.source.get(start..)?;
        let line = tail.split(['\n', '\r']).next().unwrap_or(tail);
        TaskMarkerScanner::new(line).scan()
    }

    /// Resolve the parent list item position for a given depth.
    fn parent_for_depth(&self, depth: u8) -> Option<SourceByteOffset> {
        if depth == 0 {
            return None;
        }
        self.open_item_by_depth
            .get(usize::from(depth).saturating_sub(1))
            .copied()
    }
}

struct ListItemContext<'list_item> {
    position: SourceByteOffset,
    text: &'list_item Text,
    task_marker: Option<bool>,
    inline_links: &'list_item [InlineLink],
    children: &'list_item [Node],
}

impl ListItemContext<'_> {
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &NodeKind"
    )]
    fn text_content(&self) -> Box<str> {
        if !self.text.is_empty() {
            return self.text.to_boxed_str();
        }
        for child in self.children {
            if let NodeKind::Paragraph {
                text: child_text,
                ..
            } = child.kind()
                && !child_text.is_empty()
            {
                return child_text.to_boxed_str();
            }
        }
        "".into()
    }
}

/// Parse markdown and extract raw note artifacts.
///
/// # Errors
/// Returns [`NoteIngestError`] when parsing or extraction fails.
#[inline]
pub fn extract_markdown(
    markdown: &str,
    path: NotePath,
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
) -> Result<RawNote, NoteIngestError> {
    let parsed = parse_markdown(markdown, obsidian_options())?;
    extract_raw_note(
        parsed.nodes(),
        parsed.frontmatter().cloned(),
        parsed.reference_links().to_vec(),
        markdown,
        path,
        parsed.source_hash_boxed(),
        parsed.source_bytes(),
        created_at,
        modified_at,
    )
    .map_err(NoteIngestError::Domain)
}

/// Extract raw note artifacts from AST nodes and metadata.
///
/// # Errors
/// Returns [`NoteError`] when section extraction or token scanning fails.
#[expect(
    clippy::too_many_arguments,
    reason = "Raw extraction requires full note context"
)]
#[inline]
fn extract_raw_note(
    nodes: &[Node],
    frontmatter_block: Option<MetadataBlock>,
    reference_links: Vec<ReferenceLinkDefinition>,
    source: &str,
    path: NotePath,
    source_hash: Box<str>,
    source_bytes: u64,
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
) -> Result<RawNote, NoteError> {
    let mut collector = RawCollector::new(source);
    let reference_links = reference_links
        .into_iter()
        .map(|definition| {
            RawReferenceLink::new(
                definition.id().into(),
                definition.target().into(),
                definition.position(),
            )
        })
        .collect::<Vec<_>>();

    // Single-pass node collection gathers facts and sections
    collector.collect_nodes(nodes, 0)?;

    let mut sections = collector.sections;
    let frontmatter = frontmatter_block.map(|block| {
        let range = block.range();
        sections.push(RawSection::new(RawSectionKind::Frontmatter, range, 0));
        RawFrontmatter::new(
            block.kind().to_frontmatter_format(),
            block.text().into(),
            range,
        )
    });
    sections.sort_by_key(|section| u32::from(section.range().start()));

    let block_refs = BlockRefScanner::new(source).collect()?;

    Ok(RawNote::new(
        path,
        source_hash,
        source_bytes,
        created_at,
        modified_at,
        frontmatter,
        collector.headings,
        sections,
        collector.links,
        collector.tags,
        collector.list_items,
        collector.tasks,
        collector.inline_fields,
        reference_links,
        block_refs,
    ))
}

struct TaskMarkerScanner<'source> {
    chars: std::iter::Peekable<std::str::Chars<'source>>,
}

impl<'source> TaskMarkerScanner<'source> {
    fn new(line: &'source str) -> Self {
        Self {
            chars: line.chars().peekable(),
        }
    }

    fn scan(&mut self) -> Option<char> {
        self.skip_whitespace();
        self.consume_list_marker()?;
        self.skip_whitespace();
        self.parse_checkbox_marker()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.chars.peek(), Some(ch) if ch.is_whitespace()) {
            self.chars.next();
        }
    }

    fn consume_list_marker(&mut self) -> Option<()> {
        let first = self.chars.peek().copied()?;
        if matches!(first, '-' | '*' | '+') {
            self.chars.next();
            return Some(());
        }
        if !first.is_ascii_digit() {
            return None;
        }
        while matches!(self.chars.peek(), Some(ch) if ch.is_ascii_digit()) {
            self.chars.next();
        }
        match self.chars.peek().copied()? {
            '.' | ')' => {
                self.chars.next();
                Some(())
            }
            _ => None,
        }
    }

    fn parse_checkbox_marker(&mut self) -> Option<char> {
        if self.chars.next()? != '[' {
            return None;
        }
        let marker = self.chars.next()?;
        if self.chars.next()? != ']' {
            return None;
        }
        Some(marker)
    }

    fn raw_task_kind_from_marker(marker: char) -> RawTaskKind {
        match marker {
            ' ' => RawTaskKind::Unchecked(marker),
            'x' | 'X' => RawTaskKind::Checked(marker),
            _ => RawTaskKind::Other(marker),
        }
    }
}

struct InlineFieldScanner;

impl InlineFieldScanner {
    fn scan(
        text: &Text,
        fields: &mut Vec<RawInlineField>,
    ) -> Result<(), NoteError> {
        let has_potential =
            text.nodes().iter().any(|n| n.content().contains("::"));
        if !has_potential {
            return Ok(());
        }

        let mut combined = String::new();
        let mut segments = Vec::new();

        for node in text.nodes() {
            if node.origin() != crate::note::parser::ast::TextOrigin::Normal {
                continue;
            }
            let start = combined.len();
            combined.push_str(node.content());
            segments.push((start, node.range().start()));
        }

        if combined.is_empty() {
            return Ok(());
        }

        Self::scan_text(&combined, &segments, fields)
    }

    fn scan_text(
        text: &str,
        segments: &[(usize, SourceByteOffset)],
        fields: &mut Vec<RawInlineField>,
    ) -> Result<(), NoteError> {
        let mut bracket_spans = Vec::new();
        Self::scan_delim(
            text,
            b'[',
            b']',
            segments,
            fields,
            &mut bracket_spans,
        )?;
        Self::scan_delim(
            text,
            b'(',
            b')',
            segments,
            fields,
            &mut bracket_spans,
        )?;
        Self::scan_bare(text, segments, fields, &bracket_spans)?;
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Inline field parsing needs delimiters and position mapping"
    )]
    fn scan_delim(
        text: &str,
        open_delim: u8,
        close_delim: u8,
        segments: &[(usize, SourceByteOffset)],
        fields: &mut Vec<RawInlineField>,
        spans: &mut Vec<(usize, usize)>,
    ) -> Result<(), NoteError> {
        let bytes = text.as_bytes();
        let mut cursor = 0;
        while let Some(open_rel) = bytes
            .get(cursor..)
            .and_then(|slice| slice.iter().position(|&b| b == open_delim))
        {
            let open = cursor.saturating_add(open_rel);
            let after_open = open.saturating_add(1);
            let Some(close_rel) = bytes
                .get(after_open..)
                .and_then(|slice| slice.iter().position(|&b| b == close_delim))
            else {
                break;
            };
            let close = after_open.saturating_add(close_rel);
            spans.push((open, close.saturating_add(1)));
            let Some(inner) = text.get(after_open..close) else {
                cursor = close.saturating_add(1);
                continue;
            };
            if let Some((key, value)) = inner.split_once("::") {
                let key_trimmed = key.trim();
                let value_trimmed = value.trim();
                if !key_trimmed.is_empty() && !value_trimmed.is_empty() {
                    let key_start = key
                        .find(key_trimmed)
                        .unwrap_or(0)
                        .saturating_add(after_open);
                    let position =
                        Self::position_for_offset(segments, key_start)?;
                    fields.push(RawInlineField::new(
                        Self::normalize_key(key_trimmed),
                        value_trimmed.into(),
                        position,
                    ));
                }
            }
            cursor = close.saturating_add(1);
        }
        Ok(())
    }

    fn scan_bare(
        text: &str,
        segments: &[(usize, SourceByteOffset)],
        fields: &mut Vec<RawInlineField>,
        bracket_spans: &[(usize, usize)],
    ) -> Result<(), NoteError> {
        let mut offset = 0usize;
        for line in text.split_inclusive(['\n', '\r']) {
            let trimmed = line.trim_end_matches(['\n', '\r']);
            let Some((key, value)) = trimmed.split_once("::") else {
                offset = offset.saturating_add(line.len());
                continue;
            };
            let key_trimmed = key.trim();
            let value_trimmed = value.trim();
            if key_trimmed.is_empty() || value_trimmed.is_empty() {
                offset = offset.saturating_add(line.len());
                continue;
            }
            let key_start =
                trimmed.find(key_trimmed).unwrap_or(0).saturating_add(offset);
            if bracket_spans
                .iter()
                .any(|&(start, end)| key_start >= start && key_start < end)
            {
                offset = offset.saturating_add(line.len());
                continue;
            }
            let position = Self::position_for_offset(segments, key_start)?;
            fields.push(RawInlineField::new(
                Self::normalize_key(key_trimmed),
                value_trimmed.into(),
                position,
            ));
            offset = offset.saturating_add(line.len());
        }
        Ok(())
    }

    fn position_for_offset(
        segments: &[(usize, SourceByteOffset)],
        offset: usize,
    ) -> Result<SourceByteOffset, NoteError> {
        let mut current = None;
        for &(start, position) in segments.iter().rev() {
            if start <= offset {
                current = Some((start, position));
                break;
            }
        }
        let (segment_start, segment_pos) = current
            .ok_or(NoteError::Structure("inline field offset out of range"))?;
        let delta = offset.saturating_sub(segment_start);
        let base =
            usize::try_from(u32::from(segment_pos)).map_err(|_error| {
                NoteError::Structure("inline field offset out of range")
            })?;
        SourceByteOffset::try_from_usize(base.saturating_add(delta))
    }

    fn normalize_key(key: &str) -> Box<str> {
        let stripped = key
            .chars()
            .filter(|ch| !matches!(ch, '*' | '_' | '~' | '`'))
            .collect::<String>();
        stripped
            .split_whitespace()
            .map(str::to_ascii_lowercase)
            .collect::<Vec<_>>()
            .join("-")
            .into_boxed_str()
    }
}

struct TagScanner;

impl TagScanner {
    fn scan(
        text: &str,
        base_offset: SourceByteOffset,
    ) -> Result<Vec<RawTag>, NoteError> {
        let mut tags = Vec::new();
        let mut chars = text.char_indices().peekable();
        let mut prev_is_alnum = false;
        let base =
            usize::try_from(u32::from(base_offset)).map_err(|_error| {
                NoteError::Structure("tag offset out of range")
            })?;

        while let Some((start_idx, ch)) = chars.next() {
            if ch != '#' || prev_is_alnum {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            }

            let Some(mut end_idx) = start_idx.checked_add(ch.len_utf8()) else {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            };
            while let Some(&(next_idx, next_ch)) = chars.peek() {
                if !(next_ch.is_alphanumeric()
                    || matches!(next_ch, '_' | '-' | '/'))
                {
                    break;
                }
                chars.next();
                let Some(updated) = next_idx.checked_add(next_ch.len_utf8())
                else {
                    break;
                };
                end_idx = updated;
            }

            let Some(raw) = text.get(start_idx..end_idx) else {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            };

            if raw.len() > 1 {
                let offset = base.saturating_add(start_idx);
                let position = SourceByteOffset::try_from_usize(offset)?;
                tags.push(RawTag::new(raw.into(), position));
            }

            prev_is_alnum =
                raw.chars().last().is_some_and(char::is_alphanumeric);
        }

        Ok(tags)
    }
}

struct BlockRefScanner<'source> {
    source: &'source str,
}

impl<'source> BlockRefScanner<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
        }
    }

    fn collect(&self) -> Result<Vec<RawBlockRef>, NoteError> {
        let mut refs = Vec::new();
        let mut offset = 0usize;
        let mut in_code_block = false;
        let mut in_frontmatter = false;
        let mut frontmatter_fence: Option<&'static str> = None;

        for line in self.source.split_inclusive('\n') {
            let mut trimmed_line = line.trim_end_matches(['\n', '\r']);

            if offset == 0 {
                if trimmed_line == "---" {
                    in_frontmatter = true;
                    frontmatter_fence = Some("---");
                    offset = offset.saturating_add(line.len());
                    continue;
                }
                if trimmed_line == "+++" {
                    in_frontmatter = true;
                    frontmatter_fence = Some("+++");
                    offset = offset.saturating_add(line.len());
                    continue;
                }
            }

            if in_frontmatter {
                if frontmatter_fence.is_some_and(|fence| fence == trimmed_line)
                {
                    in_frontmatter = false;
                }
                offset = offset.saturating_add(line.len());
                continue;
            }

            let trimmed_start = trimmed_line.trim_start();
            if trimmed_start.starts_with("```")
                || trimmed_start.starts_with("~~~")
            {
                in_code_block = !in_code_block;
                offset = offset.saturating_add(line.len());
                continue;
            }

            if in_code_block {
                offset = offset.saturating_add(line.len());
                continue;
            }

            trimmed_line = trimmed_line.trim_end();
            if let Some(caret_idx) = trimmed_line.rfind('^') {
                let before = trimmed_line.get(..caret_idx).unwrap_or("");
                let after = trimmed_line
                    .get(caret_idx.saturating_add(1)..)
                    .unwrap_or("");
                let id = after.trim();
                let valid = !id.is_empty()
                    && id.chars().all(|ch| {
                        ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
                    });
                if valid
                    && (before.is_empty()
                        || before
                            .chars()
                            .last()
                            .is_some_and(char::is_whitespace))
                {
                    let position = SourceByteOffset::try_from_usize(
                        offset.saturating_add(caret_idx),
                    )?;
                    refs.push(RawBlockRef::new(id.into(), position));
                }
            }
            offset = offset.saturating_add(line.len());
        }

        Ok(refs)
    }
}

struct LinkTarget<'source>(&'source str);

impl<'source> LinkTarget<'source> {
    fn new(target: &'source str) -> Self {
        Self(target)
    }

    fn split(self) -> (&'source str, Option<&'source str>) {
        if self.is_external() {
            return (self.0, None);
        }
        let Some((path, anchor_text)) = self.0.split_once('#') else {
            return (self.0, None);
        };
        (path, Some(anchor_text))
    }

    fn is_external(&self) -> bool {
        self.0.starts_with("http://")
            || self.0.starts_with("https://")
            || self.0.starts_with("ftp://")
            || self.0.starts_with("mailto:")
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
mod tests {
    use super::*;
    use crate::note::{paths::NotePath, raw::RawTaskKind};

    #[test]
    fn extract_markdown_collects_task_tokens() -> Result<(), NoteIngestError> {
        let markdown = "- [ ] #task Review PR [priority:: 1]";
        let path = NotePath::try_new("notes/task.md")?;
        let raw = extract_markdown(markdown, path, None, None)?;

        assert_eq!(raw.tasks().len(), 1);
        let task = raw.tasks().first().expect("task should exist");
        assert_eq!(task.task_kind().marker(), ' ');
        assert!(task.tags().iter().any(|tag| tag.as_ref() == "#task"));
        assert!(task.inline_fields().iter().any(|pair| pair.0.as_ref()
            == "priority"
            && pair.1.as_ref() == "1"));
        Ok(())
    }

    #[test]
    fn extract_markdown_preserves_task_marker_case()
    -> Result<(), NoteIngestError> {
        let markdown = "- [X] #task Done";
        let path = NotePath::try_new("notes/task.md")?;
        let raw = extract_markdown(markdown, path, None, None)?;

        let task = raw.tasks().first().expect("task should exist");
        assert!(matches!(task.task_kind(), RawTaskKind::Checked('X')));
        Ok(())
    }
}
