//! File format detection and classification.

use std::ffi::OsStr;

use rkyv::{Archive, Deserialize, Serialize};

/// Supported file formats for structured parsing and classification.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub enum FileFormat {
    /// JSON format.
    Json,
    /// TOML format.
    Toml,
    /// YAML format.
    Yaml,
    /// Markdown format.
    Markdown,
    /// Image format (png, jpg, jpeg, gif, webp, svg, bmp, ico).
    Image,
    /// PDF format.
    Pdf,
    /// Text-based document format (doc, docx, odt, rtf, txt).
    Document,
    /// Archive format (zip, tar, gz, rar, 7z, wasm).
    Archive,
    /// Fallback for other binary formats.
    Binary,
    /// Unknown or unrecognized format.
    Unknown,
}

impl FileFormat {
    /// Detect format from file extension.
    #[inline]
    #[must_use]
    pub fn from_extension(ext: &OsStr) -> Self {
        let ext_str = ext.to_str().unwrap_or_default().to_ascii_lowercase();
        match ext_str.as_str() {
            "json" => Self::Json,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "md" | "markdown" => Self::Markdown,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" => {
                Self::Image
            }
            "pdf" => Self::Pdf,
            "doc" | "docx" | "odt" | "rtf" | "txt" => Self::Document,
            "zip" | "tar" | "gz" | "rar" | "7z" | "wasm" => Self::Archive,
            _ => Self::Unknown,
        }
    }

    /// Returns `true` if this is a markdown format.
    #[inline]
    #[must_use]
    pub fn is_markdown(&self) -> bool {
        matches!(self, Self::Markdown)
    }

    /// Returns `true` if this is a structured data format.
    #[inline]
    #[must_use]
    pub fn is_structured(&self) -> bool {
        matches!(self, Self::Json | Self::Toml | Self::Yaml)
    }
}

/// Borrowed extension view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileExtensionRef<'a>(pub(crate) &'a OsStr);

impl FileExtensionRef<'_> {
    /// Get the format for this extension.
    #[inline]
    #[must_use]
    pub fn format(&self) -> FileFormat {
        FileFormat::from_extension(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_detect_various_formats() {
        assert_eq!(
            FileFormat::from_extension(OsStr::new("md")),
            FileFormat::Markdown
        );
        assert_eq!(
            FileFormat::from_extension(OsStr::new("PNG")),
            FileFormat::Image
        );
        assert_eq!(
            FileFormat::from_extension(OsStr::new("pdf")),
            FileFormat::Pdf
        );
        assert_eq!(
            FileFormat::from_extension(OsStr::new("txt")),
            FileFormat::Document
        );
        assert_eq!(
            FileFormat::from_extension(OsStr::new("zip")),
            FileFormat::Archive
        );
        assert_eq!(
            FileFormat::from_extension(OsStr::new("unknown")),
            FileFormat::Unknown
        );
    }

    #[test]
    fn should_identify_structured_formats() {
        assert!(FileFormat::Json.is_structured());
        assert!(FileFormat::Toml.is_structured());
        assert!(FileFormat::Yaml.is_structured());
        assert!(!FileFormat::Markdown.is_structured());
        assert!(!FileFormat::Image.is_structured());
    }
}
