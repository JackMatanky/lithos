//! Public discovery outcome for the settings service.

use crate::candidate::CandidatePath;

/// Public discovery output for settings service callers.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DiscoveryOutcome {
    /// Vault-local candidate config paths.
    vault: Box<[CandidatePath]>,
    /// Global candidate config paths.
    global: Box<[CandidatePath]>,
}

impl DiscoveryOutcome {
    /// Create new discovery outcome.
    #[must_use]
    #[inline]
    pub fn new(
        vault: Box<[CandidatePath]>,
        global: Box<[CandidatePath]>,
    ) -> Self {
        Self {
            vault,
            global,
        }
    }

    /// Vault-local candidate config paths.
    #[must_use]
    #[inline]
    pub fn vault(&self) -> &[CandidatePath] {
        &self.vault
    }

    /// Global candidate config paths.
    #[must_use]
    #[inline]
    pub fn global(&self) -> &[CandidatePath] {
        &self.global
    }
}
