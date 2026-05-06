//! File property validation constraints.

use std::path::{Component, Path};

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::{
    error::SchemaError, identifier::SchemaName, raw::file::RawFileProperty,
};

/// File property validation constraints.
///
/// # Examples
/// ```
/// use lithos_core::schema::property_spec::FileSpec;
///
/// let spec = FileSpec::try_new(None, None)?;
/// let _ = spec;
/// # Ok::<_, lithos_core::schema::error::SchemaError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Hash, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct FileSpec {
    directory: Option<VaultRelPath>,
    file_class: Option<Box<str>>,
}

impl FileSpec {
    /// Create a validated `FileSpec`.
    ///
    /// # Errors
    /// Returns `SchemaError` if the directory path is not vault-relative or if
    /// `file_class` is present but empty.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::property_spec::FileSpec;
    ///
    /// let spec = FileSpec::try_new(Some("attachments"), None)?;
    /// let _ = spec;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn try_new(
        directory: Option<&str>,
        file_class: Option<&str>,
    ) -> Result<Self, SchemaError> {
        let directory = match directory {
            Some(dir) => Some(VaultRelPath::try_new(dir)?),
            None => None,
        };

        if let Some(fc) = file_class
            && fc.is_empty()
        {
            return Err(SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::InvalidFileClass {
                    class: "".into(),
                },
            ));
        }

        Ok(Self {
            directory,
            file_class: file_class.map(Into::into),
        })
    }

    #[inline]
    pub(super) fn validate_str(&self, value: &str) -> Result<(), SchemaError> {
        VaultRelPath::validate_path(value)?;

        if let Some(dir) = self.directory.as_ref() {
            let value_path = Path::new(value);
            let dir_path = Path::new(dir.as_str());

            // File must be INSIDE directory, not AT directory level
            if !value_path.starts_with(dir_path) || value_path == dir_path {
                return Err(SchemaError::PropertySpec(
                    crate::schema::error::PropertySpecError::InvalidDirectoryPath {
                        path: format!(
                            "File {value} must be inside (not at) directory {}",
                            dir.as_str()
                        )
                        .into(),
                    },
                ));
            }
        }

        Ok(())
    }

    /// Apply overrides from a raw file spec.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{
    ///     property_spec::FileSpec, raw::file::RawFileSpec,
    /// };
    ///
    /// let base = FileSpec::try_new(None, None)?;
    /// let overrides = RawFileSpec::default();
    /// let _updated = base.apply_overrides(&overrides)?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    ///
    /// Fields that are `None` in the overrides preserve the base values.
    ///
    /// # Errors
    /// Returns `SchemaError` if override values are invalid.
    #[inline]
    pub fn apply_overrides(
        self,
        overrides: &crate::schema::raw::file::RawFileSpec,
    ) -> Result<Self, SchemaError> {
        let directory = overrides
            .directory
            .as_deref()
            .or_else(|| self.directory.as_ref().map(VaultRelPath::as_str));
        let file_class = overrides
            .file_class
            .as_ref()
            .map(SchemaName::as_str)
            .or(self.file_class.as_deref());
        Self::try_new(directory, file_class)
    }
}

impl ArchivedFileSpec {
    /// Validates a file path against directory and file class constraints
    /// directly from the database without deserialization.
    ///
    /// This is a zero-copy validation method that operates on the archived
    /// representation.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validate(&self, value: &str) -> Result<(), SchemaError> {
        VaultRelPath::validate_path(value)?;

        if let Some(dir) = self.directory.as_ref() {
            let value_path = Path::new(value);
            let dir_path = Path::new(dir.0.as_ref());

            // File must be INSIDE directory, not AT directory level
            if !value_path.starts_with(dir_path) || value_path == dir_path {
                return Err(SchemaError::PropertySpec(
                    crate::schema::error::PropertySpecError::InvalidDirectoryPath {
                        path: format!(
                            "File {value} must be inside (not at) directory {}",
                            dir.0.as_ref()
                        )
                        .into(),
                    },
                ));
            }
        }

        Ok(())
    }
}

impl TryFrom<crate::schema::raw::file::RawFileSpec> for FileSpec {
    type Error = SchemaError;

    #[inline]
    fn try_from(
        raw: crate::schema::raw::file::RawFileSpec,
    ) -> Result<Self, Self::Error> {
        let file_class = raw.file_class.as_ref().map(SchemaName::as_str);
        Self::try_new(raw.directory.as_deref(), file_class)
    }
}

impl TryFrom<RawFileProperty> for FileSpec {
    type Error = SchemaError;

    #[inline]
    fn try_from(raw: RawFileProperty) -> Result<Self, Self::Error> {
        let raw_spec = crate::schema::raw::file::RawFileSpec {
            directory: raw.directory,
            file_class: raw.file_class,
        };
        raw_spec.try_into()
    }
}

/// Vault-relative path (no traversal, no absolute paths).
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
struct VaultRelPath(Box<str>);

impl VaultRelPath {
    #[inline]
    fn try_new(path: &str) -> Result<Self, SchemaError> {
        Self::validate_path(path)?;
        Ok(Self(path.into()))
    }

    #[inline]
    fn as_str(&self) -> &str {
        &self.0
    }

