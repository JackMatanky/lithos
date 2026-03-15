//! Raw extraction entry points for AST → RawNote.

use std::time::SystemTime;

use super::{
    block_refs::collect_block_refs,
    frontmatter::RawFrontmatter,
    headings::RawHeading,
    links::{RawLink, RawLinkStyle},
    list_items::{RawListDepth, RawListItem, RawListType},
    note::RawNote,
    sections::extract_sections,
    tags::scan_raw_tags,
    task_tokens::RawTaskTokens,
    tasks::RawTask,
};
use crate::note::{
    error::NoteError,
    parser::{
        ast::{AstLinkStyle, AstListType, AstNode, AstNodeKind, TextOrigin},
        frontmatter::MetadataBlock,
    },
    position::SourceByteOffset,
};

/// Extract raw note artifacts from AST nodes and metadata.
pub fn extract_raw_note(
    nodes: &[AstNode],
    frontmatter_block: Option<MetadataBlock>,
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
    let mut list_stack: Vec<RawListType> = Vec::new();

    let sections = extract_sections(nodes)?;

    let mut open_item_by_depth: Vec<SourceByteOffset> = Vec::new();

    for node in nodes {
        match node.kind() {
            AstNodeKind::Heading {
                level,
                text,
            } => {
                let raw = RawHeading::new(
                    *level,
                    text.to_string().into_boxed_str(),
                    node.range(),
                    node.range().start(),
                );
                headings.push(raw);
                scan_text_nodes(text, &mut tags)?;
            }
            AstNodeKind::Paragraph {
                text,
            } => {
                scan_text_nodes(text, &mut tags)?;
            }
            AstNodeKind::ListStart {
                list_type,
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
            }
            AstNodeKind::ListEnd => {
                list_stack.pop();
            }
            AstNodeKind::ListItem {
                text,
                task,
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
                let parent = parent_for_depth(depth_value, &open_item_by_depth);

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
                let raw_text = text.to_string();
                list_items.push(RawListItem::new(
                    list_type,
                    depth,
                    raw_text.clone().into_boxed_str(),
                    is_checkbox,
                    status_symbol,
                    position,
                    parent,
                ));

                if is_checkbox {
                    let raw_tags = scan_raw_tags(&raw_text, position)?;
                    let tags = raw_tags
                        .into_iter()
                        .map(|tag| tag.value().into())
                        .collect();
                    let tokens = RawTaskTokens::parse(&raw_text, &[]);
                    tasks.push(RawTask::new(
                        status_symbol,
                        raw_text.into_boxed_str(),
                        tags,
                        tokens.inline_fields().to_vec(),
                        tokens.emoji_dates().to_vec(),
                        position,
                    ));
                }

                scan_text_nodes(text, &mut tags)?;
            }
            AstNodeKind::Link {
                style,
                is_embed,
                target,
                alias,
            } => {
                let raw_style = match style {
                    AstLinkStyle::Wiki => RawLinkStyle::Wiki,
                    AstLinkStyle::Markdown => RawLinkStyle::Markdown,
                };
                let alias_text = alias.to_string();
                let alias = if alias_text.trim().is_empty() {
                    None
                } else {
                    Some(alias_text.into_boxed_str())
                };
                let (target_raw, anchor) =
                    super::links::split_raw_target_and_anchor(target);
                links.push(RawLink::new(
                    raw_style,
                    *is_embed,
                    target_raw.into(),
                    alias,
                    anchor.map(Into::into),
                    node.range().start(),
                ));
            }
            AstNodeKind::CodeBlock {
                ..
            }
            | AstNodeKind::BlockQuote {
                ..
            } => {}
        }
    }

    let frontmatter = frontmatter_block
        .map(|block| RawFrontmatter::new(block.kind(), block.text().into()));
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
        block_refs,
    ))
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
            markdown,
            path,
            "hash".into(),
            markdown.len() as u64,
            None,
            None,
        )?;

        assert_eq!(raw.tasks().len(), 1);
        let task = raw.tasks().first().expect("task should exist");
        assert_eq!(task.status_symbol(), Some(' '));
        assert!(task.tags().iter().any(|tag| tag.as_ref() == "#task"));
        assert!(
            task.inline_fields()
                .iter()
                .any(|(key, value)| key.as_ref() == "priority"
                    && value.as_ref() == "1")
        );
        Ok(())
    }
}
