//! Markdown parser boundary for note ingestion.

use std::ops::Range;

use pulldown_cmark::{
    BlockQuoteKind, CodeBlockKind, Event, LinkType, Options, Parser, Tag,
    TagEnd, utils::TextMergeWithOffset,
};

use self::{
    ast::{
        AstBlockQuoteKind, AstLinkStyle, AstListType, AstNode, AstNodeKind,
        Text, TextNode, TextOrigin, TextStyle,
    },
    frontmatter::{MetadataBlock, MetadataBlockKind},
    note::ParsedNote,
};
use crate::note::{
    error::NoteIngestError,
    position::{SourceByteOffset, SourceByteRange},
};

pub(crate) mod ast;
pub(crate) mod frontmatter;
pub(crate) mod note;

pub(crate) use note::ParsedNote;

/// Build the pulldown-cmark option set used for Obsidian-compatible parsing.
#[inline]
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

/// Parse markdown into a minimal AST plus raw frontmatter block.
pub fn parse_markdown(
    markdown: &str,
    options: Options,
) -> Result<ParsedNote, NoteIngestError> {
    let events = Parser::new_ext(markdown, options).into_offset_iter();
    let merged = TextMergeWithOffset::new(events);

    let mut nodes = Vec::new();
    let mut frontmatter_kind: Option<MetadataBlockKind> = None;
    let mut frontmatter_text = String::new();
    let mut frontmatter = None;

    let mut item_stack: Vec<ListItemBuilder> = Vec::new();
    let mut current_item: Option<ListItemBuilder> = None;
    let mut current_heading: Option<HeadingBuilder> = None;
    let mut current_paragraph: Option<ParagraphBuilder> = None;
    let mut current_link: Option<LinkBuilder> = None;
    let mut current_code_block: Option<CodeBlockBuilder> = None;
    let mut quote_stack: Vec<BlockQuoteBuilder> = Vec::new();

    let mut code_block_depth = 0u32;
    let mut inline_styles: Vec<TextStyle> = Vec::new();

    for (event, range) in merged {
        let range = to_range(range)?;
        match event {
            Event::Start(Tag::MetadataBlock(kind)) => {
                frontmatter_kind = Some(kind.into());
                frontmatter_text.clear();
            }
            Event::End(TagEnd::MetadataBlock(kind)) => {
                let kind = MetadataBlockKind::from(kind);
                if frontmatter_kind == Some(kind) && frontmatter.is_none() {
                    if !frontmatter_text.is_empty() {
                        frontmatter = Some(MetadataBlock::new(
                            kind,
                            frontmatter_text.clone().into_boxed_str(),
                        ));
                    }
                }
                frontmatter_kind = None;
                frontmatter_text.clear();
            }
            Event::Start(Tag::Heading {
                level,
                ..
            }) => {
                current_heading = Some(HeadingBuilder::new(
                    heading_level(level),
                    range.start(),
                ));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(builder) = current_heading.take() {
                    let node = builder.finish(range.end())?;
                    nodes.push(node);
                }
            }
            Event::Start(Tag::Paragraph) => {
                current_paragraph = Some(ParagraphBuilder::new(range.start()));
            }
            Event::End(TagEnd::Paragraph) => {
                if let Some(builder) = current_paragraph.take() {
                    let node = builder.finish(range.end())?;
                    let is_empty_paragraph = matches!(node.kind(), AstNodeKind::Paragraph { text } if text.is_empty());
                    if !is_empty_paragraph {
                        nodes.push(node);
                    }
                }
            }
            Event::Start(Tag::List(start)) => {
                let list_type = match start {
                    Some(start_num) => AstListType::Ordered {
                        start: start_num,
                    },
                    None => AstListType::Unordered,
                };
                nodes.push(AstNode::new(
                    AstNodeKind::ListStart {
                        list_type,
                    },
                    range,
                ));
            }
            Event::End(TagEnd::List(_)) => {
                nodes.push(AstNode::new(AstNodeKind::ListEnd, range));
            }
            Event::Start(Tag::Item) => {
                if let Some(active) = current_item.take() {
                    item_stack.push(active);
                }
                current_item = Some(ListItemBuilder::new(range.start()));
            }
            Event::End(TagEnd::Item) => {
                if let Some(builder) = current_item.take() {
                    let node = builder.finish(range.end())?;
                    nodes.push(node);
                }
                current_item = item_stack.pop();
            }
            Event::TaskListMarker(checked) => {
                if let Some(item) = current_item.as_mut() {
                    item.task = Some(checked);
                }
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                ..
            }) => {
                current_link = Some(LinkBuilder::new(
                    link_style(link_type),
                    false,
                    dest_url.to_string().into_boxed_str(),
                    range.start(),
                ));
            }
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                ..
            }) => {
                current_link = Some(LinkBuilder::new(
                    link_style(link_type),
                    true,
                    dest_url.to_string().into_boxed_str(),
                    range.start(),
                ));
            }
            Event::End(TagEnd::Link | TagEnd::Image) => {
                if let Some(builder) = current_link.take() {
                    nodes.push(builder.finish(range.end())?);
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                code_block_depth = code_block_depth.saturating_add(1);
                current_code_block =
                    Some(CodeBlockBuilder::new(kind, range.start()));
            }
            Event::End(TagEnd::CodeBlock) => {
                code_block_depth = code_block_depth.saturating_sub(1);
                if let Some(builder) = current_code_block.take() {
                    nodes.push(builder.finish(range.end())?);
                }
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                quote_stack.push(BlockQuoteBuilder::new(
                    kind.map(map_block_quote_kind),
                    range.start(),
                ));
            }
            Event::End(TagEnd::BlockQuote) => {
                if let Some(builder) = quote_stack.pop() {
                    nodes.push(builder.finish(range.end())?);
                }
            }
            Event::Start(Tag::Emphasis) => {
                inline_styles.push(TextStyle::Emphasis);
            }
            Event::End(TagEnd::Emphasis) => {
                pop_style(&mut inline_styles, TextStyle::Emphasis);
            }
            Event::Start(Tag::Strong) => {
                inline_styles.push(TextStyle::Strong);
            }
            Event::End(TagEnd::Strong) => {
                pop_style(&mut inline_styles, TextStyle::Strong);
            }
            Event::Start(Tag::Strikethrough) => {
                inline_styles.push(TextStyle::Strikethrough);
            }
            Event::End(TagEnd::Strikethrough) => {
                pop_style(&mut inline_styles, TextStyle::Strikethrough);
            }
            Event::Text(text) => {
                if frontmatter_kind.is_some() {
                    frontmatter_text.push_str(&text);
                    continue;
                }
                if code_block_depth > 0 {
                    continue;
                }
                let style = current_style(&inline_styles);
                let origin = if current_link.is_some() {
                    TextOrigin::LinkAlias
                } else {
                    TextOrigin::Normal
                };
                push_text(
                    &mut current_heading,
                    &mut current_paragraph,
                    &mut current_item,
                    &mut current_link,
                    &text,
                    style,
                    origin,
                    range,
                );
            }
            Event::Code(text) => {
                if frontmatter_kind.is_some() {
                    frontmatter_text.push_str(&text);
                    continue;
                }
                if code_block_depth > 0 {
                    continue;
                }
                let origin = if current_link.is_some() {
                    TextOrigin::LinkAlias
                } else {
                    TextOrigin::Normal
                };
                push_text(
                    &mut current_heading,
                    &mut current_paragraph,
                    &mut current_item,
                    &mut current_link,
                    &text,
                    TextStyle::Code,
                    origin,
                    range,
                );
            }
            Event::SoftBreak | Event::HardBreak => {
                if frontmatter_kind.is_some() {
                    frontmatter_text.push('\n');
                    continue;
                }
                if code_block_depth > 0 {
                    continue;
                }
                let origin = if current_link.is_some() {
                    TextOrigin::LinkAlias
                } else {
                    TextOrigin::Normal
                };
                push_break(
                    &mut current_heading,
                    &mut current_paragraph,
                    &mut current_item,
                    &mut current_link,
                    origin,
                    range,
                );
            }
            Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::Rule
            | Event::Start(_)
            | Event::End(_) => {}
        }
    }

    Ok(ParsedNote::new(nodes, frontmatter))
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
        _ => AstLinkStyle::Markdown,
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

