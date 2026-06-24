//! Template aggregate and value object types.
//!
//! Core domain types for the Template context:
//! - [`TemplateId`] — UUID v7-based identifier
//! - [`TemplateName`] — path-derived, subdirectory-qualified template name
//! - [`TemplateBody`] — non-empty renderable source text
//! - [`Template`] — primary renderable aggregate

use std::{fmt, path::Path, time::SystemTime};

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};
use trace_fs::PathKey;
use trace_utils::UuidV7;
use uuid::Uuid;

use super::error::{TemplateBodyError, TemplateNameError};

// ============================================================================
// TemplateId
// ============================================================================

/// Unique identifier for a template.
///
/// Wraps a UUID v7 (time-ordered) identifier for templates, matching the
/// `NoteId`/`SchemaId` pattern.
///
/// # Examples
///
/// ```
/// use trace_template::TemplateId;
///
/// let id = TemplateId::new();
/// let _uuid = id.as_uuid_v7();
/// ```
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq))]
pub struct TemplateId(pub(crate) UuidV7);

impl TemplateId {
    /// Creates a new UUID v7-based `TemplateId`.
    ///
    /// # Examples
    ///
    /// ```
    /// use trace_template::TemplateId;
    ///
    /// let id = TemplateId::new();
    /// let _ = id.as_uuid_v7();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(UuidV7::new())
    }

    /// Parses a template identifier from a string.
    ///
    /// # Errors
    ///
    /// Returns [`trace_utils::UuidV7Error`] if parsing fails or the UUID is
    /// not v7.
    ///
    /// # Examples
    ///
    /// ```
    /// use trace_template::TemplateId;
    ///
    /// let id = TemplateId::new();
    /// let s = id.to_string();
    /// let parsed = TemplateId::parse(&s).unwrap();
    /// assert_eq!(id, parsed);
    /// ```
    #[inline]
    pub fn parse(id: &str) -> Result<Self, trace_utils::UuidV7Error> {
        Ok(Self(UuidV7::parse(id)?))
    }

    /// Returns a reference to the inner [`UuidV7`].
    #[inline]
    #[must_use]
    pub const fn as_uuid_v7(&self) -> &UuidV7 {
        &self.0
    }
}

impl Default for TemplateId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TemplateId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<UuidV7> for TemplateId {
    #[inline]
    fn from(value: UuidV7) -> Self {
        Self(value)
    }
}

impl TryFrom<Uuid> for TemplateId {
    type Error = trace_utils::UuidV7Error;

    #[inline]
    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        Ok(Self(UuidV7::try_from(value)?))
    }
}

impl AsRef<UuidV7> for TemplateId {
    #[inline]
    fn as_ref(&self) -> &UuidV7 {
        &self.0
    }
}

// ============================================================================
// TemplateName
// ============================================================================

/// Path-derived, subdirectory-qualified template name.
///
/// Constructed from the file path of a template relative to the configured
/// template directory root. Uses `/` as the separator for subdirectory
/// qualification.
///
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// use trace_template::TemplateName;
///
/// // Flat template: templates/standup.md → "standup"
/// let name = TemplateName::try_new(
///     Path::new("templates/standup.md"),
///     Path::new("templates"),
/// )
/// .unwrap();
/// assert_eq!(name.as_str(), "standup");
///
/// // Nested template: templates/daily/standup.md → "daily/standup"
/// let name = TemplateName::try_new(
///     Path::new("templates/daily/standup.md"),
///     Path::new("templates"),
/// )
/// .unwrap();
/// assert_eq!(name.as_str(), "daily/standup");
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct TemplateName(String);

