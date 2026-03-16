//! Raw extraction entry points for AST → `RawNote`.

use std::time::SystemTime;

use super::{
    block_refs::collect_block_refs,
    frontmatter::RawFrontmatter,
    headings::RawHeading,
    inline_fields::{RawInlineField, scan_inline_fields},
    links::{RawLink, RawLinkStyle},
    list_items::{RawListDepth, RawListItem, RawListType, RawTaskKind},
    note::RawNote,
    reference_links::RawReferenceLink,
    sections::{RawSection, RawSectionKind, extract_sections},
    tags::scan_raw_tags,
    task_tokens::RawTaskTokens,
    tasks::RawTask,
};
use crate::note::{
    error::NoteError,
    parser::{
        ast::{InlineLink, LinkStyle, ListStyle, Node, NodeKind, TextOrigin},
        frontmatter::MetadataBlock,
        note::ReferenceLinkDefinition,
    },
    position::SourceByteOffset,
};

/// Extract raw note artifacts from AST nodes and metadata.
///
/// # Errors
///
/// Returns [`NoteError`] when section extraction or token scanning fails.
#[expect(
    clippy::too_many_arguments,
    reason = "Raw extraction requires full note context"
)]
#[inline]
pub fn extract_raw_note(
    nodes: &[Node],
    frontmatter_block: Option<MetadataBlock>,
    reference_links: Vec<ReferenceLinkDefinition>,
    source: &str,
    path: crate::note::paths::NotePath,
    source_hash: Box<str>,
    source_bytes: u64,
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
) -> Result<RawNote, NoteError> {
    let mut headings = Vec::new();
    let mut links = Vec::new();
    let mut tags = Vec::new();
    let mut list_items = Vec::new();
    let mut tasks = Vec::new();
    let mut inline_fields = Vec::new();
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
    let mut list_stack: Vec<RawListType> = Vec::new();

    let mut sections = extract_sections(nodes)?;

    let mut open_item_by_depth: Vec<SourceByteOffset> = Vec::new();

    walk_nodes(
        source,
        nodes,
        &mut list_stack,
        &mut open_item_by_depth,
        &mut headings,
        &mut links,
        &mut tags,
        &mut list_items,
        &mut tasks,
        &mut inline_fields,
    )?;

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
        headings,
        sections,
        links,
        tags,
        list_items,
        tasks,
        inline_fields,
        reference_links,
        block_refs,
    ))
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on &NodeKind"
)]
#[expect(
    clippy::too_many_arguments,
    reason = "Traversal requires multiple accumulators"
)]
#[expect(
    clippy::too_many_lines,
    reason = "Traversal handles all node variants"
)]
fn walk_nodes(
    source: &str,
    nodes: &[Node],
    list_stack: &mut Vec<RawListType>,
    open_item_by_depth: &mut Vec<SourceByteOffset>,
    headings: &mut Vec<RawHeading>,
    links: &mut Vec<RawLink>,
    tags: &mut Vec<super::tags::RawTag>,
    list_items: &mut Vec<RawListItem>,
    tasks: &mut Vec<RawTask>,
    inline_fields: &mut Vec<RawInlineField>,
) -> Result<(), NoteError> {
    for node in nodes {
        match node.kind() {
            NodeKind::Heading {
                level,
                text,
                links: inline_links,
            } => {
                let raw = RawHeading::new(
                    *level,
                    text.to_boxed_str(),
                    node.range(),
                    node.range().start(),
                );
                headings.push(raw);
                scan_text_nodes(text, tags)?;
                scan_inline_fields(text, inline_fields)?;
                collect_inline_links(inline_links, links);
            }
            NodeKind::Paragraph {
                text,
                links: inline_links,
            } => {
                scan_text_nodes(text, tags)?;
                scan_inline_fields(text, inline_fields)?;
                collect_inline_links(inline_links, links);
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
                list_stack.push(list_type);
                walk_nodes(
                    source,
                    items,
                    list_stack,
                    open_item_by_depth,
                    headings,
                    links,
                    tags,
                    list_items,
                    tasks,
                    inline_fields,
                )?;
                list_stack.pop();
            }
            NodeKind::ListItem {
                text,
                task_marker,
                links: inline_links,
                children,
            } => {
                let position = node.range().start();
                let depth_value =
                    u8::try_from(list_stack.len()).unwrap_or(u8::MAX);
                let depth_index = usize::from(depth_value);
                if open_item_by_depth.len() <= depth_index {
                    open_item_by_depth
                        .resize(depth_index.saturating_add(1), position);
                }
                if let Some(slot) = open_item_by_depth.get_mut(depth_index) {
                    *slot = position;
                }
                open_item_by_depth.truncate(depth_index.saturating_add(1));
                let parent = parent_for_depth(depth_value, open_item_by_depth);

                let list_type = list_stack
                    .last()
                    .copied()
                    .unwrap_or(RawListType::Unordered);
                let depth = if depth_value == 0 {
                    RawListDepth::Root
                } else {
                    RawListDepth::Nested(depth_value)
                };

                let task_kind = match task_marker {
                    Some(checked) => {
                        let fallback = if *checked {
                            'x'
                        } else {
                            ' '
                        };
                        let marker = task_marker_from_source(source, position)
                            .unwrap_or(fallback);
                        Some(raw_task_kind_from_marker(marker))
                    }
                    None => None,
                };
                let raw_text = list_item_text(text, children);
                if let Some(task_kind) = task_kind {
                    list_items.push(RawListItem::new(
                        list_type,
                        depth,
                        raw_text.clone(),
                        Some(task_kind),
                        position,
                        parent,
                    ));
                    let raw_tags = scan_raw_tags(raw_text.as_ref(), position)?;
                    let tags_for_task = raw_tags
                        .into_iter()
                        .map(|tag| tag.value().into())
                        .collect();
                    let tokens = RawTaskTokens::parse(raw_text.as_ref(), &[]);
                    tasks.push(RawTask::new(
                        task_kind,
                        raw_text,
                        tags_for_task,
                        tokens.inline_fields().to_vec(),
                        tokens.emoji_dates().to_vec(),
                        position,
                    ));
                } else {
                    list_items.push(RawListItem::new(
                        list_type, depth, raw_text, None, position, parent,
                    ));
                }

                scan_text_nodes(text, tags)?;
                scan_inline_fields(text, inline_fields)?;
                collect_inline_links(inline_links, links);
                walk_nodes(
                    source,
                    children,
                    list_stack,
                    open_item_by_depth,
                    headings,
                    links,
                    tags,
                    list_items,
                    tasks,
                    inline_fields,
                )?;
            }
            NodeKind::BlockQuote {
                nodes: quote_nodes,
                ..
            } => {
                walk_nodes(
                    source,
                    quote_nodes,
                    list_stack,
                    open_item_by_depth,
                    headings,
                    links,
                    tags,
                    list_items,
                    tasks,
                    inline_fields,
                )?;
            }
            NodeKind::CodeBlock {
                ..
            } => {}
        }
    }
    Ok(())
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on &NodeKind"
)]
fn list_item_text(
    text: &crate::note::parser::ast::Text,
    children: &[Node],
) -> Box<str> {
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

fn task_marker_from_source(
    source: &str,
    position: SourceByteOffset,
) -> Option<char> {
    let start = usize::try_from(u32::from(position)).ok()?;
    let tail = source.get(start..)?;
    let line = tail.split(['\n', '\r']).next().unwrap_or(tail);
    checkbox_marker_from_line(line)
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

/// Scan text nodes for tag tokens, excluding link alias text.
fn scan_text_nodes(
    text: &crate::note::parser::ast::Text,
    tags: &mut Vec<super::tags::RawTag>,
) -> Result<(), NoteError> {
    for node in text.nodes() {
        if matches!(node.origin(), TextOrigin::LinkAlias) {
            continue;
        }
        let raw = scan_raw_tags(node.content(), node.range().start())?;
        tags.extend(raw);
    }
    Ok(())
}

fn collect_inline_links(inline_links: &[InlineLink], links: &mut Vec<RawLink>) {
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
            super::links::split_raw_target_and_anchor(link.target());
        links.push(RawLink::new(
            raw_style,
            link.is_embed(),
            target_raw.into(),
            alias,
            anchor.map(Into::into),
            link.range().start(),
        ));
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
mod tests {
    use super::*;
    use crate::note::{parser, paths::NotePath, raw::list_items::RawTaskKind};

    #[test]
    fn extract_raw_note_collects_task_tokens() -> Result<(), NoteError> {
        let markdown = "- [ ] #task Review PR [priority:: 1]";
        let parsed: parser::note::ParsedNote =
            parser::parse_markdown(markdown, parser::obsidian_options())
                .map_err(NoteError::from)?;
        let path = NotePath::try_new("notes/task.md")?;
        let raw = extract_raw_note(
            parsed.nodes(),
            parsed.frontmatter().cloned(),
            parsed.reference_links().to_vec(),
            markdown,
            path,
            "hash".into(),
            u64::try_from(markdown.len()).map_err(|_error| {
                NoteError::Structure("source length out of range")
            })?,
            None,
            None,
        )?;

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
    fn extract_raw_note_preserves_task_marker_case() -> Result<(), NoteError> {
        let markdown = "- [X] #task Done";
        let parsed: parser::note::ParsedNote =
            parser::parse_markdown(markdown, parser::obsidian_options())
                .map_err(NoteError::from)?;
        let path = NotePath::try_new("notes/task.md")?;
        let raw = extract_raw_note(
            parsed.nodes(),
            parsed.frontmatter().cloned(),
            parsed.reference_links().to_vec(),
            markdown,
            path,
            "hash".into(),
            u64::try_from(markdown.len()).map_err(|_error| {
                NoteError::Structure("source length out of range")
            })?,
            None,
            None,
        )?;

        let task = raw.tasks().first().expect("task should exist");
        assert!(matches!(task.task_kind(), RawTaskKind::Checked('X')));
        Ok(())
    }
}