fn push_text(
    heading: &mut Option<HeadingBuilder>,
    paragraph: &mut Option<ParagraphBuilder>,
    item: &mut Option<ListItemBuilder>,
    link: &mut Option<LinkBuilder>,
    text: &str,
    style: TextStyle,
    origin: TextOrigin,
    range: SourceByteRange,
) {
    if let Some(builder) = heading.as_mut() {
        builder.text.push(text, style, origin, range);
    }
    if let Some(builder) = paragraph.as_mut() {
        builder.text.push(text, style, origin, range);
    }
    if let Some(builder) = item.as_mut() {
        builder.text.push(text, style, origin, range);
    }
    if let Some(builder) = link.as_mut() {
        builder.alias.push(text, style, TextOrigin::LinkAlias, range);
    }
}

fn push_break(
    heading: &mut Option<HeadingBuilder>,
    paragraph: &mut Option<ParagraphBuilder>,
    item: &mut Option<ListItemBuilder>,
    link: &mut Option<LinkBuilder>,
    origin: TextOrigin,
    range: SourceByteRange,
) {
    if let Some(builder) = heading.as_mut() {
        builder.text.push_break(origin, range);
    }
    if let Some(builder) = paragraph.as_mut() {
        builder.text.push_break(origin, range);
    }
    if let Some(builder) = item.as_mut() {
        builder.text.push_break(origin, range);
    }
    if let Some(builder) = link.as_mut() {
        builder.alias.push_break(TextOrigin::LinkAlias, range);
    }
}

