use pulldown_cmark::{MetadataBlockKind, Tag as CmarkTag};

use crate::note::{
    error::NoteError,
    frontmatter::{Frontmatter, FrontmatterFormat},
    heading::Heading,
    position::{SourceByteOffset, SourceByteRange},
    structure::{BlockRef, BlockRefId, Section, SectionKind},
};

pub(super) fn handle_section_start(
    tag: &CmarkTag<'_>,
    start: usize,
    section_depth: &mut u32,
    current_section: &mut Option<(SectionKind, SourceByteOffset)>,
) -> Result<(), NoteError> {
    let Some(kind) = section_kind_for_tag(tag) else {
        return Ok(());
    };
    if *section_depth == 0 {
        let start = SourceByteOffset::try_from_usize(start)?;
        *current_section = Some((kind, start));
    }
    *section_depth = section_depth.saturating_add(1);
    Ok(())
}

pub(super) fn parse_frontmatter_block(
    kind: MetadataBlockKind,
    text: &str,
) -> Result<Option<Frontmatter>, NoteError> {
    if text.is_empty() {
        return Ok(None);
    }
    let format = match kind {
        MetadataBlockKind::YamlStyle => FrontmatterFormat::Yaml,
        MetadataBlockKind::PlusesStyle => FrontmatterFormat::Toml,
    };
    let parsed =
        Frontmatter::parse(format, text).map_err(NoteError::Frontmatter)?;
    Ok(Some(parsed))
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on borrowed pulldown-cmark tags"
)]
pub(super) fn section_kind_for_tag(tag: &CmarkTag<'_>) -> Option<SectionKind> {
    match tag {
        CmarkTag::Paragraph => Some(SectionKind::Paragraph),
        CmarkTag::Heading {
            ..
        } => Some(SectionKind::Heading),
        CmarkTag::List(_) => Some(SectionKind::List),
        CmarkTag::CodeBlock(_) => Some(SectionKind::Code),
        CmarkTag::BlockQuote(kind) => Some(if kind.is_some() {
            SectionKind::Callout
        } else {
            SectionKind::BlockQuote
        }),
        CmarkTag::Table(_) => Some(SectionKind::Table),
        CmarkTag::MetadataBlock(_) => Some(SectionKind::Frontmatter),
        CmarkTag::HtmlBlock
        | CmarkTag::Item
        | CmarkTag::FootnoteDefinition(_)
        | CmarkTag::DefinitionList
        | CmarkTag::DefinitionListTitle
        | CmarkTag::DefinitionListDefinition
        | CmarkTag::TableHead
        | CmarkTag::TableRow
        | CmarkTag::TableCell
        | CmarkTag::Emphasis
        | CmarkTag::Strong
        | CmarkTag::Strikethrough
        | CmarkTag::Superscript
        | CmarkTag::Subscript
        | CmarkTag::Link {
            ..
        }
        | CmarkTag::Image {
            ..
        } => None,
    }
}

pub(super) fn close_section(
    sections: &mut Vec<Section>,
    current_section: &mut Option<(SectionKind, SourceByteOffset)>,
    section_depth: &mut u32,
    event_range: std::ops::Range<usize>,
    heading: Option<Heading>,
) -> Result<(), NoteError> {
    if *section_depth == 0 {
        return Ok(());
    }
    *section_depth = section_depth.saturating_sub(1);
    if *section_depth > 0 {
        return Ok(());
    }
    let Some((kind, start)) = current_section.take() else {
        return Ok(());
    };
    let end = SourceByteOffset::try_from_usize(event_range.end)?;
    let source_range = SourceByteRange::new(start, end)?;
    sections.push(Section::new(kind, heading, source_range));
    Ok(())
}

pub(super) fn collect_block_refs(
    source: &str,
) -> Result<Vec<BlockRef>, NoteError> {
    let mut refs = Vec::new();
    let mut offset = 0usize;
    let mut in_code_block = false;
    let mut in_frontmatter = false;
    let mut frontmatter_fence: Option<&'static str> = None;

    for line in source.split_inclusive('\n') {
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
            if frontmatter_fence.is_some_and(|fence| fence == trimmed_line) {
                in_frontmatter = false;
            }
            offset = offset.saturating_add(line.len());
            continue;
        }

        let trimmed_start = trimmed_line.trim_start();
        if trimmed_start.starts_with("```") || trimmed_start.starts_with("~~~")
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
            let after =
                trimmed_line.get(caret_idx.saturating_add(1)..).unwrap_or("");
            let id = after.trim();
            let valid = !id.is_empty()
                && id.chars().all(|ch| {
                    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
                });
            if valid
                && (before.is_empty()
                    || before.chars().last().is_some_and(char::is_whitespace))
            {
                let position = SourceByteOffset::try_from_usize(
                    offset.saturating_add(caret_idx),
                )?;
                let block_id = BlockRefId::try_new(id)?;
                refs.push(BlockRef::new(block_id, position));
            }
        }
        offset = offset.saturating_add(line.len());
    }

    Ok(refs)
}