    #[inline]
    fn validate_path(path: &str) -> Result<(), SchemaError> {
        if path.is_empty() {
            return Err(SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::InvalidDirectoryPath {
                    path: "Path cannot be empty".into(),
                },
            ));
        }

        for component in Path::new(path).components() {
            match component {
                Component::Normal(_) => {}
                Component::CurDir => {
                    return Err(SchemaError::PropertySpec(
                        crate::schema::error::PropertySpecError::InvalidDirectoryPath {
                            path: format!(
                                "Invalid path {path}: '.' component is not \
                                 allowed"
                            )
                            .into(),
                        },
                    ));
                }
                Component::ParentDir => {
                    return Err(SchemaError::PropertySpec(
                        crate::schema::error::PropertySpecError::InvalidDirectoryPath {
                            path: format!(
                                "Invalid path {path}: '..' component is not \
                                 allowed"
                            )
                            .into(),
                        },
                    ));
                }
                Component::RootDir => {
                    return Err(SchemaError::PropertySpec(
                        crate::schema::error::PropertySpecError::InvalidDirectoryPath {
                            path: format!(
                                "Invalid path {path}: absolute paths are not \
                                 allowed"
                            )
                            .into(),
                        },
                    ));
                }
                Component::Prefix(_) => {
                    return Err(SchemaError::PropertySpec(
                        crate::schema::error::PropertySpecError::InvalidDirectoryPath {
                            path: format!(
                                "Invalid path {path}: path prefixes are not \
                                 allowed"
                            )
                            .into(),
                        },
                    ));
                }
            }
        }

        Ok(())
    }
}

impl TryFrom<Box<str>> for VaultRelPath {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        Self::validate_path(&value)?;
        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn validated_spec_with_dir(dir: &str) -> FileSpec {
        FileSpec::try_new(Some(dir), None).expect("Expected valid FileSpec")
    }

    /// 3.3-UNIT-013: File Specification Validation Matrix.
    /// Priority: P1.
    #[rstest]
    #[case::in_dir("notes/my_note.md", "notes/", Ok(()))]
    #[case::out_dir(
        "other/note.md",
        "notes/",
        Err(SchemaError::PropertySpec(
            crate::schema::error::PropertySpecError::InvalidDirectoryPath {
                path: "File other/note.md must be inside (not at) directory notes/".into(),
            },
        ))
    )]
    fn file_spec_validation_matrix(
        #[case] path: &str,
        #[case] dir: &str,
        #[case] expected: Result<(), SchemaError>,
    ) {
        let spec = validated_spec_with_dir(dir);

        // WHEN: validating file paths
        let result = spec.validate_str(path);

        // THEN: the result matches the expectation
        assert_eq!(
            result, expected,
            "FileSpec validation result should match expected for path: {path}"
        );
    }

    #[test]
    fn file_spec_rejects_prefix_bypass() {
        let spec = validated_spec_with_dir("notes/");

        let result = spec.validate_str("notes_evil/note.md");
        assert!(matches!(
            result,
            Err(SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::InvalidDirectoryPath { .. }
            ))
        ));
    }

    #[test]
    fn file_spec_rejects_parent_dir_traversal() {
        let spec = validated_spec_with_dir("notes/");

        let result = spec.validate_str("../notes/note.md");
        assert!(matches!(
            result,
            Err(SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::InvalidDirectoryPath { .. }
            ))
        ));
    }

    #[test]
    fn file_spec_rejects_absolute_paths() {
        let spec = validated_spec_with_dir("notes/");

        let result = spec.validate_str("/notes/note.md");
        assert!(matches!(
            result,
            Err(SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::InvalidDirectoryPath { .. }
            ))
        ));
    }

    #[test]
    fn file_spec_rejects_value_equal_to_directory() {
        let spec = validated_spec_with_dir("notes/");

        let result = spec.validate_str("notes/");
        assert!(matches!(
            result,
            Err(SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::InvalidDirectoryPath { .. }
            ))
        ));
    }

    #[test]
    fn file_spec_rejects_directory_path_without_trailing_slash() {
        // GIVEN: FileSpec with directory "assets"
        let spec = validated_spec_with_dir("assets");

        // WHEN: validating path exactly equal to directory (without slash)
        let result = spec.validate_str("assets");

        // THEN: it should reject (file must be INSIDE directory, not AT
        // directory level)
        assert!(
            matches!(
                result,
                Err(SchemaError::PropertySpec(
                    crate::schema::error::PropertySpecError::InvalidDirectoryPath { .. }
                ))
            ),
            "Expected InvalidDirectoryPath error for exact directory match"
        );
    }

    #[test]
    fn file_spec_validates_file_class_format() {
        // GIVEN: a valid file_class spec
        let result = FileSpec::try_new(None, Some("any-schema-name"));
        assert!(
            result.is_ok(),
            "Expected valid file_class spec, got: {result:?}"
        );
    }

    #[test]
    fn file_spec_rejects_empty_file_class() {
        // GIVEN: an empty file_class spec
        let result = FileSpec::try_new(None, Some(""));

        // THEN: it should be invalid
        assert!(
            result.is_err(),
            "Expected invalid file_class spec, got: {result:?}"
        );
    }
}
