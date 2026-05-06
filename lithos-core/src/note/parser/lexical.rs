#![expect(
    clippy::arithmetic_side_effects,
    reason = "Byte-span arithmetic is explicit and bounds-checked via slices"
)]

use super::text::{TextContext, TextSequence};
use crate::note::{
    error::NoteError,
    position::{SourceByteOffset, SourceByteRange, SourceByteRangeIndex},
    raw::{RawBlockRef, RawInlineFieldToken, RawTag},
};

#[expect(
    dead_code,
    reason = "Artifact-specific policy branching is staged during parser \
              unification"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArtifactKind {
    Tag,
    InlineField,
    BlockRef,
}

pub(crate) trait ScanPolicy: Send + Sync {
    fn allow(&self, artifact: ArtifactKind, ctx: TextContext) -> bool;
}

#[derive(Debug, Default)]
pub(crate) struct DefaultScanPolicy;

impl ScanPolicy for DefaultScanPolicy {
    fn allow(&self, _artifact: ArtifactKind, ctx: TextContext) -> bool {
        !ctx.contains(TextContext::IN_LINK_LABEL)
            && !ctx.contains(TextContext::IN_IMAGE_ALT)
            && !ctx.contains(TextContext::IN_CODE_INLINE)
            && !ctx.contains(TextContext::IN_MATH_INLINE)
            && !ctx.contains(TextContext::IN_MATH_DISPLAY)
            && !ctx.contains(TextContext::IN_CODE_BLOCK)
            && !ctx.contains(TextContext::IN_FRONTMATTER)
    }
}

pub(crate) fn build_scan_index(
    projection: &TextSequence,
    policy: &dyn ScanPolicy,
) -> SourceByteRangeIndex {
    let mut index = SourceByteRangeIndex::new();
    for node in projection.nodes() {
        if policy.allow(ArtifactKind::Tag, node.context()) {
            index.push(node.range());
        }
    }
    index.sort_and_coalesce();
    index
}

#[derive(Debug, Default)]
pub(crate) struct LexicalArtifacts<'source> {
    tags: Vec<RawTag<'source>>,
    inline_fields: Vec<RawInlineFieldToken<'source>>,
    block_refs: Vec<RawBlockRef<'source>>,
}

impl<'source> LexicalArtifacts<'source> {
    #[must_use]
    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<RawTag<'source>>,
        Vec<RawInlineFieldToken<'source>>,
        Vec<RawBlockRef<'source>>,
    ) {
        (self.tags, self.inline_fields, self.block_refs)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ArtifactLexer {
    emoji_markers: Box<[char]>,
}

impl ArtifactLexer {
    #[must_use]
    pub(crate) fn new<T: Into<Box<[char]>>>(emoji_markers: T) -> Self {
        Self {
            emoji_markers: emoji_markers.into(),
        }
    }

    pub(crate) fn collect<'source>(
        &self,
        source: &'source str,
        projection: &TextSequence,
        policy: &dyn ScanPolicy,
    ) -> Result<LexicalArtifacts<'source>, NoteError> {
        let index = build_scan_index(projection, policy);
        let tags = collect_tags(source, &index)?;
        let inline_fields =
            collect_inline_fields(source, &index, &self.emoji_markers)?;
        let block_refs = collect_block_refs(source, &index)?;
        Ok(LexicalArtifacts {
            tags,
            inline_fields,
            block_refs,
        })
    }
}

fn collect_tags<'source>(
    source: &'source str,
    index: &SourceByteRangeIndex,
) -> Result<Vec<RawTag<'source>>, NoteError> {
    let mut tags = Vec::with_capacity(8);
    for range in index {
        let Some(segment) = source.get(range.as_usize_range()) else {
            continue;
        };
        let base = range.start();
        scan_tags_in_segment(segment, base, &mut tags)?;
    }
    Ok(tags)
}

fn collect_inline_fields<'source>(
    source: &'source str,
    index: &SourceByteRangeIndex,
    emoji_markers: &[char],
) -> Result<Vec<RawInlineFieldToken<'source>>, NoteError> {
    let mut fields = Vec::with_capacity(8);
    for range in index {
        let Some(segment) = source.get(range.as_usize_range()) else {
            continue;
        };
        let base = range.start();
        scan_inline_fields_in_segment(
            segment,
            base,
            emoji_markers,
            &mut fields,
        )?;
    }
    Ok(fields)
}

fn collect_block_refs<'source>(
    source: &'source str,
    index: &SourceByteRangeIndex,
) -> Result<Vec<RawBlockRef<'source>>, NoteError> {
    let mut refs = Vec::with_capacity(8);
    for range in index {
        let Some(segment) = source.get(range.as_usize_range()) else {
            continue;
        };
        let base = range.start();
        scan_block_refs_in_segment(segment, base, &mut refs)?;
    }
    Ok(refs)
}

