//! Boundary data returned by the redesigned discovery service.

use crate::fs::{DirPath, FilePath};

/// A validated candidate config path and the base directory it was found from.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code, reason = "Contract slice; wired into discovery later")]
pub(crate) struct CandidatePath {
    /// Base directory used to resolve the candidate.
    base: DirPath,
    /// Candidate config file path.
    path: FilePath,
}

#[allow(dead_code, reason = "Contract slice; wired into discovery later")]
impl CandidatePath {
    /// Creates a validated discovery candidate path.
    #[inline]
    #[must_use]
    pub(crate) fn new(base: DirPath, path: FilePath) -> Self {
        Self {
            base,
            path,
        }
    }

    /// Returns the base directory used to resolve this candidate.
    #[inline]
    #[must_use]
    pub(crate) fn base(&self) -> &DirPath {
        &self.base
    }

    /// Returns the candidate config file path.
    #[inline]
    #[must_use]
    pub(crate) fn path(&self) -> &FilePath {
        &self.path
    }
}

/// Pure discovery output consumed by downstream configuration loading.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code, reason = "Contract slice; wired into discovery later")]
pub(crate) struct DiscoveryResult {
    /// Ordered vault-local candidates.
    vault: Vec<CandidatePath>,
    /// Ordered global candidates.
    global: Vec<CandidatePath>,
}

#[allow(dead_code, reason = "Contract slice; wired into discovery later")]
impl DiscoveryResult {
    /// Creates discovery output from ordered vault and global candidates.
    #[inline]
    #[must_use]
    pub(crate) fn new(
        vault: Vec<CandidatePath>,
        global: Vec<CandidatePath>,
    ) -> Self {
        Self {
            vault,
            global,
        }
    }

    /// Returns ordered vault-local candidates.
    #[inline]
    #[must_use]
    pub(crate) fn vault(&self) -> &[CandidatePath] {
        &self.vault
    }

    /// Returns ordered global candidates.
    #[inline]
    #[must_use]
    pub(crate) fn global(&self) -> &[CandidatePath] {
        &self.global
    }

    /// Consumes the result into owned candidate vectors.
    #[inline]
    #[must_use]
    pub(crate) fn into_parts(self) -> (Vec<CandidatePath>, Vec<CandidatePath>) {
        (self.vault, self.global)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::{DirPath, FilePath};

    fn candidate(root: &tempfile::TempDir, name: &str) -> CandidatePath {
        let path = root.path().join(name);
        std::fs::write(&path, "").expect("write candidate file");

        CandidatePath::new(
            DirPath::try_new(root.path().to_path_buf())
                .expect("valid base dir"),
            FilePath::try_new(path).expect("valid candidate file"),
        )
    }

    mod discovery_result {
        use super::*;

        #[test]
        fn keeps_vault_and_global_candidates_separate() {
            let vault_root = tempfile::tempdir().expect("vault root");
            let global_root = tempfile::tempdir().expect("global root");
            let vault = candidate(&vault_root, "lithos.toml");
            let global = candidate(&global_root, "lithos.toml");

            let result =
                DiscoveryResult::new(vec![vault.clone()], vec![global.clone()]);

            assert_eq!(result.vault(), [vault]);
            assert_eq!(result.global(), [global]);
        }

        #[test]
        fn consumes_result_into_candidate_vectors() {
            let vault_root = tempfile::tempdir().expect("vault root");
            let global_root = tempfile::tempdir().expect("global root");
            let vault = candidate(&vault_root, "lithos.toml");
            let global = candidate(&global_root, "lithos.toml");
            let result =
                DiscoveryResult::new(vec![vault.clone()], vec![global.clone()]);

            let (vault_candidates, global_candidates) = result.into_parts();

            assert_eq!(vault_candidates, vec![vault]);
            assert_eq!(global_candidates, vec![global]);
        }
    }
}
