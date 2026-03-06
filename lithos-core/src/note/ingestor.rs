//! Ingestor adapter for loading markdown notes from the filesystem.
//!
//! Pure file discovery. No parsing or DB access.

use crate::{
    config::aggregate::Config,
    fs::{FsReader, types::Markdown},
    note::{error::NoteError, paths::NotePath},
};

/// Ingestor for discovering note files from a filesystem source.
///
/// This adapter is responsible for:
/// - Scanning the vault for markdown note files
///
/// It does NOT:
/// - Assign IDs
/// - Parse markdown content
/// - Query the database or persist projections
///
/// # Examples
/// ```ignore
/// use lithos_core::{
///     config::aggregate::Config,
///     note::ingestor::Ingestor,
/// };
///
/// let config = todo!("Provide a Config");
/// let ingestor = Ingestor::new(&config);
/// let _paths = ingestor.scan_note_paths()?;
/// # Ok::<_, lithos_core::note::error::NoteError>(())
/// ```
pub struct Ingestor<'config> {
    source: FsReader,
    config: &'config Config,
}

impl<'config> Ingestor<'config> {
    /// Create a new note ingestor using the vault root from config.
    #[inline]
    #[must_use]
    pub fn new(config: &'config Config) -> Self {
        Self {
            source: FsReader::new(config.vault_metadata().root().as_path()),
            config,
        }
    }

    /// Scan the vault for markdown note files.
    ///
    /// # Errors
    /// Returns [`NoteError`] if listing or validation fails.
    #[inline]
    pub fn scan_note_paths(&self) -> Result<Vec<NotePath>, NoteError> {
        let pattern = "**/*";
        let files = self
            .source
            .list_files(pattern)
            .map_err(|error| NoteError::Storage(error.to_string().into()))?;

        let mut notes = Vec::with_capacity(files.len());
        for path in files {
            if !Markdown::is_supported(&path) {
                continue;
            }
            if let Err(error) = self.source.validate_path(&path) {
                return Err(NoteError::Storage(error.to_string().into()));
            }
            let path_str = path.to_str().ok_or_else(|| {
                NoteError::Storage("invalid UTF-8 in note path".into())
            })?;
            let note_path = NotePath::try_new(path_str).map_err(|error| {
                NoteError::Storage(error.to_string().into())
            })?;
            notes.push(note_path);
        }

        Ok(notes)
    }

    #[inline]
    #[must_use]
    /// Access the configuration used for vault discovery.
    pub const fn config(&self) -> &'config Config {
        self.config
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Assertions are used to fail tests"
)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::config::{
        aggregate::Config,
        raw::RawConfig,
        vault::{VaultId, VaultRoot},
    };

    fn write_file(root: &std::path::Path, relative: &str, contents: &str) {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create dirs");
        }
        std::fs::write(&path, contents).expect("write file");
    }

    fn test_config(root: &std::path::Path) -> Config {
        Config::build(
            &RawConfig::default(),
            VaultId::new(),
            VaultRoot::try_new(root.to_path_buf()).expect("vault root"),
            crate::config::aggregate::Version::initial(),
        )
        .expect("config")
    }

    #[test]
    fn scan_note_paths_returns_note_paths() -> Result<(), NoteError> {
        let dir = TempDir::new()
            .map_err(|error| NoteError::Storage(error.to_string().into()))?;
        write_file(dir.path(), "notes/alpha.md", "# Alpha");
        write_file(dir.path(), "notes/beta.md", "# Beta");

        let config = test_config(dir.path());
        let ingestor = Ingestor::new(&config);

        let paths = ingestor.scan_note_paths()?;
        let mut names: Vec<_> = paths.iter().map(NotePath::as_str).collect();
        names.sort_unstable();

        assert_eq!(names, vec!["notes/alpha.md", "notes/beta.md"]);
        Ok(())
    }
}
