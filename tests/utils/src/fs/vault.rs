//! Test Vault utility for integration testing of vault operations.
//!
//! This module provides a high-level `TestVault` struct that handles the
//! creation and management of a Lithos vault in a temporary directory,
//! including standard configurations and helper methods for adding notes and
//! metadata.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::fs::temp::TempDir;

/// A Lithos vault initialized for testing in an isolated temporary directory.
///
/// This utility provides a fluent-style API for scaffolding Obsidian-like vault
/// structures, including standard Lithos metadata files and directories.
///
/// # Examples
///
/// ```rust
/// use std::fs;
///
/// use lithos_test_utils::TestVault;
///
/// # fn main() -> std::io::Result<()> {
/// let vault = TestVault::new()?;
///
/// // Add notes with relative paths
/// let note_path =
///     vault.add_note("Work/Project.md", "# Project\nStatus: Active")?;
/// assert!(note_path.exists());
///
/// // Add raw binary files
/// vault.add_file("Assets/logo.png", &[0x89, 0x50, 0x4E, 0x47])?;
///
/// // Access the vault root path
/// let root = vault.path();
/// assert!(root.join("lithos.toml").exists());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct TestVault {
    #[expect(dead_code)]
    temp_dir: TempDir,
    root: PathBuf,
}

impl TestVault {
    /// Creates and initializes a new test vault in a temporary directory.
    pub fn new() -> std::io::Result<Self> {
        let temp_dir = TempDir::with_prefix("lithos_test_vault")?;
        let root = temp_dir.path().to_path_buf();

        // Initialize standard Lithos structure
        fs::create_dir_all(root.join(".lithos/indices"))?;
        fs::create_dir_all(root.join(".lithos/cache"))?;

        // Create a default lithos.toml
        let config = r#"
[vault]
name = "Test Vault"
version = "0.1.0"
"#;
        fs::write(root.join("lithos.toml"), config)?;

        Ok(Self {
            temp_dir,
            root,
        })
    }

    /// Returns the absolute path to the vault root.
    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Adds a note to the vault at the specified relative path.
    pub fn add_note(
        &self,
        relative_path: impl AsRef<Path>,
        content: &str,
    ) -> std::io::Result<PathBuf> {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
        Ok(path)
    }

    /// Adds a raw file to the vault.
    pub fn add_file(
        &self,
        relative_path: impl AsRef<Path>,
        data: &[u8],
    ) -> std::io::Result<PathBuf> {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, data)?;
        Ok(path)
    }

    /// Returns the path to the .lithos directory.
    pub fn dot_lithos(&self) -> PathBuf {
        self.root.join(".lithos")
    }

    /// Returns a PathBuf for a file relative to the vault root without creating
    /// it.
    pub fn relative_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.root.join(path)
    }
}

#[cfg(test)]
// # LINT_DISABLE_REASON: TestVault initialization and assertions in tests use
// unwrap for conciseness. # LINT_DISABLE_REASON: Options tried: manual Result
// propagation. # LINT_DISABLE_REASON: Justification: standard practice in test
// code.
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn vault_initializes_standard_structure_on_new() {
        let vault = TestVault::new().unwrap();
        assert!(vault.path().exists());
        assert!(vault.path().join("lithos.toml").exists());
        assert!(vault.dot_lithos().exists());
        assert!(vault.dot_lithos().join("indices").exists());
    }

    #[test]
    fn vault_allows_adding_notes_and_raw_files() {
        let vault = TestVault::new().unwrap();
        let note_path = vault.add_note("work/meeting.md", "# Meeting").unwrap();
        assert!(note_path.exists());
        assert_eq!(fs::read_to_string(note_path).unwrap(), "# Meeting");

        let data = vec![0, 1, 2, 3];
        let file_path = vault.add_file("bin/data.bin", &data).unwrap();
        assert!(file_path.exists());
        assert_eq!(fs::read(file_path).unwrap(), data);
    }
}
