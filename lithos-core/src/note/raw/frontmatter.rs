//! Raw frontmatter extraction helpers.

use std::{collections::HashMap, fmt::Write as _};

use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone as _, Utc};

use super::super::parser::frontmatter::MetadataBlockKind;
use crate::note::{
    error::FrontmatterParseError,
    frontmatter::{Frontmatter, FrontmatterFormat},
    value::FieldValue,
};

/// Raw frontmatter block captured from metadata events.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawFrontmatter {
    kind: MetadataBlockKind,
    text: Box<str>,
}

impl RawFrontmatter {
    /// Create a raw frontmatter block.
    #[inline]
    #[must_use]
    pub fn new(kind: MetadataBlockKind, text: Box<str>) -> Self {
        Self {
            kind,
            text,
        }
    }

    /// Return the metadata block kind.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> MetadataBlockKind {
        self.kind
    }

    /// Return the raw frontmatter text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl TryFrom<RawFrontmatter> for Frontmatter {
    type Error = FrontmatterParseError;

    #[inline]
    fn try_from(raw: RawFrontmatter) -> Result<Self, Self::Error> {
        let format = match raw.kind() {
            MetadataBlockKind::YamlStyle => FrontmatterFormat::Yaml,
            MetadataBlockKind::PlusesStyle => FrontmatterFormat::Toml,
        };
        parse_frontmatter(format, raw.text())
    }
}

pub(crate) fn parse_frontmatter(
    format: FrontmatterFormat,
    text: &str,
) -> Result<Frontmatter, FrontmatterParseError> {
    let fields = match format {
        FrontmatterFormat::Yaml => {
            let yaml_value: serde_yaml::Value =
                if let Ok(value) = serde_yaml::from_str(text) {
                    value
                } else {
                    let sanitized = sanitize_yaml_obsidian_links(text);
                    serde_yaml::from_str(&sanitized).map_err(|_e| {
                        FrontmatterParseError::InvalidYaml {
                            reason: "failed to parse yaml",
                        }
                    })?
                };
            yaml_to_field_map(&yaml_value)?
        }
        FrontmatterFormat::Toml => {
            let toml_value: toml::Value =
                toml::from_str(text).map_err(|_e| {
                    FrontmatterParseError::InvalidToml {
                        reason: "failed to parse toml",
                    }
                })?;
            toml_to_field_map(&toml_value)?
        }
    };

    Ok(Frontmatter::new(fields))
}

fn yaml_to_field_map(
    value: &serde_yaml::Value,
) -> Result<HashMap<Box<str>, FieldValue>, FrontmatterParseError> {
    let map =
        value.as_mapping().ok_or(FrontmatterParseError::NotYamlMapping)?;

    let mut fields = HashMap::with_capacity(map.len());
    for (key, value_item) in map {
        let key_str =
            key.as_str().ok_or(FrontmatterParseError::NonStringKey)?;

        let field_value =
            FieldValue::try_from_yaml(value_item).map_err(|_error| {
                FrontmatterParseError::InvalidYamlValue {
                    reason: "invalid yaml value",
                }
            })?;

        fields.insert(key_str.into(), field_value);
    }

    Ok(fields)
}

fn toml_to_field_map(
    value: &toml::Value,
) -> Result<HashMap<Box<str>, FieldValue>, FrontmatterParseError> {
    let table = value.as_table().ok_or(FrontmatterParseError::NotTomlTable)?;

    let mut fields = HashMap::with_capacity(table.len());
    for (key, value_item) in table {
        let field_value = field_value_from_toml(value_item)?;
        fields.insert(key.as_str().into(), field_value);
    }

    Ok(fields)
}

fn field_value_from_toml(
    value: &toml::Value,
) -> Result<FieldValue, FrontmatterParseError> {
    if let Some(text) = value.as_str() {
        return Ok(FieldValue::String(text.into()));
    }
    if let Some(number) = value.as_integer() {
        const MAX_SAFE_INTEGER: u64 = 0x0020_0000_0000_0000;
        let magnitude = number.unsigned_abs();
        if magnitude > MAX_SAFE_INTEGER {
            return Err(FrontmatterParseError::InvalidTomlValue {
                reason: "integer value exceeds safe f64 range",
            });
        }

        #[expect(
            clippy::as_conversions,
            reason = "checked MAX_SAFE_INTEGER ensures exact f64"
        )]
        #[expect(
            clippy::cast_precision_loss,
            reason = "checked MAX_SAFE_INTEGER ensures exact f64"
        )]
        let parsed = number as f64;
        return Ok(FieldValue::Number(parsed));
    }
    if let Some(number) = value.as_float() {
        return Ok(FieldValue::Number(number));
    }
    if let Some(value) = value.as_bool() {
        return Ok(FieldValue::Boolean(value));
    }
    if let Some(datetime) = value.as_datetime() {
        let timestamp = toml_datetime_to_timestamp(datetime)?;
        return Ok(FieldValue::Date(timestamp));
    }
    if let Some(values) = value.as_array() {
        let mut items = Vec::with_capacity(values.len());
        for item in values {
            items.push(field_value_from_toml(item)?);
        }
        return Ok(FieldValue::Array(items));
    }
    if let Some(table) = value.as_table() {
        let mut obj = HashMap::with_capacity(table.len());
        for (key, value_item) in table {
            obj.insert(key.as_str().into(), field_value_from_toml(value_item)?);
        }
        return Ok(FieldValue::Object(obj));
    }

    Err(FrontmatterParseError::InvalidTomlValue {
        reason: "unsupported toml value",
    })
}

