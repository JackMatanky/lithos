//! Markdown parser boundary for note ingestion.
//!
//! This module is the ingestion boundary between file content and the raw
//! extraction layer. It is the only location in the note context that uses
//! `pulldown-cmark`. The parser consumes the event stream and emits a minimal
//! structural AST plus an optional raw metadata block, with byte ranges that
//! preserve source offsets for downstream extraction.
//!
//! Module components:
//! - `ast`: node and text definitions for the minimal structural AST.
//! - `frontmatter`: raw metadata block capture (fence kind + raw text).
//! - `note`: `ParsedNote`, the parser output container.
//!
//! Integration in the note pipeline:
//! - `parser` produces `ParsedNote` from markdown input.
//! - `raw` extracts `Raw*` facts from AST and metadata block.
//! - `aggregate` converts `Raw*` into domain facts using `TryFrom`.
//! - `storage` persists domain facts and builds indexes.
//!
//! Boundary guarantees:
//! - No domain validation or normalization is performed here.
//! - No raw extraction or configuration-driven parsing is performed here.
//! - `pulldown-cmark` types do not escape this module.

pub(crate) mod ast;
pub(crate) mod frontmatter;
pub(crate) mod note;

use std::ops::Range;

pub(crate) use note::ParsedNote;
use pulldown_cmark::{
    BlockQuoteKind, CodeBlockKind, Event, LinkType, OffsetIter, Options,
    Parser, Tag, TagEnd, utils::TextMergeWithOffset,
};

use self::{
    ast::{
        AstBlockQuoteKind, AstInlineLink, AstLinkStyle, AstListType, AstNode,
        AstNodeKind, Text, TextNode, TextOrigin, TextStyle,
    },
    frontmatter::MetadataBlock,
};
use crate::note::{
    error::NoteIngestError,
    position::{SourceByteOffset, SourceByteRange},
};

/// Returns the pulldown-cmark option set used for Obsidian-compatible parsing.
#[inline]
#[must_use]
pub const fn obsidian_options() -> Options {
    Options::ENABLE_TASKLISTS
        .union(Options::ENABLE_WIKILINKS)
        .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
        .union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
        .union(Options::ENABLE_HEADING_ATTRIBUTES)
        .union(Options::ENABLE_TABLES)
        .union(Options::ENABLE_FOOTNOTES)
        .union(Options::ENABLE_STRIKETHROUGH)
        .union(Options::ENABLE_MATH)
}

/// Parses markdown into a minimal AST and optional raw frontmatter block.
///
/// # Errors
///
/// Returns [`NoteIngestError`] if byte ranges cannot be represented or AST
/// construction fails.
#[inline]
pub fn parse_markdown(
    markdown: &str,
    options: Options,
) -> Result<ParsedNote, NoteIngestError> {
    ParserState::new(markdown, options).parse()
}

struct ParserState<'source> {
    inner: TextMergeWithOffset<'source, OffsetIter<'source>>,
}

