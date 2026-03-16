//! Raw extraction entry points for AST → `RawNote`.

use std::time::SystemTime;

use super::{
    block_refs::collect_block_refs,
    frontmatter::RawFrontmatter,
    headings::RawHeading,
    inline_fields::{RawInlineField, scan_inline_fields},
    links::{RawLink, RawLinkStyle},
    list_items::{RawListDepth, RawListItem, RawListType},
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
        ast::{
            AstInlineLink, AstLinkStyle, AstListType, AstNode, AstNodeKind,
            TextOrigin,
        },
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
    nodes: &[AstNode],
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
    reason = "Match ergonomics on &AstNodeKind"
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
    nodes: &[AstNode],
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
            AstNodeKind::Heading {
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
            AstNodeKind::Paragraph {
                text,
                links: inline_links,
            } => {
                scan_text_nodes(text, tags)?;
                scan_inline_fields(text, inline_fields)?;
                collect_inline_links(inline_links, links);
            }
            AstNodeKind::List {
                list_type,
                items,
            } => {
                let list_type = match list_type {
                    AstListType::Ordered {
                        start,
                    } => RawListType::Ordered {
                        start: *start,
                    },
                    AstListType::Unordered => RawListType::Unordered,
                };
                list_stack.push(list_type);
                walk_nodes(
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
            AstNodeKind::ListItem {
                text,
                task,
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

                let (is_checkbox, status_symbol) = match task {
                    Some(checked) => (
                        true,
                        Some(if *checked {
                            'x'
                        } else {
                            ' '
                        }),
                    ),
                    None => (false, None),
                };
                let raw_text = list_item_text(text, children);
                if is_checkbox {
                    list_items.push(RawListItem::new(
                        list_type,
                        depth,
                        raw_text.clone(),
                        is_checkbox,
                        status_symbol,
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
                        status_symbol,
                        raw_text,
                        tags_for_task,
                        tokens.inline_fields().to_vec(),
                        tokens.emoji_dates().to_vec(),
                        position,
                    ));
                } else {
                    list_items.push(RawListItem::new(
                        list_type,
                        depth,
                        raw_text,
                        is_checkbox,
                        status_symbol,
                        position,
                        parent,
                    ));
                }

                scan_text_nodes(text, tags)?;
                scan_inline_fields(text, inline_fields)?;
                collect_inline_links(inline_links, links);
                walk_nodes(
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
            AstNodeKind::BlockQuote {
                nodes: quote_nodes,
                ..
            } => {
                walk_nodes(
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
            AstNodeKind::CodeBlock {
                ..
            } => {}
        }
    }
    Ok(())
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on &AstNodeKind"
)]
fn list_item_text(
    text: &crate::note::parser::ast::Text,
    children: &[AstNode],
) -> Box<str> {
    if !text.is_empty() {
        return text.to_boxed_str();
    }
    for child in children {
        if let AstNodeKind::Paragraph {
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

fn collect_inline_links(
    inline_links: &[AstInlineLink],
    links: &mut Vec<RawLink>,
) {
    for link in inline_links {
        let raw_style = match link.style() {
            AstLinkStyle::Wiki => RawLinkStyle::Wiki,
            AstLinkStyle::Markdown => RawLinkStyle::Markdown,
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
    use crate::note::{parser, paths::NotePath};

    #[test]
    fn extract_raw_note_collects_task_tokens() -> Result<(), NoteError> {
        let markdown = "- [ ] #task Review PR [priority:: 1]";
        let parsed =
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
        assert_eq!(task.status_symbol(), Some(' '));
        assert!(task.tags().iter().any(|tag| tag.as_ref() == "#task"));
        assert!(task.inline_fields().iter().any(|pair| pair.0.as_ref()
            == "priority"
            && pair.1.as_ref() == "1"));
        Ok(())
    }
}