/// Internal accumulator for inline text fragments.
struct TextBuilder {
    nodes: Vec<TextNode>,
}

impl TextBuilder {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    fn push(
        &mut self,
        text: &str,
        style: TextStyle,
        origin: TextOrigin,
        range: SourceByteRange,
    ) {
        if text.is_empty() {
            return;
        }
        self.nodes.push(TextNode::new(text.into(), style, origin, range));
    }

    fn push_break(&mut self, origin: TextOrigin, range: SourceByteRange) {
        if self.nodes.last().is_some_and(|node| node.content().ends_with(' ')) {
            return;
        }
        self.nodes.push(TextNode::new(
            " ".into(),
            TextStyle::Plain,
            origin,
            range,
        ));
    }

    fn finish(self) -> Text {
        Text::new(self.nodes)
    }
}

/// Internal accumulator for heading nodes.
struct HeadingBuilder {
    level: u8,
    start: SourceByteOffset,
    text: TextBuilder,
}

impl HeadingBuilder {
    fn new(level: u8, start: SourceByteOffset) -> Self {
        Self {
            level,
            start,
            text: TextBuilder::new(),
        }
    }

    fn finish(self, end: SourceByteOffset) -> Result<AstNode, NoteIngestError> {
        let range = SourceByteRange::new(self.start, end)
            .map_err(NoteIngestError::Domain)?;
        Ok(AstNode::new(
            AstNodeKind::Heading {
                level: self.level,
                text: self.text.finish(),
            },
            range,
        ))
    }
}

/// Internal accumulator for paragraph nodes.
struct ParagraphBuilder {
    start: SourceByteOffset,
    text: TextBuilder,
}

impl ParagraphBuilder {
    fn new(start: SourceByteOffset) -> Self {
        Self {
            start,
            text: TextBuilder::new(),
        }
    }