impl TemplateName {
    /// Constructs a `TemplateName` from a template file path and the template
    /// directory root.
    ///
    /// Strips the root prefix from `file_path`, then derives a
    /// `/`-separated name by joining the parent components and stem.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateNameError::Derivation`] if:
    /// - `file_path` cannot be stripped of `root`
    /// - the resulting path has no file stem
    /// - the derived name is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// use trace_template::TemplateName;
    ///
    /// let name = TemplateName::try_new(
    ///     Path::new("templates/daily/standup.md"),
    ///     Path::new("templates"),
    /// )
    /// .unwrap();
    /// assert_eq!(name.as_str(), "daily/standup");
    /// ```
    #[inline]
    pub fn try_new(
        file_path: &Path,
        root: &Path,
    ) -> Result<Self, TemplateNameError> {
        // Strip the root prefix to get the relative path within the template
        // dir.
        let relative = file_path
            .strip_prefix(root)
            .map_err(|_| TemplateNameError::Derivation)?;

        // Get the file stem (filename without extension).
        let stem = relative
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or(TemplateNameError::Derivation)?;

        if stem.is_empty() {
            return Err(TemplateNameError::Derivation);
        }

        // Build the qualified name: parent components + stem, joined with '/'.
        let parent = relative.parent();
        let name = match parent {
            Some(p) if p.components().next().is_some() => {
                // Has subdirectory components — build qualified name.
                let parent_str =
                    p.to_str().ok_or(TemplateNameError::Derivation)?;
                // Normalize path separators to forward slashes.
                let normalized =
                    parent_str.replace(std::path::MAIN_SEPARATOR, "/");
                format!("{normalized}/{stem}")
            }
            _ => {
                // At root level — use stem directly.
                stem.to_owned()
            }
        };

        if name.is_empty() {
            return Err(TemplateNameError::Derivation);
        }

        Ok(Self(name))
    }

    /// Returns the template name as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// use trace_template::TemplateName;
    ///
    /// let name = TemplateName::try_new(
    ///     Path::new("templates/standup.md"),
    ///     Path::new("templates"),
    /// )
    /// .unwrap();
    /// assert_eq!(name.as_str(), "standup");
    /// ```
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TemplateName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TemplateName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// TemplateBody
// ============================================================================

/// Non-empty renderable source text for a template.
///
/// Structural invariants only: the string must be non-empty and valid UTF-8.
/// Jinja syntax validity is explicitly **not** the domain's responsibility.
///
/// # Examples
///
/// ```
/// use trace_template::TemplateBody;
///
/// let body = TemplateBody::try_new("Hello {{ name }}!").unwrap();
/// assert_eq!(body.as_str(), "Hello {{ name }}!");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct TemplateBody(String);

impl TemplateBody {
    /// Constructs a `TemplateBody`, rejecting empty strings.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateBodyError::Empty`] if `content` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use trace_template::TemplateBody;
    ///
    /// assert!(TemplateBody::try_new("").is_err());
    /// assert!(TemplateBody::try_new("hello").is_ok());
    /// ```
    #[inline]
    pub fn try_new<S: Into<String>>(
        content: S,
    ) -> Result<Self, TemplateBodyError> {
        let content = content.into();
        if content.is_empty() {
            return Err(TemplateBodyError::Empty);
        }
        Ok(Self(content))
    }

    /// Returns the template body as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use trace_template::TemplateBody;
    ///
    /// let body = TemplateBody::try_new("Hello!").unwrap();
    /// assert_eq!(body.as_str(), "Hello!");
    /// ```
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TemplateBody {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// Template
// ============================================================================

/// Primary renderable template aggregate.
///
/// Combines identity, path, derived name, body, and an ingestion timestamp.
/// The `recorded_at` field is set internally at construction — callers cannot
/// pass an inconsistent timestamp.
///
/// Marked `#[non_exhaustive]` to allow future frontmatter/query evolution
/// without a breaking change.
///
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// use trace_fs::PathKey;
/// use trace_template::{Template, TemplateBody, TemplateId, TemplateName};
///
/// let id = TemplateId::new();
/// let path = PathKey::try_new("templates/standup.md").unwrap();
/// let name = TemplateName::try_new(
///     Path::new("templates/standup.md"),
///     Path::new("templates"),
/// )
/// .unwrap();
/// let body = TemplateBody::try_new("# {{ title }}").unwrap();
/// let template = Template::new(id, path, name, body);
/// assert_eq!(template.id(), &id);
/// ```
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Template {
    /// Template identity.
    id: TemplateId,
    /// Vault-relative storage path.
    path: PathKey,
    /// Derived template name.
    name: TemplateName,
    /// Renderable source body.
    body: TemplateBody,
    /// Ingestion timestamp (private — not part of the public API).
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl Template {
    /// Creates a new `Template`, setting `recorded_at` to `SystemTime::now()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// use trace_fs::PathKey;
    /// use trace_template::{Template, TemplateBody, TemplateId, TemplateName};
    ///
    /// let id = TemplateId::new();
    /// let path = PathKey::try_new("templates/standup.md").unwrap();
    /// let name = TemplateName::try_new(
    ///     Path::new("templates/standup.md"),
    ///     Path::new("templates"),
    /// )
    /// .unwrap();
    /// let body = TemplateBody::try_new("# {{ title }}").unwrap();
    /// let t = Template::new(id, path, name, body);
    /// assert_eq!(t.id(), &id);
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        id: TemplateId,
        path: PathKey,
        name: TemplateName,
        body: TemplateBody,
    ) -> Self {
        Self {
            id,
            path,
            name,
            body,
            recorded_at: SystemTime::now(),
        }
    }

    /// Returns the template identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> &TemplateId {
        &self.id
    }

    /// Returns the vault-relative storage path.
    #[inline]
    #[must_use]
    pub const fn path(&self) -> &PathKey {
        &self.path
    }

    /// Returns the derived template name.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &TemplateName {
        &self.name
    }

