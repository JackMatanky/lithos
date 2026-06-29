//! Discovered candidate configuration file paths.

use traces_fs::{DirPath, FilePath};

/// A validated candidate config path and the base directory it was found from.
///
/// Both `base` and `path` are filesystem-validated at construction.
/// The `base` directory is the starting point used to resolve `path`
/// during a traversal or global probe pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidatePath {
    /// Base directory used to resolve the candidate.
    base: DirPath,
    /// Candidate config file path.
    path: FilePath,
}

impl CandidatePath {
    /// Creates a validated discovery candidate path.
    #[inline]
    #[must_use]
    pub fn new(base: DirPath, path: FilePath) -> Self {
        Self {
            base,
            path,
        }
    }

    /// Returns the base directory used to resolve this candidate.
    #[inline]
    #[must_use]
    pub fn base(&self) -> &DirPath {
        &self.base
    }

    /// Returns the candidate config file path.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &FilePath {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_base_and_path() {
        let root = tempfile::tempdir().expect("temp dir");
        let file_path = root.path().join("traces.toml");
        std::fs::write(&file_path, "").expect("write");

        let base = DirPath::try_new(root.path().to_path_buf()).unwrap();
        let path = FilePath::try_new(file_path).unwrap();

        let candidate = CandidatePath::new(base.clone(), path.clone());
        assert_eq!(candidate.base(), &base);
        assert_eq!(candidate.path(), &path);
    }
}
