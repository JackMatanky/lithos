//! Public discovery outcome for the settings service.

use crate::{candidate::CandidatePath, discovery::report::DiscoveryReport};

/// Public discovery output for settings service callers.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DiscoveryOutcome {
    /// Vault-local candidate config paths.
    vault: Box<[CandidatePath]>,
    /// Global candidate config paths.
    global: Box<[CandidatePath]>,
    /// Non-fatal discovery diagnostics.
    report: DiscoveryReport,
}

impl DiscoveryOutcome {
    /// Create new discovery outcome.
    #[must_use]
    #[inline]
    pub fn new(
        vault: Box<[CandidatePath]>,
        global: Box<[CandidatePath]>,
        report: DiscoveryReport,
    ) -> Self {
        Self {
            vault,
            global,
            report,
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

    /// Non-fatal discovery diagnostics.
    #[must_use]
    #[inline]
    pub fn report(&self) -> &DiscoveryReport {
        &self.report
    }
}
