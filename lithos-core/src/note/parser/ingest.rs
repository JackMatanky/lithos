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
        block_refs::collect_block_refs,
        frontmatter::RawFrontmatter,
        headings::RawHeading,
        inline_fields::{RawInlineField, scan_inline_fields},
        links::{RawLink, RawLinkStyle, split_raw_target_and_anchor},
        list_items::{RawListDepth, RawListItem, RawListType, RawTaskKind},
        note::RawNote,
        reference_links::RawReferenceLink,
        sections::{RawSection, RawSectionKind, extract_sections},
        tags::{RawTag, scan_raw_tags},
        task_tokens::RawTaskTokens,
        tasks::RawTask,
    },
};

struct RawCollector<'source> {
    source: &'source str,
    list_stack: Vec<RawListType>,
    open_item_by_depth: Vec<SourceByteOffset>,
    headings: Vec<RawHeading>,
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
    fn collect_nodes(&mut self, nodes: &[Node]) -> Result<(), NoteError> {
        for node in nodes {
            match node.kind() {
                NodeKind::Heading {
                    level,
                    text,
                    links: inline_links,
                } => {
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
                    self.collect_nodes(items)?;
                    self.list_stack.pop();
                }
                NodeKind::ListItem {
                    text,
                    task_marker,
                    links: inline_links,
                    children,
                } => {
                    let list_item = ListItemContext {
                        position: node.range().start(),
                        text,
                        task_marker: *task_marker,
                        inline_links,
                        children,
                    };
                    self.collect_list_item(&list_item)?;
                }
                NodeKind::BlockQuote {
                    nodes: quote_nodes,
                    ..
                } => {
                    self.collect_nodes(quote_nodes)?;
                }
                NodeKind::CodeBlock {
                    ..
                } => {}
            }
        }
        Ok(())
    }

    fn collect_list_item(
        &mut self,
        list_item: &ListItemContext<'_>,
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
        let parent = parent_for_depth(depth_value, &self.open_item_by_depth);

        let list_type =
            self.list_stack.last().copied().unwrap_or(RawListType::Unordered);
        let depth = if depth_value == 0 {
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
                Some(raw_task_kind_from_marker(marker))
            }
            None => None,
        };
        let raw_text = list_item_text(list_item.text, list_item.children);
        if let Some(task_kind) = task_kind {
            self.list_items.push(RawListItem::new(
                list_type,
                depth,
                raw_text.clone(),
                Some(task_kind),
                position,
                parent,
            ));
            let raw_tags = scan_raw_tags(raw_text.as_ref(), position)?;
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
                list_type, depth, raw_text, None, position, parent,
            ));
        }

        self.collect_text_nodes(list_item.text)?;
        self.collect_inline_links(list_item.inline_links);
        self.collect_nodes(list_item.children)?;
        Ok(())
    }

    fn collect_text_nodes(&mut self, text: &Text) -> Result<(), NoteError> {
        for node in text.nodes() {
            if matches!(node.origin(), TextOrigin::LinkAlias) {
                continue;
            }
            let raw = scan_raw_tags(node.content(), node.range().start())?;
            self.tags.extend(raw);
        }
        scan_inline_fields(text, &mut self.inline_fields)?;
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
            let (target_raw, anchor) =
                split_raw_target_and_anchor(link.target());
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
        checkbox_marker_from_line(line)
    }
}

struct ListItemContext<'list_item> {
    position: SourceByteOffset,
    text: &'list_item Text,
    task_marker: Option<bool>,
    inline_links: &'list_item [InlineLink],
    children: &'list_item [Node],
}

/// Parse markdown and extract raw note artifacts.
///
/// # Errors
/// Returns [`NoteIngestError`] when parsing or extraction fails.
#[inline]
pub fn ingest_markdown(
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
    let mut sections = extract_sections(nodes)?;

    collector.collect_nodes(nodes)?;

    let frontmatter = frontmatter_block.map(|block| {
        let range = block.range();
        sections.push(RawSection::new(RawSectionKind::Frontmatter, range, 0));
        RawFrontmatter::new(block.kind(), block.text().into(), range)
    });
    sections.sort_by_key(|section| u32::from(section.range().start()));
    let block_refs = collect_block_refs(source)?;

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

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on &NodeKind"
)]
fn list_item_text(text: &Text, children: &[Node]) -> Box<str> {
    if !text.is_empty() {
        return text.to_boxed_str();
    }
    for child in children {
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

fn checkbox_marker_from_line(line: &str) -> Option<char> {
    let mut chars = line.chars().peekable();
    skip_whitespace(&mut chars);
    consume_list_marker(&mut chars)?;
    skip_whitespace(&mut chars);
    parse_checkbox_marker(&mut chars)
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while matches!(chars.peek(), Some(ch) if ch.is_whitespace()) {
        chars.next();
    }
}

fn consume_list_marker(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Option<()> {
    let first = chars.peek().copied()?;
    if matches!(first, '-' | '*' | '+') {
        chars.next();
        return Some(());
    }
    if !first.is_ascii_digit() {
        return None;
    }
    while matches!(chars.peek(), Some(ch) if ch.is_ascii_digit()) {
        chars.next();
    }
    match chars.peek().copied()? {
        '.' | ')' => {
            chars.next();
            Some(())
        }
        _ => None,
    }
}

fn parse_checkbox_marker(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Option<char> {
    if chars.next()? != '[' {
        return None;
    }
    let marker = chars.next()?;
    if chars.next()? != ']' {
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

/// Resolve the parent list item position for a given depth.
fn parent_for_depth(
    depth: u8,
    open_item_by_depth: &[SourceByteOffset],
) -> Option<SourceByteOffset> {
    if depth == 0 {
        return None;
    }
    open_item_by_depth.get(usize::from(depth).saturating_sub(1)).copied()
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
mod tests {
    use super::*;
    use crate::note::{paths::NotePath, raw::list_items::RawTaskKind};

    #[test]
    fn ingest_markdown_collects_task_tokens() -> Result<(), NoteIngestError> {
        let markdown = "- [ ] #task Review PR [priority:: 1]";
        let path = NotePath::try_new("notes/task.md")?;
        let raw = ingest_markdown(markdown, path, None, None)?;

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
    fn ingest_markdown_preserves_task_marker_case()
    -> Result<(), NoteIngestError> {
        let markdown = "- [X] #task Done";
        let path = NotePath::try_new("notes/task.md")?;
        let raw = ingest_markdown(markdown, path, None, None)?;

        let task = raw.tasks().first().expect("task should exist");
        assert!(matches!(task.task_kind(), RawTaskKind::Checked('X')));
        Ok(())
    }
}