    /// Returns the renderable source body.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &TemplateBody {
        &self.body
    }

    /// Returns the ingestion timestamp.
    #[inline]
    #[must_use]
    pub const fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    mod template_id {
        use super::*;

        mod constructor {
            use pretty_assertions::{assert_eq, assert_ne};

            use super::*;

            #[test]
            fn new_generates_unique_ids() {
                let id1 = TemplateId::new();
                let id2 = TemplateId::new();
                assert_ne!(
                    id1, id2,
                    "Two new() calls should produce different ids"
                );
            }

            #[test]
            fn new_returns_uuid_v7() {
                let id = TemplateId::new();
                assert_eq!(
                    id.as_uuid_v7().as_uuid().get_version(),
                    Some(uuid::Version::SortRand),
                    "new() should produce a UUID v7"
                );
            }

            #[test]
            fn as_uuid_v7_returns_inner_reference() {
                let id = TemplateId::new();
                let uuid_ref = id.as_uuid_v7();
                assert_eq!(
                    uuid_ref.as_uuid().get_version(),
                    Some(uuid::Version::SortRand)
                );
            }
        }

        mod defaults {
            use pretty_assertions::{assert_eq, assert_ne};

            use super::*;

            #[test]
            fn default_generates_unique_ids() {
                let id1 = TemplateId::default();
                let id2 = TemplateId::default();
                assert_ne!(id1, id2, "default() should generate unique ids");
            }

            #[test]
            fn default_is_v7() {
                let id = TemplateId::default();
                assert_eq!(
                    id.as_uuid_v7().as_uuid().get_version(),
                    Some(uuid::Version::SortRand)
                );
            }
        }

        mod formatting {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn display_is_parseable_back() {
                let id = TemplateId::new();
                let s = id.to_string();
                let parsed = TemplateId::parse(&s).unwrap();
                assert_eq!(id, parsed, "Display then parse should round-trip");
            }

            #[test]
            fn display_format_is_uuid_string() {
                let id = TemplateId::new();
                let s = id.to_string();
                // UUID string has standard dashes and 36 chars
                assert_eq!(s.len(), 36, "UUID display should be 36 chars");
                assert!(s.contains('-'), "UUID display should contain dashes");
            }
        }

        mod conversions {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn from_uuid_v7_constructs_correctly() {
                let inner = UuidV7::new();
                let id = TemplateId::from(inner);
                assert_eq!(id.as_uuid_v7(), &inner);
            }

            #[test]
            fn try_from_uuid_rejects_non_v7() {
                // Uuid::new_v5 is available without extra feature flags.
                let non_v7 = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"traces");
                let result = TemplateId::try_from(non_v7);
                assert!(result.is_err(), "Non-v7 UUID should be rejected");
            }

            #[test]
            fn try_from_uuid_accepts_v7() {
                let v7 = uuid::Uuid::now_v7();
                let result = TemplateId::try_from(v7);
                assert!(result.is_ok(), "v7 UUID should be accepted");
            }

            #[test]
            fn as_ref_returns_inner_uuid_v7() {
                let id = TemplateId::new();
                let r: &UuidV7 = id.as_ref();
                assert_eq!(r, id.as_uuid_v7());
            }
        }