fn toml_datetime_to_timestamp(
    datetime: &toml::value::Datetime,
) -> Result<i64, FrontmatterParseError> {
    let mut rendered = String::new();
    write!(&mut rendered, "{datetime}").map_err(|_error| {
        FrontmatterParseError::InvalidTomlValue {
            reason: "failed to format datetime",
        }
    })?;
    if let Ok(parsed) = DateTime::parse_from_rfc3339(&rendered) {
        return Ok(parsed.timestamp());
    }
    if let Ok(naive) =
        NaiveDateTime::parse_from_str(&rendered, "%Y-%m-%dT%H:%M:%S%.f")
    {
        return Ok(Utc.from_utc_datetime(&naive).timestamp());
    }
    if let Ok(date) = NaiveDate::parse_from_str(&rendered, "%Y-%m-%d") {
        let naive = date.and_hms_opt(0, 0, 0).ok_or(
            FrontmatterParseError::InvalidTomlValue {
                reason: "invalid date components",
            },
        )?;
        return Ok(Utc.from_utc_datetime(&naive).timestamp());
    }
    Err(FrontmatterParseError::InvalidTomlValue {
        reason: "invalid toml datetime",
    })
}

fn sanitize_yaml_obsidian_links(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let line_end = line.trim_end_matches(['\n', '\r']);
        let line_ending = line.get(line_end.len()..).unwrap_or("");
        let trimmed = line_end.trim_start();
        let indent_len = line_end.len().saturating_sub(trimmed.len());
        let indent = line_end.get(..indent_len).unwrap_or("");

        if let Some(updated) = sanitize_yaml_list_item(trimmed, indent) {
            output.push_str(&updated);
            output.push_str(line_ending);
            continue;
        }

        if let Some(updated) = sanitize_yaml_mapping_entry(trimmed, indent) {
            output.push_str(&updated);
            output.push_str(line_ending);
            continue;
        }

        output.push_str(line_end);
        output.push_str(line_ending);
    }
    output
}

fn sanitize_yaml_list_item(line: &str, indent: &str) -> Option<String> {
    let rest = line.strip_prefix('-')?.trim_start();
    if !is_unquoted_obsidian_link(rest) {
        return None;
    }
    let mut updated = String::with_capacity(
        indent.len().saturating_add(rest.len()).saturating_add(4),
    );
    updated.push_str(indent);
    updated.push_str("- ");
    updated.push('"');
    updated.push_str(rest);
    updated.push('"');
    Some(updated)
}

fn sanitize_yaml_mapping_entry(line: &str, indent: &str) -> Option<String> {
    let colon_index = line.find(':')?;
    let split_index = colon_index.saturating_add(1);
    let (key, rest) = line.split_at(split_index);
    let value = rest.trim_start();
    if value.is_empty() || value.starts_with('|') || value.starts_with('>') {
        return None;
    }
    if !is_unquoted_obsidian_link(value) {
        return None;
    }
    let whitespace_len = rest.len().saturating_sub(value.len());
    let whitespace = rest.get(..whitespace_len).unwrap_or("");
    let mut updated = String::with_capacity(
        indent
            .len()
            .saturating_add(key.len())
            .saturating_add(whitespace.len())
            .saturating_add(value.len())
            .saturating_add(2),
    );
    updated.push_str(indent);
    updated.push_str(key);
    updated.push_str(whitespace);
    updated.push('"');
    updated.push_str(value);
    updated.push('"');
    Some(updated)
}

fn is_unquoted_obsidian_link(value: &str) -> bool {
    if value.starts_with('"') || value.starts_with('\'') {
        return false;
    }
    value.starts_with("[[") || value.starts_with("![[")
}
