use pulldown_cmark::LinkType;

use crate::note::{
    link::{AliasMode, EmbedState, LinkBuilder, Style},
    position::SourceByteOffset,
};

pub(super) fn start_link_builder(
    link_type: LinkType,
    dest_url: &pulldown_cmark::CowStr<'_>,
    position: SourceByteOffset,
    is_embed: bool,
) -> LinkBuilder {
    let embed = if is_embed {
        EmbedState::Embed
    } else {
        EmbedState::Link
    };
    match link_type {
        LinkType::WikiLink {
            has_pothole,
        } => {
            let alias_mode = if has_pothole {
                AliasMode::Collect
            } else {
                AliasMode::Ignore
            };
            LinkBuilder::new(
                dest_url.as_ref(),
                position,
                Style::WikiLink,
                embed,
                alias_mode,
            )
        }
        LinkType::Autolink | LinkType::Email => LinkBuilder::new(
            dest_url.as_ref(),
            position,
            Style::MdLink,
            embed,
            AliasMode::Ignore,
        ),
        LinkType::Inline
        | LinkType::Reference
        | LinkType::ReferenceUnknown
        | LinkType::Collapsed
        | LinkType::CollapsedUnknown
        | LinkType::Shortcut
        | LinkType::ShortcutUnknown => LinkBuilder::new(
            dest_url.as_ref(),
            position,
            Style::MdLink,
            embed,
            AliasMode::Collect,
        ),
    }
}