        mod serialization {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn rkyv_round_trip() {
                let id = TemplateId::new();
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&id)
                    .expect("Failed to serialize TemplateId");
                let deserialized: TemplateId =
                    rkyv::from_bytes::<TemplateId, rkyv::rancor::Error>(&bytes)
                        .expect("Failed to deserialize TemplateId");
                assert_eq!(
                    id, deserialized,
                    "rkyv round-trip should preserve identity"
                );
            }
        }
    }

    mod template_name {
        use super::*;

        mod constructor {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn flat_md_at_root_derives_stem() {
                let name = TemplateName::try_new(
                    Path::new("templates/standup.md"),
                    Path::new("templates"),
                )
                .unwrap();
                assert_eq!(name.as_str(), "standup");
            }

            #[test]
            fn nested_subdirectory_derives_qualified_name() {
                let name = TemplateName::try_new(
                    Path::new("templates/daily/standup.md"),
                    Path::new("templates"),
                )
                .unwrap();
                assert_eq!(name.as_str(), "daily/standup");
            }

            #[test]
            fn deeply_nested_derives_full_path() {
                let name = TemplateName::try_new(
                    Path::new("templates/work/daily/standup.md"),
                    Path::new("templates"),
                )
                .unwrap();
                assert_eq!(name.as_str(), "work/daily/standup");
            }

            #[test]
            fn as_str_returns_inner_string() {
                let name = TemplateName::try_new(
                    Path::new("templates/note.md"),
                    Path::new("templates"),
                )
                .unwrap();
                assert_eq!(name.as_str(), "note");
            }

            #[test]
            fn display_matches_as_str() {
                let name = TemplateName::try_new(
                    Path::new("templates/note.md"),
                    Path::new("templates"),
                )
                .unwrap();
                assert_eq!(name.to_string(), name.as_str());
            }

            #[test]
            fn as_ref_str_matches_as_str() {
                let name = TemplateName::try_new(
                    Path::new("templates/note.md"),
                    Path::new("templates"),
                )
                .unwrap();
                let s: &str = name.as_ref();
                assert_eq!(s, name.as_str());
            }
        }

        mod validation {
            use super::*;

            #[test]
            fn path_not_under_root_returns_derivation_error() {
                let result = TemplateName::try_new(
                    Path::new("other/note.md"),
                    Path::new("templates"),
                );
                assert!(
                    matches!(result, Err(TemplateNameError::Derivation)),
                    "Path not under root should return Derivation error"
                );
            }

            #[test]
            fn path_without_stem_returns_derivation_error() {
                // A path like "templates/.md" has no meaningful stem (stem
                // would be ".md" on some platforms)
                // Use a directory-only path which has no stem.
                let result = TemplateName::try_new(
                    Path::new("templates/"),
                    Path::new("templates"),
                );
                assert!(
                    result.is_err(),
                    "Path with no stem should return error"
                );
            }
        }

        mod serialization {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn rkyv_round_trip() {
                let name = TemplateName::try_new(
                    Path::new("templates/daily/standup.md"),
                    Path::new("templates"),
                )
                .unwrap();
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&name)
                    .expect("Failed to serialize TemplateName");
                let deserialized: TemplateName = rkyv::from_bytes::<
                    TemplateName,
                    rkyv::rancor::Error,
                >(&bytes)
                .expect("Failed to deserialize TemplateName");
                assert_eq!(
                    name, deserialized,
                    "rkyv round-trip should preserve name"
                );
            }
        }
    }

    mod template_body {
        use super::*;

        mod constructor {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn valid_content_constructs_successfully() {
                let body = TemplateBody::try_new("Hello {{ name }}!").unwrap();
                assert_eq!(body.as_str(), "Hello {{ name }}!");
            }

            #[test]
            fn as_str_returns_inner_content() {
                let body = TemplateBody::try_new("content").unwrap();
                assert_eq!(body.as_str(), "content");
            }

            #[test]
            fn as_ref_str_matches_as_str() {
                let body = TemplateBody::try_new("content").unwrap();
                let s: &str = body.as_ref();
                assert_eq!(s, body.as_str());
            }

            #[test]
            fn does_not_validate_jinja_syntax() {
                // Invalid Jinja syntax should still be accepted — not the
                // domain's responsibility.
                let body = TemplateBody::try_new("{{ unclosed").unwrap();
                assert_eq!(body.as_str(), "{{ unclosed");
            }
        }

        mod validation {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn empty_string_returns_empty_error() {
                let result = TemplateBody::try_new("");
                assert!(
                    matches!(result, Err(TemplateBodyError::Empty)),
                    "Empty string should return TemplateBodyError::Empty"
                );
            }

            #[test]
            fn whitespace_only_is_valid() {
                // Whitespace is not empty — structurally valid.
                let body = TemplateBody::try_new("   ").unwrap();
                assert_eq!(body.as_str(), "   ");
            }
        }
    }

    mod template {
        use super::*;

        fn make_test_template() -> Template {
            let id = TemplateId::new();
            let path = PathKey::try_new("templates/standup.md").unwrap();
            let name = TemplateName::try_new(
                Path::new("templates/standup.md"),
                Path::new("templates"),
            )
            .unwrap();
            let body =
                TemplateBody::try_new("# Standup\n{{ content }}").unwrap();
            Template::new(id, path, name, body)
        }

        mod constructor {
            use super::*;

            #[test]
            fn new_sets_recorded_at_internally() {
                let before = SystemTime::now();
                let template = make_test_template();
                let after = SystemTime::now();
                assert!(
                    template.recorded_at() >= before,
                    "recorded_at should be >= before construction"
                );
                assert!(
                    template.recorded_at() <= after,
                    "recorded_at should be <= after construction"
                );
            }
        }

        mod accessors {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn id_accessor_returns_correct_id() {
                let id = TemplateId::new();
                let path = PathKey::try_new("templates/note.md").unwrap();
                let name = TemplateName::try_new(
                    Path::new("templates/note.md"),
                    Path::new("templates"),
                )
                .unwrap();
                let body = TemplateBody::try_new("content").unwrap();
                let template = Template::new(id, path, name, body);
                assert_eq!(template.id(), &id);
            }

            #[test]
            fn path_accessor_returns_correct_path() {
                let id = TemplateId::new();
                let path = PathKey::try_new("templates/note.md").unwrap();
                let name = TemplateName::try_new(
                    Path::new("templates/note.md"),
                    Path::new("templates"),
                )
                .unwrap();
                let body = TemplateBody::try_new("content").unwrap();
                let expected_path = path.clone();
                let template = Template::new(id, path, name, body);
                assert_eq!(template.path(), &expected_path);
            }

            #[test]
            fn name_accessor_returns_correct_name() {
                let template = make_test_template();
                assert_eq!(template.name().as_str(), "standup");
            }

            #[test]
            fn body_accessor_returns_correct_body() {
                let template = make_test_template();
                assert!(template.body().as_str().contains("Standup"));
            }

            #[test]
            fn recorded_at_accessor_returns_system_time() {
                let template = make_test_template();
                // Just verify it's accessible and reasonable (after Unix
                // epoch).
                let epoch = SystemTime::UNIX_EPOCH;
                assert!(
                    template.recorded_at() > epoch,
                    "recorded_at should be after Unix epoch"
                );
            }
        }

        mod serialization {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn rkyv_round_trip() {
                let id = TemplateId::new();
                let path = PathKey::try_new("templates/standup.md").unwrap();
                let name = TemplateName::try_new(
                    Path::new("templates/standup.md"),
                    Path::new("templates"),
                )
                .unwrap();
                let body = TemplateBody::try_new("# {{ title }}").unwrap();
                let template = Template::new(id, path, name, body);

                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&template)
                    .expect("Failed to serialize Template");
                let deserialized: Template =
                    rkyv::from_bytes::<Template, rkyv::rancor::Error>(&bytes)
                        .expect("Failed to deserialize Template");

                assert_eq!(deserialized.id(), template.id());
                assert_eq!(deserialized.path(), template.path());
                assert_eq!(deserialized.name(), template.name());
                assert_eq!(deserialized.body(), template.body());
                // recorded_at is serialized as unix seconds — check
                // second-level precision.
                let orig_secs = template
                    .recorded_at()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let deser_secs = deserialized
                    .recorded_at()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                assert_eq!(
                    orig_secs, deser_secs,
                    "recorded_at unix seconds should round-trip"
                );
            }
        }
    }
}
