use super::tags::add_tag;
use crate::{
    config::aggregate::Config,
    note::{
        frontmatter::Frontmatter, link::FrontmatterLink, tag::Tag as NoteTag,
        value::FieldValue,
    },
};

pub(super) fn collect_frontmatter_tags(
    frontmatter: &Frontmatter,
    config: &Config,
    tags: &mut Vec<NoteTag>,
) {
    let key = config.frontmatter().tags();
    let Some(value) = frontmatter.get(key) else {
        return;
    };

    let mut collect_tokens = |text: &str| {
        for token in text.split(|ch: char| ch.is_whitespace() || ch == ',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            if let Ok(tag) = NoteTag::try_from_token(token) {
                add_tag(tags, tag);
            }
        }
    };

    if let Some(text) = value.as_str() {
        collect_tokens(text);
        return;
    }

    if let Some(values) = value.as_array() {
        for item in values {
            if let Some(text) = item.as_str() {
                collect_tokens(text);
            }
        }
    }
}

pub(super) fn collect_frontmatter_links(
    frontmatter: &Frontmatter,
    links: &mut Vec<FrontmatterLink>,
) {
    for (key, value) in frontmatter.fields() {
        collect_frontmatter_links_for_value(key, value, links);
    }
}

fn collect_frontmatter_links_for_value(
    key: &str,
    value: &FieldValue,
    links: &mut Vec<FrontmatterLink>,
) {
    if let Some(text) = value.as_str() {
        if let Ok(Some(link)) =
            crate::note::link::parse_frontmatter_link(key, text)
        {
            links.push(link);
        }
        return;
    }

    if let Some(values) = value.as_array() {
        for item in values {
            if let Some(text) = array_as_wikilink(item)
                && let Ok(Some(link)) =
                    crate::note::link::parse_frontmatter_link(key, &text)
            {
                links.push(link);
                continue;
            }
            collect_frontmatter_links_for_value(key, item, links);
        }
        return;
    }

    if let Some(values) = value.object_fields() {
        for (child_key, child_value) in values {
            let child_key_str: &str = child_key;
            let mut combined = String::with_capacity(
                key.len().saturating_add(child_key_str.len()).saturating_add(1),
            );
            combined.push_str(key);
            combined.push('.');
            combined.push_str(child_key_str);
            collect_frontmatter_links_for_value(&combined, child_value, links);
        }
    }
}

fn array_as_wikilink(value: &FieldValue) -> Option<String> {
    let outer = value.as_array()?;
    if outer.len() != 1 {
        return None;
    }
    if let Some(text) = outer.first().and_then(FieldValue::as_str) {
        return Some(wrap_wikilink_text(text));
    }
    let inner = outer.first()?.as_array()?;
    if inner.len() != 1 {
        return None;
    }
    let text = inner.first().and_then(FieldValue::as_str)?;
    Some(wrap_wikilink_text(text))
}

fn wrap_wikilink_text(text: &str) -> String {
    let mut combined = String::with_capacity(text.len().saturating_add(4));
    combined.push_str("[[");
    combined.push_str(text);
    combined.push_str("]]");
    combined
}
