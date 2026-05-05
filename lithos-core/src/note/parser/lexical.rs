use super::text::{TextContext, TextSequence};
use crate::note::{
    error::NoteError,
    position::SourceByteRangeIndex,
    scanner::{NoteScanner, ScannedRawArtifacts},
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
    index
}

pub(crate) fn scan_projection<'source>(
    scanner: &NoteScanner,
    source: &'source str,
    projection: &TextSequence,
    policy: &dyn ScanPolicy,
) -> Result<ScannedRawArtifacts<'source>, NoteError> {
    let index = build_scan_index(projection, policy);
    scanner.scan_ranges(source, &index)
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
}
