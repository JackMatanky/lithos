//! Raw task token parsing helpers.

type RawInlineField = (Box<str>, Box<str>);

/// Raw inline token collection extracted from task text.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawTaskTokens {
    inline_fields: Vec<RawInlineField>,
    emoji_dates: Vec<RawInlineField>,
}

impl RawTaskTokens {
    /// Create a new token collection.
    #[inline]
    #[must_use]
    pub fn new(
        inline_fields: Vec<RawInlineField>,
        emoji_dates: Vec<RawInlineField>,
    ) -> Self {
        Self {
            inline_fields,
            emoji_dates,
        }
    }

    /// Parse inline fields and emoji dates from text.
    #[must_use]
    pub fn parse(text: &str, emoji_markers: &[char]) -> Self {
        let inline_fields = Self::parse_inline_fields(text);
        let emoji_dates = Self::parse_emoji_dates(text, emoji_markers);
        Self::new(inline_fields, emoji_dates)
    }

    /// Return parsed inline field tokens.
    #[inline]
    #[must_use]
    pub fn inline_fields(&self) -> &[RawInlineField] {
        &self.inline_fields
    }

    /// Return parsed emoji date tokens.
    #[inline]
    #[must_use]
    pub fn emoji_dates(&self) -> &[RawInlineField] {
        &self.emoji_dates
    }

    fn parse_inline_fields(text: &str) -> Vec<RawInlineField> {
        let mut fields = Vec::new();
        Self::for_each_inline_field(text, |key, value| {
            fields.push((key.into(), value.into()));
        });
        fields
    }

    fn for_each_inline_field(text: &str, mut f: impl FnMut(&str, &str)) {
        Self::for_each_inline_field_delim(text, b'[', b']', &mut f);
        Self::for_each_inline_field_delim(text, b'(', b')', &mut f);
    }

    fn for_each_inline_field_delim(
        text: &str,
        open_delim: u8,
        close_delim: u8,
        f: &mut impl FnMut(&str, &str),
    ) {
        let bytes = text.as_bytes();
        let mut cursor = 0;
        while let Some(open_rel) = bytes
            .get(cursor..)
            .and_then(|slice| slice.iter().position(|&b| b == open_delim))
        {
            let open = cursor.saturating_add(open_rel);
            let after_open = open.saturating_add(1);
            let Some(close_rel) = bytes
                .get(after_open..)
                .and_then(|slice| slice.iter().position(|&b| b == close_delim))
            else {
                break;
            };
            let close = after_open.saturating_add(close_rel);
            let Some(inner) = text.get(after_open..close) else {
                break;
            };
            if let Some((key, value)) = inner.split_once("::") {
                let key = key.trim();
                let value = value.trim();
                if !key.is_empty() && !value.is_empty() {
                    f(key, value);
                }
            }
            cursor = close.saturating_add(1);
        }
    }

    fn parse_emoji_dates(
        text: &str,
        emoji_markers: &[char],
    ) -> Vec<RawInlineField> {
        if emoji_markers.is_empty() {
            return Vec::new();
        }
        let mut tokens = Vec::new();
        for (idx, ch) in text.char_indices() {
            if !emoji_markers.contains(&ch) {
                continue;
            }
            let value_start = idx.saturating_add(ch.len_utf8());
            let Some(tail) = text.get(value_start..) else {
                continue;
            };
            let Some(value) = tail.split_whitespace().next() else {
                continue;
            };
            if value.is_empty() {
                continue;
            }
            let mut buffer = [0u8; 4];
            let encoded = ch.encode_utf8(&mut buffer);
            tokens.push((encoded.into(), value.into()));
        }
        tokens
    }
}