fn scan_tags_in_segment<'source>(
    segment: &'source str,
    base: SourceByteOffset,
    out: &mut Vec<RawTag<'source>>,
) -> Result<(), NoteError> {
    for (idx, ch) in segment.char_indices() {
        if ch != '#' {
            continue;
        }
        let prev_is_alnum = idx > 0
            && segment
                .get(..idx)
                .and_then(|prefix| prefix.chars().next_back())
                .is_some_and(char::is_alphanumeric);
        if prev_is_alnum {
            continue;
        }

        let mut len = 1usize;
        let mut has_content = false;
        for c in segment.get(idx + 1..).unwrap_or("").chars() {
            if c.is_alphanumeric() || matches!(c, '_' | '-' | '/') {
                len = len.saturating_add(c.len_utf8());
                has_content = true;
            } else {
                break;
            }
        }
        if !has_content {
            continue;
        }

        let Some(value) = segment.get(idx..idx.saturating_add(len)) else {
            continue;
        };
        let start = base.add_offset(idx)?;
        let end = start.add_offset(len)?;
        out.push(RawTag::new(value.into(), SourceByteRange::new(start, end)?));
    }
    Ok(())
}

fn scan_inline_fields_in_segment<'source>(
    segment: &'source str,
    base: SourceByteOffset,
    emoji_markers: &[char],
    out: &mut Vec<RawInlineFieldToken<'source>>,
) -> Result<(), NoteError> {
    for (line_start, line) in lines_with_offsets(segment) {
        let leading_ws = line
            .chars()
            .take_while(|c| c.is_whitespace() && *c != '\n' && *c != '\r')
            .map(char::len_utf8)
            .sum::<usize>();
        let trimmed = line.get(leading_ws..).unwrap_or("");

        if let Some((key, value, total_len)) = parse_bare_field(trimmed)
            && !value.is_empty()
        {
            let abs = base.add_offset(line_start.saturating_add(leading_ws))?;
            let end = abs.add_offset(total_len)?;
            out.push(RawInlineFieldToken::new(
                key.into(),
                value.into(),
                SourceByteRange::new(abs, end)?,
            ));
        }

        scan_delimited_fields_in_line(line, line_start, base, out)?;
        scan_emoji_fields_in_line(line, line_start, base, emoji_markers, out)?;
    }
    Ok(())
}

fn scan_delimited_fields_in_line<'source>(
    line: &'source str,
    line_start: usize,
    base: SourceByteOffset,
    out: &mut Vec<RawInlineFieldToken<'source>>,
) -> Result<(), NoteError> {
    for (idx, ch) in line.char_indices() {
        let closer = match ch {
            '[' => ']',
            '(' => ')',
            _ => continue,
        };
        let Some(rest) = line.get(idx + ch.len_utf8()..) else {
            continue;
        };
        let Some(sep) = rest.find("::") else {
            continue;
        };
        let after_sep = sep.saturating_add(2);
        let Some(close_rel) =
            rest.get(after_sep..).and_then(|s| s.find(closer))
        else {
            continue;
        };

        let key = rest.get(..sep).unwrap_or("").trim();
        if key.is_empty() {
            continue;
        }
        let value = rest
            .get(after_sep..after_sep.saturating_add(close_rel))
            .unwrap_or("")
            .trim();
        let consumed = ch
            .len_utf8()
            .saturating_add(after_sep)
            .saturating_add(close_rel)
            .saturating_add(closer.len_utf8());

        let abs = base.add_offset(line_start.saturating_add(idx))?;
        let end = abs.add_offset(consumed)?;
        out.push(RawInlineFieldToken::new(
            key.into(),
            value.into(),
            SourceByteRange::new(abs, end)?,
        ));
    }
    Ok(())
}

fn scan_emoji_fields_in_line<'source>(
    line: &'source str,
    line_start: usize,
    base: SourceByteOffset,
    emoji_markers: &[char],
    out: &mut Vec<RawInlineFieldToken<'source>>,
) -> Result<(), NoteError> {
    let Some(first) = line.chars().next() else {
        return Ok(());
    };
    if !emoji_markers.contains(&first) {
        return Ok(());
    }

    let mut consumed = first.len_utf8();
    let rest = line.get(consumed..).unwrap_or("");
    let ws_len = rest
        .chars()
        .take_while(|c| c.is_whitespace() && *c != '\n' && *c != '\r')
        .map(char::len_utf8)
        .sum::<usize>();
    consumed = consumed.saturating_add(ws_len);

    let value = line.get(consumed..).unwrap_or("").trim();
    if value.is_empty() {
        return Ok(());
    }

    let abs = base.add_offset(line_start)?;
    let end = abs.add_offset(line.trim_end_matches(['\r', '\n']).len())?;
    out.push(RawInlineFieldToken::new(
        first.to_string().into(),
        value.into(),
        SourceByteRange::new(abs, end)?,
    ));
    Ok(())
}

