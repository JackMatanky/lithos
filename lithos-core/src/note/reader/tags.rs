use crate::note::tag::{Tag as NoteTag, scan_tags};

pub(super) fn add_tag(tags: &mut Vec<NoteTag>, tag: NoteTag) {
    if !tags.iter().any(|existing| existing.full_path() == tag.full_path()) {
        tags.push(tag);
    }
}

pub(super) fn collect_tags(
    text: &str,
    inside_code_block: bool,
    inside_link: bool,
    tags: &mut Vec<NoteTag>,
) {
    if inside_code_block || inside_link {
        return;
    }
    for tag in scan_tags(text) {
        add_tag(tags, tag);
    }
}