impl<'source> ParserState<'source> {
    fn new(markdown: &'source str, options: Options) -> Self {
        let events = Parser::new_ext(markdown, options).into_offset_iter();
        let inner = TextMergeWithOffset::new(events);
        Self {
            inner,
        }
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Parser ignores unrelated events"
    )]
    fn parse(mut self) -> Result<ParsedNote, NoteIngestError> {
        let mut nodes = Vec::new();
        let mut frontmatter = None;

        while let Some((event, range)) = self.next() {
            let range = to_range(range)?;
            match event {
                Event::Start(Tag::MetadataBlock(kind)) => {
                    if frontmatter.is_none() {
                        let block = self.parse_metadata(kind)?;
                        frontmatter = Some(block);
                    } else {
                        self.consume_container(Tag::MetadataBlock(kind))?;
                    }
                }
                Event::Start(tag) if is_container_tag(&tag) => {
                    if let Some(node) =
                        self.parse_container(tag, range.start())?
                    {
                        nodes.push(node);
                    }
                }
                _ => {}
            }
        }

        Ok(ParsedNote::new(nodes, frontmatter))
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Metadata parsing ignores non-text events"
    )]
    fn parse_metadata(
        &mut self,
        kind: pulldown_cmark::MetadataBlockKind,
    ) -> Result<MetadataBlock, NoteIngestError> {
        let mut text = String::new();
        for (event, _range) in self.by_ref() {
            match event {
                Event::Text(value) | Event::Code(value) => {
                    text.push_str(&value);
                }
                Event::SoftBreak | Event::HardBreak => {
                    text.push('\n');
                }
                Event::End(TagEnd::MetadataBlock(end_kind))
                    if end_kind == kind =>
                {
                    return Ok(MetadataBlock::new(
                        kind.into(),
                        text.into_boxed_str(),
                    ));
                }
                _ => {}
            }
        }
        Err(NoteIngestError::Source("unclosed metadata block".into()))
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "Tag is lightweight and compared by value"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Container parsing ignores unrelated events"
    )]
    fn consume_container(
        &mut self,
        tag: Tag<'source>,
    ) -> Result<(), NoteIngestError> {
        while let Some((event, _range)) = self.next() {
            match event {
                Event::Start(inner) if is_container_tag(&inner) => {
                    self.consume_container(inner)?;
                }
                Event::End(end) if tag.to_end() == end => return Ok(()),
                _ => {}
            }
        }
        Err(NoteIngestError::Source("unclosed container".into()))
    }

    #[expect(
        clippy::cognitive_complexity,
        reason = "Parser must handle nested event streams"
    )]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "Tag is lightweight and compared by value"
    )]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics over reference patterns"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "Parser event handling is inherently verbose"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Parser ignores unrelated events"
    )]
    fn parse_container(
        &mut self,
        tag: Tag<'source>,
        start: SourceByteOffset,
    ) -> Result<Option<AstNode>, NoteIngestError> {
        let mut children = Vec::new();
        let mut text_nodes = Vec::new();
        let mut inline_links = Vec::new();
        let mut inline_styles = Vec::new();
        let mut current_link: Option<LinkFrame> = None;
        let mut task: Option<bool> = None;
        let mut code_text = String::new();

        let accepts_text =
            matches!(tag, Tag::Heading { .. } | Tag::Paragraph | Tag::Item);
        let is_code_block = matches!(tag, Tag::CodeBlock(_));
        let list_type = match &tag {
            Tag::List(list_start) => Some(match list_start {
                Some(start_num) => AstListType::Ordered {
                    start: *start_num,
                },
                None => AstListType::Unordered,
            }),
            _ => None,
        };
        let heading_level = match &tag {
            Tag::Heading {
                level,
                ..
            } => Some(heading_level(*level)),
            _ => None,
        };
        let block_quote_kind = match &tag {
            Tag::BlockQuote(kind) => kind.map(map_block_quote_kind),
            _ => None,
        };
        let (fenced, info) = match &tag {
            Tag::CodeBlock(kind) => match kind {
                CodeBlockKind::Fenced(info) => {
                    (true, Some(info.as_ref().into()))
                }
                CodeBlockKind::Indented => (false, None),
            },
            _ => (false, None),
        };

        while let Some((event, range)) = self.next() {
            let range = to_range(range)?;
            match event {
                Event::Start(inner) if is_container_tag(&inner) => {
                    if let Some(node) =
                        self.parse_container(inner, range.start())?
                    {
                        children.push(node);
                    }
                }
                Event::Start(Tag::Link {
                    link_type,
                    dest_url,
                    ..
                }) => {
                    current_link = Some(LinkFrame::new(
                        link_style(link_type),
                        false,
                        dest_url.as_ref().into(),
                        range.start(),
                    ));
                }
                Event::Start(Tag::Image {
                    link_type,
                    dest_url,
                    ..
                }) => {
                    current_link = Some(LinkFrame::new(
                        link_style(link_type),
                        true,
                        dest_url.as_ref().into(),
                        range.start(),
                    ));
                }
                Event::End(TagEnd::Link | TagEnd::Image) => {
                    if let Some(link) = current_link.take() {
                        inline_links.push(link.into_inline(range.end())?);
                    }
                }
                Event::Start(inner) => {
                    if let Some(style) = inline_style(&inner) {
                        inline_styles.push(style);
                    }
                }
                Event::End(end) if tag.to_end() == end => {
                    let range = SourceByteRange::new(start, range.end())
                        .map_err(NoteIngestError::Domain)?;
                    return Ok(build_node(
                        &tag,
                        range,
                        heading_level,
                        list_type,
                        block_quote_kind,
                        fenced,
                        info,
                        text_nodes,
                        inline_links,
                        task,
                        children,
                        code_text,
                    ));
                }
                Event::End(end) => {
                    if let Some(style) = inline_style_end(end) {
                        pop_style(&mut inline_styles, style);
                    }
                }
                Event::TaskListMarker(checked) => {
                    task = Some(checked);
                }
                Event::Text(text) => match (is_code_block, accepts_text) {
                    (true, _) => code_text.push_str(&text),
                    (false, true) => {
                        let style = current_style(&inline_styles);
                        let origin = link_origin(current_link.as_ref());
                        push_text_node(
                            &mut text_nodes,
                            &mut current_link,
                            &text,
                            style,
                            origin,
                            range,
                        );
                    }
                    _ => {}
                },
                Event::Code(text) => match (is_code_block, accepts_text) {
                    (true, _) => code_text.push_str(&text),
                    (false, true) => {
                        let origin = link_origin(current_link.as_ref());
                        push_text_node(
                            &mut text_nodes,
                            &mut current_link,
                            &text,
                            TextStyle::Code,
                            origin,
                            range,
                        );
                    }
                    _ => {}
                },
                Event::SoftBreak | Event::HardBreak => {
                    match (is_code_block, accepts_text) {
                        (true, _) => code_text.push('\n'),
                        (false, true) => {
                            let origin = link_origin(current_link.as_ref());
                            push_break_node(
                                &mut text_nodes,
                                &mut current_link,
                                origin,
                                range,
                            );
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        Ok(None)
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Iterator adapter forwards to inner iterator"
)]
impl<'source> Iterator for ParserState<'source> {
    type Item = (Event<'source>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

fn is_container_tag(tag: &Tag<'_>) -> bool {
    matches!(
        tag,
        Tag::Paragraph
            | Tag::Heading { .. }
            | Tag::BlockQuote(..)
            | Tag::CodeBlock(..)
            | Tag::HtmlBlock
            | Tag::List(..)
            | Tag::Item
            | Tag::FootnoteDefinition(..)
            | Tag::Table(..)
            | Tag::TableHead
            | Tag::TableRow
            | Tag::TableCell
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::MetadataBlock(..)
    )
}

#[expect(clippy::pattern_type_mismatch, reason = "Match ergonomics on &Tag")]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "Inline style ignores other tags"
)]
fn inline_style(tag: &Tag<'_>) -> Option<TextStyle> {
    match tag {
        Tag::Emphasis => Some(TextStyle::Emphasis),
        Tag::Strong => Some(TextStyle::Strong),
        Tag::Strikethrough => Some(TextStyle::Strikethrough),
        _ => None,
    }
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "Inline style ignores other end tags"
)]
fn inline_style_end(tag: TagEnd) -> Option<TextStyle> {
    match tag {
        TagEnd::Emphasis => Some(TextStyle::Emphasis),
        TagEnd::Strong => Some(TextStyle::Strong),
        TagEnd::Strikethrough => Some(TextStyle::Strikethrough),
        _ => None,
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Parser text node construction requires multiple fields"
)]
fn push_text_node(
    text_nodes: &mut Vec<TextNode>,
    current_link: &mut Option<LinkFrame>,
    text: &str,
    style: TextStyle,
    origin: TextOrigin,
    range: SourceByteRange,
) {
    if text.is_empty() {
        return;
    }
    let node = TextNode::new(text.into(), style, origin, range);
    text_nodes.push(node.clone());
    if let Some(link) = current_link.as_mut() {
        link.alias.push(node);
    }
}

fn push_break_node(
    text_nodes: &mut Vec<TextNode>,
    current_link: &mut Option<LinkFrame>,
    origin: TextOrigin,
    range: SourceByteRange,
) {
    if text_nodes.last().is_some_and(|node| node.content().ends_with(' ')) {
        return;
    }
    let node = TextNode::new(" ".into(), TextStyle::Plain, origin, range);
    text_nodes.push(node.clone());
    if let Some(link) = current_link.as_mut() {
        link.alias.push(node);
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Parser node construction requires multiple fields"
)]
fn build_node(
    tag: &Tag<'_>,
    range: SourceByteRange,
    heading_level: Option<u8>,
    list_type: Option<AstListType>,
    block_quote_kind: Option<AstBlockQuoteKind>,
    fenced: bool,
    info: Option<Box<str>>,
    text_nodes: Vec<TextNode>,
    inline_links: Vec<AstInlineLink>,
    task: Option<bool>,
    children: Vec<AstNode>,
    code_text: String,
) -> Option<AstNode> {
    let kind = match tag {
        &Tag::Heading {
            ..
        } => AstNodeKind::Heading {
            level: heading_level.unwrap_or(1),
            text: Text::new(text_nodes),
            links: inline_links,
        },
        &Tag::Paragraph => AstNodeKind::Paragraph {
            text: Text::new(text_nodes),
            links: inline_links,
        },
        &Tag::List(..) => AstNodeKind::List {
            list_type: list_type.unwrap_or(AstListType::Unordered),
            items: children,
        },
        &Tag::Item => AstNodeKind::ListItem {
            text: Text::new(text_nodes),
            task,
            links: inline_links,
            children,
        },
        &Tag::BlockQuote(..) => AstNodeKind::BlockQuote {
            kind: block_quote_kind,
            nodes: children,
        },
        &Tag::CodeBlock(..) => AstNodeKind::CodeBlock {
            fenced,
            info,
            text: code_text.into_boxed_str(),
        },
        &Tag::HtmlBlock
        | &Tag::FootnoteDefinition(..)
        | &Tag::Table(..)
        | &Tag::TableHead
        | &Tag::TableRow
        | &Tag::TableCell
        | &Tag::DefinitionList
        | &Tag::DefinitionListTitle
        | &Tag::DefinitionListDefinition
        | &Tag::MetadataBlock(..)
        | &Tag::Emphasis
        | &Tag::Strong
        | &Tag::Strikethrough
        | &Tag::Superscript
        | &Tag::Subscript
        | &Tag::Link {
            ..
        }
        | &Tag::Image {
            ..
        } => return None,
    };
    Some(AstNode::new(kind, range))
}

/// Parse-time state for link nodes.
struct LinkFrame {
    style: AstLinkStyle,
    is_embed: bool,
    target: Box<str>,
    alias: Vec<TextNode>,
    start: SourceByteOffset,
}

impl LinkFrame {
    fn new(
        style: AstLinkStyle,
        is_embed: bool,
        target: Box<str>,
        start: SourceByteOffset,
    ) -> Self {
        Self {
            style,
            is_embed,
            target,
            alias: Vec::new(),
            start,
        }
    }

    fn into_inline(
        self,
        end: SourceByteOffset,
    ) -> Result<AstInlineLink, NoteIngestError> {
        let range = SourceByteRange::new(self.start, end)
            .map_err(NoteIngestError::Domain)?;
        Ok(AstInlineLink::new(
            self.style,
            self.is_embed,
            self.target,
            Text::new(self.alias),
            range,
        ))
    }
}

fn to_range(range: Range<usize>) -> Result<SourceByteRange, NoteIngestError> {
    let start = SourceByteOffset::try_from_usize(range.start)
        .map_err(NoteIngestError::Domain)?;
    let end = SourceByteOffset::try_from_usize(range.end)
        .map_err(NoteIngestError::Domain)?;
    SourceByteRange::new(start, end).map_err(NoteIngestError::Domain)
}

fn heading_level(level: pulldown_cmark::HeadingLevel) -> u8 {
    match level {
        pulldown_cmark::HeadingLevel::H1 => 1,
        pulldown_cmark::HeadingLevel::H2 => 2,
        pulldown_cmark::HeadingLevel::H3 => 3,
        pulldown_cmark::HeadingLevel::H4 => 4,
        pulldown_cmark::HeadingLevel::H5 => 5,
        pulldown_cmark::HeadingLevel::H6 => 6,
    }
}

fn link_style(link_type: LinkType) -> AstLinkStyle {
    match link_type {
        LinkType::WikiLink {
            ..
        } => AstLinkStyle::Wiki,
        LinkType::Inline
        | LinkType::Reference
        | LinkType::ReferenceUnknown
        | LinkType::Collapsed
        | LinkType::CollapsedUnknown
        | LinkType::Shortcut
        | LinkType::ShortcutUnknown
        | LinkType::Autolink
        | LinkType::Email => AstLinkStyle::Markdown,
    }
}

fn map_block_quote_kind(kind: BlockQuoteKind) -> AstBlockQuoteKind {
    match kind {
        BlockQuoteKind::Note => AstBlockQuoteKind::Note,
        BlockQuoteKind::Tip => AstBlockQuoteKind::Tip,
        BlockQuoteKind::Important => AstBlockQuoteKind::Important,
        BlockQuoteKind::Warning => AstBlockQuoteKind::Warning,
        BlockQuoteKind::Caution => AstBlockQuoteKind::Caution,
    }
}

fn current_style(stack: &[TextStyle]) -> TextStyle {
    stack.last().copied().unwrap_or(TextStyle::Plain)
}

fn pop_style(stack: &mut Vec<TextStyle>, style: TextStyle) {
    if let Some(pos) = stack.iter().rposition(|item| *item == style) {
        stack.remove(pos);
    }
}

fn link_origin(current_link: Option<&LinkFrame>) -> TextOrigin {
    if current_link.is_some() {
        TextOrigin::LinkAlias
    } else {
        TextOrigin::Normal
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
#[expect(
    clippy::panic,
    reason = "Tests use explicit panics for invalid structures"
)]
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on &AstNodeKind"
)]
#[expect(
    clippy::shadow_unrelated,
    reason = "Test helpers reuse names for clarity"
)]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "Test helpers ignore unrelated node kinds"
)]
mod tests {
    use super::*;