fn scan_block_refs_in_segment<'source>(
    segment: &'source str,
    base: SourceByteOffset,
    out: &mut Vec<RawBlockRef<'source>>,
) -> Result<(), NoteError> {
    for (idx, ch) in segment.char_indices() {
        if ch != '^' {
            continue;
        }
        let prev_is_alnum = idx > 0
            && segment
                .get(..idx)
                .and_then(|prefix| prefix.chars().next_back())
                .is_some_and(char::is_alphanumeric);
        if prev_is_alnum {
            continue;
        }

        let mut len = 1usize;
        let mut has_content = false;
        for c in segment.get(idx + 1..).unwrap_or("").chars() {
            if c.is_alphanumeric() || matches!(c, '-' | '_') {
                len = len.saturating_add(c.len_utf8());
                has_content = true;
            } else {
                break;
            }
        }
        if !has_content {
            continue;
        }

        let remaining = segment.get(idx + len..).unwrap_or("");
        let has_non_ws_tail = remaining
            .chars()
            .take_while(|c| *c != '\n' && *c != '\r')
            .any(|c| !c.is_whitespace());
        if has_non_ws_tail {
            continue;
        }

        let Some(id) = segment.get(idx + 1..idx + len) else {
            continue;
        };
        let start = base.add_offset(idx)?;
        out.push(RawBlockRef::new(id.into(), start));
    }
    Ok(())
}

fn lines_with_offsets(input: &str) -> Vec<(usize, &str)> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    for (idx, ch) in input.char_indices() {
        if ch == '\n' {
            let end = idx.saturating_add(1);
            lines.push((start, input.get(start..end).unwrap_or("")));
            start = end;
        }
    }
    if start <= input.len() {
        lines.push((start, input.get(start..).unwrap_or("")));
    }
    lines
}