    fn finish(self, end: SourceByteOffset) -> Result<AstNode, NoteIngestError> {
        let range = SourceByteRange::new(self.start, end)
            .map_err(NoteIngestError::Domain)?;
        Ok(AstNode::new(
            AstNodeKind::Paragraph {
                text: self.text.finish(),
            },
            range,
        ))
    }
}

/// Internal accumulator for list item nodes.
struct ListItemBuilder {
    start: SourceByteOffset,
    text: TextBuilder,
    task: Option<bool>,
}

impl ListItemBuilder {
    fn new(start: SourceByteOffset) -> Self {
        Self {
            start,
            text: TextBuilder::new(),
            task: None,
        }
    }

    fn finish(self, end: SourceByteOffset) -> Result<AstNode, NoteIngestError> {
        let range = SourceByteRange::new(self.start, end)
            .map_err(NoteIngestError::Domain)?;
        Ok(AstNode::new(
            AstNodeKind::ListItem {
                text: self.text.finish(),
                task: self.task,
            },
            range,
        ))
    }
}

/// Internal accumulator for link nodes.
struct LinkBuilder {
    style: AstLinkStyle,
    is_embed: bool,
    target: Box<str>,
    start: SourceByteOffset,
    alias: TextBuilder,
}

impl LinkBuilder {
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
            start,
            alias: TextBuilder::new(),
        }
    }

    fn finish(self, end: SourceByteOffset) -> Result<AstNode, NoteIngestError> {
        let range = SourceByteRange::new(self.start, end)
            .map_err(NoteIngestError::Domain)?;
        Ok(AstNode::new(
            AstNodeKind::Link {
                style: self.style,
                is_embed: self.is_embed,
                target: self.target,
                alias: self.alias.finish(),
            },
            range,
        ))
    }
}

/// Internal accumulator for code block nodes.
struct CodeBlockBuilder {
    fenced: bool,
    info: Option<Box<str>>,
    start: SourceByteOffset,
}

impl CodeBlockBuilder {
    fn new(kind: CodeBlockKind<'_>, start: SourceByteOffset) -> Self {
        match kind {
            CodeBlockKind::Indented => Self {
                fenced: false,
                info: None,
                start,
            },
            CodeBlockKind::Fenced(info) => Self {
                fenced: true,
                info: Some(info.to_string().into_boxed_str()),
                start,
            },
        }
    }

    fn finish(self, end: SourceByteOffset) -> Result<AstNode, NoteIngestError> {
        let range = SourceByteRange::new(self.start, end)
            .map_err(NoteIngestError::Domain)?;
        Ok(AstNode::new(
            AstNodeKind::CodeBlock {
                fenced: self.fenced,
                info: self.info,
            },
            range,
        ))
    }
}

/// Internal accumulator for block quote nodes.
struct BlockQuoteBuilder {
    kind: Option<AstBlockQuoteKind>,
    start: SourceByteOffset,
}

impl BlockQuoteBuilder {
    fn new(kind: Option<AstBlockQuoteKind>, start: SourceByteOffset) -> Self {
        Self {
            kind,
            start,
        }
    }

    fn finish(self, end: SourceByteOffset) -> Result<AstNode, NoteIngestError> {
        let range = SourceByteRange::new(self.start, end)
            .map_err(NoteIngestError::Domain)?;
        Ok(AstNode::new(
            AstNodeKind::BlockQuote {
                kind: self.kind,
            },
            range,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_task_list_marker_events() -> Result<(), NoteIngestError> {
        let options = Options::ENABLE_TASKLISTS;
        let parsed = parse_markdown("- [ ] task", options)?;
        let found = parsed.nodes().iter().any(|node| {
            matches!(node.kind(), AstNodeKind::ListItem {
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
                assert!(range.len() > 0);
                found = true;
                break;
            }
        }
        assert!(found);
        Ok(())
    }
}