    #[test]
    fn parses_task_list_marker_events() -> Result<(), NoteIngestError> {
        let options = Options::ENABLE_TASKLISTS;
        let parsed = parse_markdown("- [ ] task", options)?;
        let found = contains_list_item(parsed.nodes(), &|item| {
            matches!(item.kind(), AstNodeKind::ListItem {
                task: Some(false),
                ..
            })
        });
        assert!(found);
        Ok(())
    }

    #[test]
    fn captures_heading_node_with_range() -> Result<(), NoteIngestError> {
        let options = Options::ENABLE_HEADING_ATTRIBUTES;
        let parsed = parse_markdown("# Title", options)?;
        let mut found = false;
        for node in parsed.nodes() {
            if let AstNodeKind::Heading {
                level,
                ..
            } = node.kind()
            {
                assert_eq!(*level, 1);
                let range = node.range();
                assert!(!range.is_empty());
                found = true;
                break;
            }
        }
        assert!(found);
        Ok(())
    }

    #[test]
    fn tight_list_items_emit_no_paragraph_nodes() -> Result<(), NoteIngestError>
    {
        let options = Options::ENABLE_TASKLISTS;
        let parsed = parse_markdown("- one\n- two", options)?;
        let list = find_list(parsed.nodes()).expect("list node");

        if let AstNodeKind::List {
            items,
            ..
        } = list.kind()
        {
            for item in items {
                assert!(
                    !list_item_has_paragraph(item),
                    "tight list item should not contain paragraph nodes"
                );
            }
        } else {
            panic!("expected list node");
        }
        Ok(())
    }