fn parse_bare_field(line: &str) -> Option<(&str, &str, usize)> {
    let mut key_len = 0usize;
    for b in line.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-') {
            key_len = key_len.saturating_add(1);
        } else {
            break;
        }
    }
    if key_len == 0 || !line.get(key_len..).is_some_and(|s| s.starts_with("::"))
    {
        return None;
    }
    let key = line.get(..key_len)?;
    let after = line.get(key_len.saturating_add(2)..).unwrap_or("");
    let val = after.trim();
    let consumed = key_len.saturating_add(2).saturating_add(after.len());
    Some((key, val, consumed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::{
        parser::text::{TextNode, TextStyle},
        position::SourceByteRange,
    };

    fn range() -> SourceByteRange {
        SourceByteRange::try_from(0..3).expect("valid range")
    }

    #[test]
    fn default_policy_excludes_link_and_code_math_contexts() {
        let policy = DefaultScanPolicy;

        assert!(policy.allow(ArtifactKind::Tag, TextContext::NONE));
        assert!(!policy.allow(ArtifactKind::Tag, TextContext::IN_LINK_LABEL));
        assert!(!policy.allow(ArtifactKind::Tag, TextContext::IN_CODE_INLINE));
        assert!(!policy.allow(ArtifactKind::Tag, TextContext::IN_MATH_INLINE));
    }

    #[test]
    fn build_scan_index_keeps_only_allowed_ranges() {
        let nodes = vec![
            TextNode::new(
                "ok".into(),
                TextStyle::NONE,
                TextContext::NONE,
                range(),
            ),
            TextNode::new(
                "skip".into(),
                TextStyle::NONE,
                TextContext::IN_LINK_LABEL,
                SourceByteRange::try_from(4..8).expect("valid range"),
            ),
        ];
        let seq = TextSequence::from_nodes(nodes);
        let index = build_scan_index(&seq, &DefaultScanPolicy);

        assert_eq!(index.len(), 1);
        assert_eq!(
            index
                .iter()
                .next()
                .map(crate::note::position::SourceByteRange::as_usize_range),
            Some(0..3)
        );
    }

    #[test]
    fn artifact_lexer_collects_visible_tag() {
        let lexer = ArtifactLexer::new(Vec::<char>::new());
        let node = TextNode::new(
            "#ok".into(),
            TextStyle::NONE,
            TextContext::NONE,
            SourceByteRange::try_from(0..3).expect("range"),
        );
        let seq = TextSequence::from_nodes(vec![node]);

        let out =
            lexer.collect("#ok", &seq, &DefaultScanPolicy).expect("collect");
        let (tags, _fields, _refs) = out.into_parts();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags.first().map(|tag| tag.value.as_ref()), Some("#ok"));
    }

    #[test]
    fn collect_tags_ignores_alnum_prefixed_hash() {
        let lexer = ArtifactLexer::new(Vec::<char>::new());
        let source = "a#skip #ok";
        let seq = TextSequence::from_nodes(vec![TextNode::new(
            source.into(),
            TextStyle::NONE,
            TextContext::NONE,
            SourceByteRange::try_from(0..source.len()).expect("range"),
        )]);

        let out =
            lexer.collect(source, &seq, &DefaultScanPolicy).expect("collect");
        let (tags, _fields, _refs) = out.into_parts();
        assert_eq!(tags.len(), 1, "only standalone tag should be collected");
        assert_eq!(
            tags.first().map(|tag| tag.value.as_ref()),
            Some("#ok"),
            "standalone tag captured"
        );
    }

    #[test]
    fn collect_bare_field_only_at_line_start() {
        let lexer = ArtifactLexer::new(Vec::<char>::new());
        let source = "key:: value\ntext key:: no\n  indented:: yes";
        let seq = TextSequence::from_nodes(vec![TextNode::new(
            source.into(),
            TextStyle::NONE,
            TextContext::NONE,
            SourceByteRange::try_from(0..source.len()).expect("range"),
        )]);

        let out =
            lexer.collect(source, &seq, &DefaultScanPolicy).expect("collect");
        let (_tags, fields, _refs) = out.into_parts();
        assert_eq!(fields.len(), 2, "only line-start bare fields are allowed");
        assert_eq!(fields.first().map(|field| field.key.as_ref()), Some("key"));
        assert_eq!(
            fields.get(1).map(|field| field.key.as_ref()),
            Some("indented")
        );
    }

    #[test]
    fn collect_block_ref_requires_line_tail_whitespace_or_eol() {
        let lexer = ArtifactLexer::new(Vec::<char>::new());
        let source = "ok ^id\nbad ^id tail\nend ^last   ";
        let seq = TextSequence::from_nodes(vec![TextNode::new(
            source.into(),
            TextStyle::NONE,
            TextContext::NONE,
            SourceByteRange::try_from(0..source.len()).expect("range"),
        )]);

        let out =
            lexer.collect(source, &seq, &DefaultScanPolicy).expect("collect");
        let (_tags, _fields, refs) = out.into_parts();
        assert_eq!(
            refs.len(),
            2,
            "block refs with non-whitespace tail excluded"
        );
        assert_eq!(refs.first().map(|item| item.id.as_ref()), Some("id"));
        assert_eq!(refs.get(1).map(|item| item.id.as_ref()), Some("last"));
    }

    #[test]
    fn collect_disjoint_ranges_do_not_leak_state() {
        let lexer = ArtifactLexer::new(Vec::<char>::new());
        let source = "#first\n[[skip:: hidden]]\n#second";
        let nodes = vec![
            TextNode::new(
                "#first".into(),
                TextStyle::NONE,
                TextContext::NONE,
                SourceByteRange::try_from(0..6).expect("range"),
            ),
            TextNode::new(
                "#second".into(),
                TextStyle::NONE,
                TextContext::NONE,
                SourceByteRange::try_from(25..32).expect("range"),
            ),
        ];
        let seq = TextSequence::from_nodes(nodes);

        let out =
            lexer.collect(source, &seq, &DefaultScanPolicy).expect("collect");
        let (tags, fields, refs) = out.into_parts();
        assert_eq!(tags.len(), 2, "both disjoint visible ranges scanned");
        assert!(
            fields.is_empty(),
            "excluded middle range not scanned for fields"
        );
        assert!(refs.is_empty(), "excluded middle range not scanned for refs");
    }

    #[test]
    fn collect_emoji_field_uses_configured_markers() {
        let lexer = ArtifactLexer::new(vec!['\u{2705}']);
        let source = "\u{2705} done\n\u{1f4c5} date";
        let seq = TextSequence::from_nodes(vec![TextNode::new(
            source.into(),
            TextStyle::NONE,
            TextContext::NONE,
            SourceByteRange::try_from(0..source.len()).expect("range"),
        )]);

        let out =
            lexer.collect(source, &seq, &DefaultScanPolicy).expect("collect");
        let (_tags, fields, _refs) = out.into_parts();
        assert_eq!(fields.len(), 1, "only configured emoji marker is accepted");
        assert_eq!(
            fields.first().map(|field| field.key.as_ref()),
            Some("\u{2705}")
        );
        assert_eq!(
            fields.first().map(|field| field.value.as_ref()),
            Some("done")
        );
    }
}