    #[test]
    fn inline_links_attach_to_text_containers() -> Result<(), NoteIngestError> {
        let markdown =
            "# Heading with [link](https://example.com)\n\nParagraph with \
             [[wiki]] link.\n\n- Item with [ref](note.md)";
        let parsed = parse_markdown(markdown, obsidian_options())?;

        let heading = find_heading(parsed.nodes()).expect("heading node");
        if let AstNodeKind::Heading {
            links,
            ..
        } = heading.kind()
        {
            assert!(!links.is_empty(), "heading should capture inline links");
        }

        let paragraph = find_paragraph(parsed.nodes()).expect("paragraph node");
        if let AstNodeKind::Paragraph {
            links,
            ..
        } = paragraph.kind()
        {
            assert!(!links.is_empty(), "paragraph should capture inline links");
        }

        let list_item = find_list_item(parsed.nodes()).expect("list item node");
        if let AstNodeKind::ListItem {
            links,
            ..
        } = list_item.kind()
        {
            assert!(!links.is_empty(), "list item should capture inline links");
        }
        Ok(())
    }

    fn contains_list_item<F>(nodes: &[AstNode], predicate: &F) -> bool
    where
        F: Fn(&AstNode) -> bool,
    {
        for node in nodes {
            if predicate(node) {
                return true;
            }
            match node.kind() {
                AstNodeKind::List {
                    items,
                    ..
                } => {
                    if contains_list_item(items, predicate) {
                        return true;
                    }
                }
                AstNodeKind::ListItem {
                    children,
                    ..
                } => {
                    if contains_list_item(children, predicate) {
                        return true;
                    }
                }
                AstNodeKind::BlockQuote {
                    nodes,
                    ..
                } => {
                    if contains_list_item(nodes, predicate) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn find_list(nodes: &[AstNode]) -> Option<&AstNode> {
        nodes
            .iter()
            .find(|node| matches!(node.kind(), AstNodeKind::List { .. }))
    }

    fn find_heading(nodes: &[AstNode]) -> Option<&AstNode> {
        nodes
            .iter()
            .find(|node| matches!(node.kind(), AstNodeKind::Heading { .. }))
    }

    fn find_paragraph(nodes: &[AstNode]) -> Option<&AstNode> {
        for node in nodes {
            if matches!(node.kind(), AstNodeKind::Paragraph { .. }) {
                return Some(node);
            }
            if let AstNodeKind::BlockQuote {
                nodes: children,
                ..
            } = node.kind()
                && let Some(found) = find_paragraph(children)
            {
                return Some(found);
            }
        }
        None
    }

    fn find_list_item(nodes: &[AstNode]) -> Option<&AstNode> {
        for node in nodes {
            if matches!(node.kind(), AstNodeKind::ListItem { .. }) {
                return Some(node);
            }
            match node.kind() {
                AstNodeKind::List {
                    items,
                    ..
                } => {
                    if let Some(found) = find_list_item(items) {
                        return Some(found);
                    }
                }
                AstNodeKind::ListItem {
                    children,
                    ..
                } => {
                    if let Some(found) = find_list_item(children) {
                        return Some(found);
                    }
                }
                AstNodeKind::BlockQuote {
                    nodes,
                    ..
                } => {
                    if let Some(found) = find_list_item(nodes) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn list_item_has_paragraph(node: &AstNode) -> bool {
        if let AstNodeKind::ListItem {
            children,
            ..
        } = node.kind()
        {
            return children.iter().any(|child| {
                matches!(child.kind(), AstNodeKind::Paragraph { .. })
            });
        }
        false
    }
}
