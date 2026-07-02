//! Typestate processor for the redesigned discovery pipeline.
//!
//! [`DiscoveryProcessor`] executes the fixed discovery process through a
//! sequence of typed phase transitions. Each phase is a zero-sized marker type
//! that encodes what has been computed and what transitions are valid. The
//! compiler prevents out-of-order or illegal transitions.
//!
//! # Phase Sequence
//!
//! ```text
//! Init
//!  └─ FlagOverride      (probe flag vault/config; parse ceilings)
//!      ├─ [flag vault + flag config] ──────────────────────────── CacheResolution ──► Finalized
//!      ├─ [flag vault only] ───────────── GlobalResolution ─────► CacheResolution ──► Finalized
//!      └─ [flags insufficient] ──► EnvOverride
//!                                      ├─ [env vault + env config] ── CacheResolution ──► Finalized
//!                                      ├─ [env vault only] ─── GlobalResolution ─► CacheResolution ──► Finalized
//!                                      ├─ [env config only] ─► AscendingTraversal ─► CacheResolution ──► Finalized
//!                                      └─ [no overrides] ────► AscendingTraversal ─► GlobalResolution ─► CacheResolution ──► Finalized
//! ```
//!
//! Flags have highest precedence. [`FlagOverride::flag_branch`] decides
//! whether the pipeline is complete from flags alone. Only when flags are
//! insufficient does the pipeline advance to [`EnvOverride`], where
//! [`EnvOverride::branch_strategy`] makes the final routing decision.

use std::{collections::HashSet, marker::PhantomData, path::PathBuf};

use traces_fs::DirPath;

use super::{
    context::DiscoveryContext,
    error::DiscoveryError,
    probe::FolderProbe,
    report::{
        DiscoveryReport, GlobalResolutionSkipReason, LocalTraversalStopReason,
        SkippedCeiling, SkippedCeilingReason,
    },
    service::{DiscoveryResult, DiscoveryServiceConfig},
    walk::BoundedAscent,
};
use crate::{
    candidate::CandidatePath,
    discovery::location::{
        CacheLocation, CacheRoot, GlobalCacheLocation, LocalCacheLocation,
    },
    os_dirs::XDG_CACHE_HOME,
};

// ---------------------------------------------------------------------------
// Phase marker types
// ---------------------------------------------------------------------------

/// Initial phase. Holds the context and config; no probing has occurred yet.
pub(crate) struct Init;

/// CLI flag overrides have been applied. Vault candidates are populated if a
/// flag vault dir was provided. Ceiling data is parsed. [`flag_branch`] on
/// this phase decides whether the pipeline is complete from flags alone or
/// must advance to [`EnvOverride`] to consult environment variables.
///
/// [`flag_branch`]: DiscoveryProcessor::flag_branch
pub(crate) struct FlagOverride;

/// Environment variable overrides have been applied. Reached only when CLI
/// flags alone were insufficient to determine the full pipeline path. If an
/// explicit config file was supplied (via env),
/// [`LocalTraversalStopReason::ExplicitConfigFile`] is set on the report.
/// [`branch_strategy`] on this phase makes the final routing decision.
///
/// [`branch_strategy`]: DiscoveryProcessor::branch_strategy
pub(crate) struct EnvOverride;

/// Ascending traversal has run from the context anchor (or resolved env
/// anchor). Vault candidates are populated from the first directory in which
/// marker files were found, or remain empty if traversal was bounded before
/// finding any.
pub(crate) struct AscendingTraversal;

/// Global directories have been probed. Global candidates are populated (or
/// empty if no markers were found or global resolution was suppressed).
pub(crate) struct GlobalResolution;

/// Cache root has been resolved. The `cache_root` field on the processor holds
/// the computed [`CacheRoot`]. This phase is the only valid predecessor of
/// [`Finalized`].
pub(crate) struct CacheResolution;

/// All phases complete. [`DiscoveryProcessor::finalize`] consumes the processor
/// and returns the [`DiscoveryResult`] and [`DiscoveryReport`].
pub(crate) struct Finalized;

// ---------------------------------------------------------------------------
// Branch strategy
// ---------------------------------------------------------------------------

/// The execution path selected after all overrides are known.
///
/// Determined by [`DiscoveryProcessor::branch_strategy`] on the
/// [`EnvOverride`] phase and used by [`DiscoveryService`] to drive the
/// correct transition sequence.
///
/// [`DiscoveryService`]: super::service::DiscoveryService
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[expect(
    clippy::enum_variant_names,
    reason = "variants describe branch strategy, sharing a suffix is \
              intentional"
)]
pub(crate) enum ExplicitOverrideBranch {
    /// Explicit vault dir + explicit config file: vault already probed in
    /// `FlagOverride`; global resolution is skipped because both paths are
    /// already known.
    VaultProbedSkipGlobal,
    /// Explicit vault dir, no explicit config file: vault already probed in
    /// `FlagOverride`; global resolution still runs.
    VaultProbedRunGlobal,
    /// No vault override, explicit config file: ascending traversal runs to
    /// locate the vault, but global resolution is skipped.
    AscendSkipGlobal,
    /// No overrides: ascending traversal runs, then global resolution runs.
    AscendThenGlobal,
}

/// The routing decision made from CLI flags alone, before env vars are read.
///
/// Returned by [`DiscoveryProcessor::flag_branch`] on the [`FlagOverride`]
/// phase. If flags fully determine the pipeline path, the service skips
/// [`EnvOverride`] entirely, enforcing flag > env precedence at the
/// structural level.
///
/// [`DiscoveryService`]: super::service::DiscoveryService
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FlagBranch {
    /// Flag vault dir + flag config file: both paths are known from flags
    /// alone. Skip global resolution and env entirely.
    VaultAndConfigSet,
    /// Flag vault dir only, no flag config file: vault is known; global
    /// resolution still runs. Env is not consulted.
    VaultOnlySet,
    /// Flags alone are insufficient to determine the full path. Advance to
    /// [`EnvOverride`] to check environment variables.
    ConsultEnv,
}

// ---------------------------------------------------------------------------
// Processor
// ---------------------------------------------------------------------------

/// Phase-aligned typestate processor for the discovery pipeline.
///
/// `'ctx` is the lifetime of both the [`DiscoveryServiceConfig`] reference and
/// the [`DiscoveryContext`] reference (which itself borrows environment data
/// for `'ctx`). `P` is the current phase marker type.
///
/// Fields:
/// - `vault` / `global` are candidate accumulators populated across phases.
/// - `report` accumulates non-fatal metadata across phases.
/// - `ceilings` is computed during [`FlagOverride`] and used during
///   [`AscendingTraversal`].
pub(crate) struct DiscoveryProcessor<'ctx, P> {
    config: &'ctx DiscoveryServiceConfig,
    ctx: &'ctx DiscoveryContext<'ctx>,
    vault: Vec<CandidatePath>,
    global: Vec<CandidatePath>,
    report: DiscoveryReport,
    /// Parsed and canonicalized ceiling directories. Populated during
    /// [`FlagOverride`] so the same set is available in [`AscendingTraversal`]
    /// without re-parsing.
    ceilings: HashSet<PathBuf>,
    /// Resolved cache root. Populated exclusively during the
    /// [`CacheResolution`] phase transition. `None` until that phase runs.
    cache_root: Option<CacheRoot>,
    _phase: PhantomData<P>,
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

impl<'ctx> DiscoveryProcessor<'ctx, Init> {
    /// Constructs the processor with its stable config and per-invocation
    /// context. No probing occurs here.
    pub(crate) fn new(
        config: &'ctx DiscoveryServiceConfig,
        ctx: &'ctx DiscoveryContext<'ctx>,
    ) -> Self {
        Self {
            config,
            ctx,
            vault: Vec::new(),
            global: Vec::new(),
            report: DiscoveryReport::default(),
            ceilings: HashSet::new(),
            cache_root: None,
            _phase: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Init → FlagOverride
// ---------------------------------------------------------------------------

impl<'ctx> From<DiscoveryProcessor<'ctx, Init>>
    for DiscoveryProcessor<'ctx, FlagOverride>
{
    /// Probes the flag and environment vault directories and parses ceiling
    /// data.
    ///
    /// After this transition:
    /// - `vault` contains candidates from the explicit vault dir (flag takes
    ///   precedence over env; the first non-empty probe wins).
    /// - `ceilings` contains validated, canonicalized ceiling directories.
    /// - `report.skipped_ceilings` contains entries for invalid ceiling
    ///   segments.
    fn from(val: DiscoveryProcessor<'ctx, Init>) -> Self {
        let vault_probe = FolderProbe {
            patterns: val.config.vault_marker_patterns,
        };

        // Probe flag vault dir first (highest precedence), then env vault dir.
        let vault = val
            .ctx
            .flags()
            .vault_dir()
            .map(|d| vault_probe.probe(d))
            .filter(|v| !v.is_empty())
            .or_else(|| {
                val.ctx
                    .env()
                    .vault_dir()
                    .map(|d| vault_probe.probe(d))
                    .filter(|v| !v.is_empty())
            })
            .unwrap_or_default();

        // Parse ceiling dirs from the raw env value.
        let (valid_ceilings, skipped_ceilings) =
            parse_ceiling_dirs(val.ctx.env().ceiling_dirs_raw());

        let ceilings: HashSet<PathBuf> = valid_ceilings
            .into_iter()
            .map(|d| d.as_path().to_path_buf())
            .collect();

        let mut report = val.report;
        report.skipped_ceilings = skipped_ceilings;

        DiscoveryProcessor {
            config: val.config,
            ctx: val.ctx,
            vault,
            global: val.global,
            report,
            ceilings,
            cache_root: val.cache_root,
            _phase: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// FlagOverride: branch decision
// ---------------------------------------------------------------------------

impl DiscoveryProcessor<'_, FlagOverride> {
    /// Decides the pipeline path from CLI flags alone.
    ///
    /// Called by [`DiscoveryService`] before consulting env vars. When the
    /// return value is not [`FlagBranch::ConsultEnv`], the service uses the
    /// appropriate `FlagOverride →` transition directly and never constructs
    /// an [`EnvOverride`] processor.
    ///
    /// [`DiscoveryService`]: super::service::DiscoveryService
    pub(crate) fn flag_branch(&self) -> FlagBranch {
        let has_flag_vault = self.ctx.flags().vault_dir().is_some();
        let has_flag_config = self.ctx.flags().config_file().is_some();

        match (has_flag_vault, has_flag_config) {
            (true, true) => FlagBranch::VaultAndConfigSet,
            (true, false) => FlagBranch::VaultOnlySet,
            _ => FlagBranch::ConsultEnv,
        }
    }
}

// ---------------------------------------------------------------------------
// FlagOverride → GlobalResolution  (VaultOnlySet branch)
// ---------------------------------------------------------------------------

impl<'ctx> From<DiscoveryProcessor<'ctx, FlagOverride>>
    for DiscoveryProcessor<'ctx, GlobalResolution>
{
    /// Advances to global resolution when only a flag vault dir is set (no
    /// flag config file).
    ///
    /// The vault is already probed; env vars are not consulted because the
    /// vault source is fully determined by the flag.
    fn from(val: DiscoveryProcessor<'ctx, FlagOverride>) -> Self {
        run_global_resolution(val)
    }
}

// ---------------------------------------------------------------------------
// FlagOverride → EnvOverride  (ConsultEnv branch)
// ---------------------------------------------------------------------------

impl<'ctx> From<DiscoveryProcessor<'ctx, FlagOverride>>
    for DiscoveryProcessor<'ctx, EnvOverride>
{
    /// Applies environment variable overrides.
    ///
    /// Only reached when [`flag_branch`] returned [`FlagBranch::ConsultEnv`],
    /// meaning no flag vault dir was set. This transition probes the env vault
    /// dir (if present) and records
    /// [`LocalTraversalStopReason::ExplicitConfigFile`] if an env config
    /// file is set.
    ///
    /// [`flag_branch`]: DiscoveryProcessor::flag_branch
    fn from(val: DiscoveryProcessor<'ctx, FlagOverride>) -> Self {
        let vault_probe = FolderProbe {
            patterns: val.config.vault_marker_patterns,
        };

        // No flag vault dir on this path (flag_branch ruled that out).
        // Probe env vault dir if present.
        let vault = val
            .ctx
            .env()
            .vault_dir()
            .map(|d| vault_probe.probe(d))
            .filter(|v| !v.is_empty())
            .unwrap_or(val.vault);

        // A flag config file also preempts local traversal even on the
        // ConsultEnv path (flag config + no flag vault). Record this here so
        // the stop reason is correct for all ConsultEnv sub-paths.
        let has_config_override = val.ctx.flags().config_file().is_some()
            || val.ctx.env().config_file().is_some();
        let mut report = val.report;
        if has_config_override {
            report.local_traversal_stop_reason =
                LocalTraversalStopReason::ExplicitConfigFile;
        }

        DiscoveryProcessor {
            config: val.config,
            ctx: val.ctx,
            vault,
            global: val.global,
            report,
            ceilings: val.ceilings,
            cache_root: val.cache_root,
            _phase: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// EnvOverride: branch strategy
// ---------------------------------------------------------------------------

impl DiscoveryProcessor<'_, EnvOverride> {
    /// Determines the execution path when flags alone were insufficient.
    ///
    /// Only called after [`FlagBranch::ConsultEnv`] was returned by
    /// [`flag_branch`]. Checks environment variables only — CLI flags have
    /// already been ruled out as insufficient at this point.
    ///
    /// [`flag_branch`]: DiscoveryProcessor::flag_branch
    pub(crate) fn branch_strategy(&self) -> ExplicitOverrideBranch {
        // This phase is only reached when flag_branch() returned ConsultEnv,
        // so no flag vault dir or flag config file is set. Branch on env vars
        // only.
        let has_env_vault = self.ctx.env().vault_dir().is_some();
        let has_env_config = self.ctx.env().config_file().is_some();

        match (has_env_vault, has_env_config) {
            (true, true) => ExplicitOverrideBranch::VaultProbedSkipGlobal,
            (true, false) => ExplicitOverrideBranch::VaultProbedRunGlobal,
            (false, true) => ExplicitOverrideBranch::AscendSkipGlobal,
            (false, false) => ExplicitOverrideBranch::AscendThenGlobal,
        }
    }
}

// ---------------------------------------------------------------------------
// EnvOverride → AscendingTraversal
// ---------------------------------------------------------------------------

impl<'ctx> TryFrom<DiscoveryProcessor<'ctx, EnvOverride>>
    for DiscoveryProcessor<'ctx, AscendingTraversal>
{
    type Error = DiscoveryError;

    /// Walks the directory tree upward from the context anchor, probing each
    /// directory for vault marker files.
    ///
    /// This transition is only taken on the `AscendSkipGlobal` or
    /// `AscendThenGlobal` branches — meaning no vault override is present.
    /// The traversal anchor is therefore always `ctx.anchor()`.
    ///
    /// Traversal stops at:
    /// - A configured ceiling directory (sets
    ///   [`LocalTraversalStopReason::CeilingEnforced`]).
    /// - A project boundary marker child (sets
    ///   [`LocalTraversalStopReason::ProjectBoundaryMarker`]).
    /// - The filesystem root (default; report already initialised to
    ///   [`LocalTraversalStopReason::FilesystemRoot`]).
    ///
    /// # Errors
    ///
    /// Returns [`DiscoveryError::CanonicalizePath`] if the anchor cannot be
    /// canonicalized.
    fn try_from(
        val: DiscoveryProcessor<'ctx, EnvOverride>,
    ) -> Result<Self, Self::Error> {
        // No vault override is active on this branch; the anchor is always the
        // context anchor.
        let anchor_path = val.ctx.anchor().as_path().to_path_buf();
        let canonical = anchor_path.canonicalize().map_err(|source| {
            DiscoveryError::CanonicalizePath {
                path: anchor_path,
                source,
            }
        })?;

        let probe = FolderProbe {
            patterns: val.config.vault_marker_patterns,
        };

        // Destructure `val` so all fields can be moved or borrowed
        // independently without conflicting with the partial-move rules.
        let DiscoveryProcessor {
            config,
            ctx,
            vault: init_vault,
            global,
            report: init_report,
            ceilings,
            cache_root,
            _phase: _,
        } = val;

        let walker = BoundedAscent::new(
            &canonical,
            &ceilings,
            config.allow_marker_at_ceiling,
        );

        let mut vault = init_vault;
        let mut report = init_report;

        for current in walker {
            // A boundary marker is a well-known child directory (e.g. `.git`)
            // whose *presence inside* the current directory signals a project
            // root. We check whether the child exists, not whether `current`
            // itself is named after a boundary marker.
            let is_boundary = config
                .boundary_markers
                .iter()
                .any(|marker| current.join(marker).is_dir());

            if is_boundary {
                report.local_traversal_stop_reason =
                    LocalTraversalStopReason::ProjectBoundaryMarker {
                        marker: current.to_path_buf(),
                    };
                break;
            }

            // Check whether this is a ceiling directory *before* probing, so
            // the stop reason is recorded correctly whether or not a marker is
            // found at the ceiling.
            let is_ceiling = ceilings.contains(current);

            let candidates = probe.probe_dir(current);
            let found = !candidates.is_empty();
            if found {
                vault = candidates;
            }

            if is_ceiling {
                // BoundedAscent yields the ceiling once (when
                // allow_marker_at_ceiling is true) then stops. Record the
                // ceiling stop reason regardless of whether a marker was found.
                report.local_traversal_stop_reason =
                    LocalTraversalStopReason::CeilingEnforced {
                        ceiling: current.to_path_buf(),
                    };
                break;
            }

            if found {
                // Marker found in a non-ceiling directory; stop traversal.
                // Stop reason remains FilesystemRoot (the default).
                break;
            }
        }

        Ok(DiscoveryProcessor {
            config,
            ctx,
            vault,
            global,
            report,
            ceilings,
            cache_root,
            _phase: PhantomData,
        })
    }
}

// ---------------------------------------------------------------------------
// EnvOverride → GlobalResolution  (VaultProbedRunGlobal branch)
// ---------------------------------------------------------------------------

impl<'ctx> From<DiscoveryProcessor<'ctx, EnvOverride>>
    for DiscoveryProcessor<'ctx, GlobalResolution>
{
    /// Probes configured global directories for config markers.
    ///
    /// If `suppress_global` is set on the invocation flags, no probing occurs
    /// and [`GlobalResolutionSkipReason::SuppressedByFlag`] is recorded.
    fn from(val: DiscoveryProcessor<'ctx, EnvOverride>) -> Self {
        run_global_resolution(val)
    }
}

// ---------------------------------------------------------------------------
// AscendingTraversal → GlobalResolution  (AscendThenGlobal branch)
// ---------------------------------------------------------------------------

impl<'ctx> From<DiscoveryProcessor<'ctx, AscendingTraversal>>
    for DiscoveryProcessor<'ctx, GlobalResolution>
{
    /// Probes configured global directories for config markers.
    ///
    /// If `suppress_global` is set on the invocation flags, no probing occurs
    /// and [`GlobalResolutionSkipReason::SuppressedByFlag`] is recorded.
    fn from(val: DiscoveryProcessor<'ctx, AscendingTraversal>) -> Self {
        run_global_resolution(val)
    }
}

// ---------------------------------------------------------------------------
// CacheResolution → Finalized
// ---------------------------------------------------------------------------

impl<'ctx> From<DiscoveryProcessor<'ctx, CacheResolution>>
    for DiscoveryProcessor<'ctx, Finalized>
{
    /// Advances from cache resolution to the terminal finalized phase.
    ///
    /// At this point `cache_root` is guaranteed to be `Some` — set by
    /// `run_cache_resolution`. The `finalize()` method will unwrap it.
    fn from(val: DiscoveryProcessor<'ctx, CacheResolution>) -> Self {
        DiscoveryProcessor {
            config: val.config,
            ctx: val.ctx,
            vault: val.vault,
            global: val.global,
            report: val.report,
            ceilings: val.ceilings,
            cache_root: val.cache_root,
            _phase: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Any phase → CacheResolution
// ---------------------------------------------------------------------------

/// Shared body for all `→ CacheResolution` transitions.
///
/// Computes the [`CacheRoot`] from accumulated vault candidates and the
/// optional env-var cache directory override. Each concrete `From` impl
/// delegates here to avoid code duplication.
fn run_cache_resolution<P>(
    val: DiscoveryProcessor<'_, P>,
) -> DiscoveryProcessor<'_, CacheResolution> {
    let env_cache_dir = val.ctx.env().cache_dir();
    let vault_root = val.vault.first().map(CandidatePath::base);
    let cache_root = resolve_cache_root(env_cache_dir, vault_root);

    DiscoveryProcessor {
        config: val.config,
        ctx: val.ctx,
        vault: val.vault,
        global: val.global,
        report: val.report,
        ceilings: val.ceilings,
        cache_root: Some(cache_root),
        _phase: PhantomData,
    }
}

/// Advances directly from the initial phase to cache resolution,
/// bypassing vault probing.
///
/// **Test-only.** Production code must go through `FlagOverride` (and
/// optionally `EnvOverride`, `AscendingTraversal`, `GlobalResolution`)
/// before reaching `CacheResolution`. Using this impl in production would
/// compute the cache root before any vault candidates are accumulated,
/// silently producing `PlatformUserCache` even when a vault is present.
#[cfg(test)]
impl<'ctx> From<DiscoveryProcessor<'ctx, Init>>
    for DiscoveryProcessor<'ctx, CacheResolution>
{
    fn from(val: DiscoveryProcessor<'ctx, Init>) -> Self {
        run_cache_resolution(val)
    }
}

impl<'ctx> From<DiscoveryProcessor<'ctx, FlagOverride>>
    for DiscoveryProcessor<'ctx, CacheResolution>
{
    /// Advances from `FlagOverride` to cache resolution.
    ///
    /// In production this impl is only invoked on the
    /// [`FlagBranch::VaultAndConfigSet`] path (both flag vault dir and flag
    /// config file are set), so `has_flag_config` is always `true` on the
    /// reachable path. The `debug_assert!` below catches any future misuse
    /// where this transition is called without a flag config file (which would
    /// silently skip setting [`LocalTraversalStopReason::ExplicitConfigFile`]).
    fn from(val: DiscoveryProcessor<'ctx, FlagOverride>) -> Self {
        let has_flag_config = val.ctx.flags().config_file().is_some();
        // In production this is only reached via VaultAndConfigSet, where
        // has_flag_config is always true. The assert fires in debug builds if
        // this transition is ever taken without a flag config file.
        debug_assert!(
            has_flag_config,
            "FlagOverride → CacheResolution is only valid on the \
             VaultAndConfigSet path; has_flag_config should always be true \
             here"
        );
        let mut val = val;
        if has_flag_config {
            val.report.local_traversal_stop_reason =
                LocalTraversalStopReason::ExplicitConfigFile;
        }
        run_cache_resolution(val)
    }
}

impl<'ctx> From<DiscoveryProcessor<'ctx, EnvOverride>>
    for DiscoveryProcessor<'ctx, CacheResolution>
{
    fn from(val: DiscoveryProcessor<'ctx, EnvOverride>) -> Self {
        run_cache_resolution(val)
    }
}

impl<'ctx> From<DiscoveryProcessor<'ctx, AscendingTraversal>>
    for DiscoveryProcessor<'ctx, CacheResolution>
{
    fn from(val: DiscoveryProcessor<'ctx, AscendingTraversal>) -> Self {
        run_cache_resolution(val)
    }
}

impl<'ctx> From<DiscoveryProcessor<'ctx, GlobalResolution>>
    for DiscoveryProcessor<'ctx, CacheResolution>
{
    fn from(val: DiscoveryProcessor<'ctx, GlobalResolution>) -> Self {
        run_cache_resolution(val)
    }
}

// ---------------------------------------------------------------------------
// Finalized
// ---------------------------------------------------------------------------

impl DiscoveryProcessor<'_, Finalized> {
    /// Consumes the processor and returns the discovery output.
    pub(crate) fn finalize(self) -> DiscoveryResult {
        #[expect(
            clippy::expect_used,
            reason = "Finalized is only reachable via CacheResolution which \
                      always sets cache_root"
        )]
        let cache_root =
            self.cache_root.expect("cache_root must be set before Finalized");
        DiscoveryResult::new(self.vault, self.global, cache_root, self.report)
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Probes configured global directories and returns a processor in the
/// [`GlobalResolution`] phase.
///
/// Shared by both `EnvOverride → GlobalResolution` and
/// `AscendingTraversal → GlobalResolution` to avoid duplicating the probe loop
/// and suppression logic.
fn run_global_resolution<P>(
    val: DiscoveryProcessor<'_, P>,
) -> DiscoveryProcessor<'_, GlobalResolution> {
    let DiscoveryProcessor {
        config,
        ctx,
        vault,
        global: _,
        mut report,
        ceilings,
        cache_root,
        _phase: _,
    } = val;

    if ctx.flags().suppress_global() {
        report.global_resolution_skip_reason =
            Some(GlobalResolutionSkipReason::SuppressedByFlag);
        // Do not probe — return empty global candidates.
        return DiscoveryProcessor {
            config,
            ctx,
            vault,
            global: Vec::new(),
            report,
            ceilings,
            cache_root,
            _phase: PhantomData,
        };
    }

    let probe = FolderProbe {
        patterns: config.global_marker_patterns,
    };

    let global = config
        .global_directories
        .iter()
        .find_map(|global_dir| {
            let candidates = probe.probe(global_dir);
            (!candidates.is_empty()).then_some(candidates)
        })
        .unwrap_or_default();

    DiscoveryProcessor {
        config,
        ctx,
        vault,
        global,
        report,
        ceilings,
        cache_root,
        _phase: PhantomData,
    }
}

/// Resolves the cache root directory from the available inputs.
///
/// Precedence (highest → lowest):
/// 1. `TRACES_CACHE_DIR` env var (passed as `env_cache_dir`)
/// 2. Vault-local default: `<vault_root>/.traces/cache/`
/// 3. OS platform user-cache directory (`XDG_CACHE_HOME / "traces"`).
fn resolve_cache_root(
    env_cache_dir: Option<&std::path::Path>,
    vault_root: Option<&DirPath>,
) -> CacheRoot {
    if let Some(path) = env_cache_dir {
        return CacheRoot::new(
            CacheLocation::Global(GlobalCacheLocation::EnvironmentOverride),
            path.to_path_buf(),
        );
    }
    if let Some(root) = vault_root {
        return CacheRoot::new(
            CacheLocation::Local(LocalCacheLocation::ProjectCacheDirectory),
            root.as_path().join(".traces/cache"),
        );
    }
    // No env override and no vault root — use OS platform cache directory.
    let path = XDG_CACHE_HOME.join("traces");
    CacheRoot::new(
        CacheLocation::Global(GlobalCacheLocation::PlatformUserCache),
        path,
    )
}

/// Parses a raw OS path-list of ceiling directories into validated
/// [`DirPath`] values plus skipped entries.
///
/// Splitting, trimming, validation, and canonicalization are all done here so
/// this logic is not coupled to [`DiscoveryEnv`].
///
/// [`DiscoveryEnv`]: super::context::DiscoveryEnv
fn parse_ceiling_dirs(
    raw: Option<&std::ffi::OsStr>,
) -> (Vec<DirPath>, Vec<SkippedCeiling>) {
    let Some(raw) = raw else {
        return (Vec::new(), Vec::new());
    };

    let mut valid = Vec::new();
    let mut skipped = Vec::new();

    for segment in std::env::split_paths(raw) {
        let trimmed = segment.to_string_lossy().trim().to_owned();
        if trimmed.is_empty() {
            skipped.push(SkippedCeiling {
                segment,
                reason: SkippedCeilingReason::EmptySegment,
            });
            continue;
        }

        let Ok(dir) = DirPath::try_new(PathBuf::from(&trimmed)) else {
            skipped.push(SkippedCeiling {
                segment: PathBuf::from(&trimmed),
                reason: SkippedCeilingReason::InvalidPath,
            });
            continue;
        };

        match dir.as_path().canonicalize() {
            Ok(canonical) => match DirPath::try_new(canonical) {
                Ok(canon_dir) => valid.push(canon_dir),
                Err(_) => skipped.push(SkippedCeiling {
                    segment: PathBuf::from(&trimmed),
                    reason: SkippedCeilingReason::InvalidPath,
                }),
            },
            Err(_) => skipped.push(SkippedCeiling {
                segment: PathBuf::from(&trimmed),
                reason: SkippedCeilingReason::InvalidPath,
            }),
        }
    }

    (valid, skipped)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    //! Unit tests for [`DiscoveryProcessor`] phase transitions.
    //!
    //! Each sub-module covers one phase transition or method. Tests use
    //! temporary directories from [`tempfile`] so filesystem state is isolated
    //! and cleaned up automatically.

    use std::path::Path;

    use traces_fs::DirPath;

    use super::*;
    use crate::discovery::{
        context::{DiscoveryContext, DiscoveryEnv, DiscoveryFlags},
        policy::VAULT_MARKER_PATTERNS,
        service::DiscoveryServiceConfig,
    };

    // -----------------------------------------------------------------------
    // Shared fixtures
    // -----------------------------------------------------------------------

    fn default_config() -> DiscoveryServiceConfig {
        DiscoveryServiceConfig::default()
    }

    fn make_context(
        anchor: &Path,
    ) -> Result<DiscoveryContext<'_>, DiscoveryError> {
        DiscoveryContext::new(anchor)
    }

    /// Writes an empty file at `root/relative`, creating parent directories as
    /// needed. Returns the full path.
    fn write_marker(root: &Path, relative: &str) -> PathBuf {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(&path, "").expect("write marker");
        path
    }

    // -----------------------------------------------------------------------
    // Init::new
    // -----------------------------------------------------------------------

    mod new {
        use super::*;

        #[test]
        fn stores_config_vault_marker_patterns() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");
            let p = DiscoveryProcessor::new(&config, &ctx);
            assert_eq!(p.config.vault_marker_patterns, VAULT_MARKER_PATTERNS);
        }

        #[test]
        fn stores_context_anchor() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");
            let p = DiscoveryProcessor::new(&config, &ctx);
            assert_eq!(p.ctx.anchor().as_path(), root.path());
        }

        #[test]
        fn vault_is_empty() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");
            let p = DiscoveryProcessor::new(&config, &ctx);
            assert!(p.vault.is_empty());
        }

        #[test]
        fn global_is_empty() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");
            let p = DiscoveryProcessor::new(&config, &ctx);
            assert!(p.global.is_empty());
        }

        #[test]
        fn skipped_ceilings_is_empty() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");
            let p = DiscoveryProcessor::new(&config, &ctx);
            assert!(p.report.skipped_ceilings.is_empty());
        }

        #[test]
        fn local_traversal_stop_reason_is_filesystem_root() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");
            let p = DiscoveryProcessor::new(&config, &ctx);
            assert_eq!(
                p.report.local_traversal_stop_reason,
                LocalTraversalStopReason::FilesystemRoot
            );
        }

        #[test]
        fn global_resolution_skip_reason_is_none() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");
            let p = DiscoveryProcessor::new(&config, &ctx);
            assert!(p.report.global_resolution_skip_reason.is_none());
        }
    }

    // -----------------------------------------------------------------------
    // Init → FlagOverride
    // -----------------------------------------------------------------------

    mod init_to_flag_override {
        use super::*;

        #[test]
        fn probes_flag_vault_dir_when_present() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            write_marker(vault_dir.path(), "traces.toml");

            let config = default_config();
            let flags =
                DiscoveryFlags::new(None, Some(vault_dir.path()), false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();

            assert!(!flag.vault.is_empty());
        }

        #[test]
        fn vault_is_empty_when_no_flag_vault_dir() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();

            assert!(flag.vault.is_empty());
        }

        #[test]
        fn parses_ceiling_dirs_from_env() {
            let root = tempfile::tempdir().expect("root");
            let ceiling = tempfile::tempdir().expect("ceiling");
            let raw = ceiling.path().to_str().expect("utf8").to_owned();

            let config = default_config();
            let env = DiscoveryEnv::new(
                None,
                None,
                Some(std::ffi::OsStr::new(&raw)),
                None,
            )
            .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();

            assert!(!flag.ceilings.is_empty());
        }

        #[test]
        fn records_empty_ceiling_segment_as_skipped() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let env = DiscoveryEnv::new(
                None,
                None,
                Some(std::ffi::OsStr::new("")),
                None,
            )
            .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();

            assert!(!flag.report.skipped_ceilings.is_empty());
        }
    }

    // -----------------------------------------------------------------------
    // FlagOverride::flag_branch
    // -----------------------------------------------------------------------

    mod flag_branch {
        use super::*;

        fn make_flag<'a>(
            config: &'a DiscoveryServiceConfig,
            ctx: &'a DiscoveryContext<'a>,
        ) -> DiscoveryProcessor<'a, FlagOverride> {
            DiscoveryProcessor::new(config, ctx).into()
        }

        #[test]
        fn both_flag_vault_and_config_yields_vault_and_config_set() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags = DiscoveryFlags::new(
                Some(config_file.path()),
                Some(vault_dir.path()),
                false,
            )
            .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            assert_eq!(
                make_flag(&config, &ctx).flag_branch(),
                FlagBranch::VaultAndConfigSet
            );
        }

        #[test]
        fn flag_vault_only_yields_vault_only_set() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");

            let config = default_config();
            let flags =
                DiscoveryFlags::new(None, Some(vault_dir.path()), false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            assert_eq!(
                make_flag(&config, &ctx).flag_branch(),
                FlagBranch::VaultOnlySet
            );
        }

        #[test]
        fn flag_config_only_yields_consult_env() {
            // A flag config file alone does not short-circuit; env must still
            // be consulted for a possible vault override.
            let root = tempfile::tempdir().expect("root");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags =
                DiscoveryFlags::new(Some(config_file.path()), None, false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            assert_eq!(
                make_flag(&config, &ctx).flag_branch(),
                FlagBranch::ConsultEnv
            );
        }

        #[test]
        fn no_flags_yields_consult_env() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            assert_eq!(
                make_flag(&config, &ctx).flag_branch(),
                FlagBranch::ConsultEnv
            );
        }

        #[test]
        fn env_vault_does_not_affect_flag_branch() {
            // flag_branch only reads CLI flags; env vars are ignored here.
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");

            let config = default_config();
            let env =
                DiscoveryEnv::new(None, Some(vault_dir.path()), None, None)
                    .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            assert_eq!(
                make_flag(&config, &ctx).flag_branch(),
                FlagBranch::ConsultEnv
            );
        }
    }

    // -----------------------------------------------------------------------
    // FlagOverride → EnvOverride  (ConsultEnv path only)
    // -----------------------------------------------------------------------

    mod flag_to_env_override {
        use super::*;

        /// Advances to `EnvOverride` via the `ConsultEnv` path (no flag
        /// vault dir set, so `flag_branch` returns `ConsultEnv`).
        fn advance_to_env<'a>(
            config: &'a DiscoveryServiceConfig,
            ctx: &'a DiscoveryContext<'a>,
        ) -> DiscoveryProcessor<'a, EnvOverride> {
            let init = DiscoveryProcessor::new(config, ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            flag.into()
        }

        #[test]
        fn probes_env_vault_dir_when_present() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            write_marker(vault_dir.path(), "traces.toml");

            let config = default_config();
            let env =
                DiscoveryEnv::new(None, Some(vault_dir.path()), None, None)
                    .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let env_p = advance_to_env(&config, &ctx);

            assert!(!env_p.vault.is_empty());
        }

        #[test]
        fn vault_is_empty_when_no_env_vault_dir() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let env_p = advance_to_env(&config, &ctx);

            assert!(env_p.vault.is_empty());
        }

        #[test]
        fn sets_explicit_config_stop_reason_when_flag_config_present() {
            // Flag config only reaches EnvOverride via ConsultEnv; the stop
            // reason must be set here even though env has no config override.
            let root = tempfile::tempdir().expect("root");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags =
                DiscoveryFlags::new(Some(config_file.path()), None, false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let env_p = advance_to_env(&config, &ctx);

            assert_eq!(
                env_p.report.local_traversal_stop_reason,
                LocalTraversalStopReason::ExplicitConfigFile
            );
        }

        #[test]
        fn sets_explicit_config_stop_reason_when_env_config_present() {
            let root = tempfile::tempdir().expect("root");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let env_override =
                DiscoveryEnv::new(Some(config_file.path()), None, None, None)
                    .expect("env");
            let ctx =
                make_context(root.path()).expect("ctx").with_env(env_override);

            let env_p = advance_to_env(&config, &ctx);

            assert_eq!(
                env_p.report.local_traversal_stop_reason,
                LocalTraversalStopReason::ExplicitConfigFile
            );
        }

        #[test]
        fn leaves_stop_reason_as_filesystem_root_when_no_env_config() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let env_p = advance_to_env(&config, &ctx);

            assert_eq!(
                env_p.report.local_traversal_stop_reason,
                LocalTraversalStopReason::FilesystemRoot
            );
        }
    }

    // -----------------------------------------------------------------------
    // EnvOverride::branch_strategy  (env vars only; no flag overrides)
    // -----------------------------------------------------------------------

    mod branch_strategy {
        use super::*;

        /// Advances to `EnvOverride` with no CLI flags set, so only env vars
        /// influence `branch_strategy`.
        fn advance_to_env<'a>(
            config: &'a DiscoveryServiceConfig,
            ctx: &'a DiscoveryContext<'a>,
        ) -> DiscoveryProcessor<'a, EnvOverride> {
            let init = DiscoveryProcessor::new(config, ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            flag.into()
        }

        #[test]
        fn env_vault_and_config_yields_vault_probed_skip_global() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let env = DiscoveryEnv::new(
                Some(config_file.path()),
                Some(vault_dir.path()),
                None,
                None,
            )
            .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            assert_eq!(
                advance_to_env(&config, &ctx).branch_strategy(),
                ExplicitOverrideBranch::VaultProbedSkipGlobal
            );
        }

        #[test]
        fn env_vault_only_yields_vault_probed_run_global() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");

            let config = default_config();
            let env =
                DiscoveryEnv::new(None, Some(vault_dir.path()), None, None)
                    .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            assert_eq!(
                advance_to_env(&config, &ctx).branch_strategy(),
                ExplicitOverrideBranch::VaultProbedRunGlobal
            );
        }

        #[test]
        fn env_config_only_yields_ascend_skip_global() {
            let root = tempfile::tempdir().expect("root");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let env =
                DiscoveryEnv::new(Some(config_file.path()), None, None, None)
                    .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            assert_eq!(
                advance_to_env(&config, &ctx).branch_strategy(),
                ExplicitOverrideBranch::AscendSkipGlobal
            );
        }

        #[test]
        fn no_env_overrides_yields_ascend_then_global() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            assert_eq!(
                advance_to_env(&config, &ctx).branch_strategy(),
                ExplicitOverrideBranch::AscendThenGlobal
            );
        }
    }

    // -----------------------------------------------------------------------
    // EnvOverride → AscendingTraversal
    // -----------------------------------------------------------------------

    mod ascending_traversal {
        use super::*;

        fn advance_to_ascend<'a>(
            config: &'a DiscoveryServiceConfig,
            ctx: &'a DiscoveryContext<'a>,
        ) -> Result<DiscoveryProcessor<'a, AscendingTraversal>, DiscoveryError>
        {
            let init = DiscoveryProcessor::new(config, ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            env.try_into()
        }

        #[test]
        fn finds_marker_in_anchor_directory() {
            let root = tempfile::tempdir().expect("root");
            write_marker(root.path(), "traces.toml");

            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let ascend = advance_to_ascend(&config, &ctx).expect("ascend");

            assert!(!ascend.vault.is_empty());
        }

        #[test]
        fn vault_is_empty_when_no_marker_found() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();

            // Limit traversal to just root so we don't accidentally find real
            // markers in parent directories.
            let raw = root
                .path()
                .canonicalize()
                .expect("canonical")
                .to_str()
                .expect("utf8")
                .to_owned();
            let env = DiscoveryEnv::new(
                None,
                None,
                Some(std::ffi::OsStr::new(&raw)),
                None,
            )
            .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let ascend = advance_to_ascend(&config, &ctx).expect("ascend");

            assert!(ascend.vault.is_empty());
        }

        #[test]
        fn stops_at_project_boundary_marker_present_as_child() {
            // Correct scenario: anchor is a project directory that has a .git
            // child, not the CWD being inside .git.
            let root = tempfile::tempdir().expect("root");
            std::fs::create_dir(root.path().join(".git")).expect(".git dir");

            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let ascend = advance_to_ascend(&config, &ctx).expect("ascend");

            assert_eq!(
                ascend.report.local_traversal_stop_reason,
                LocalTraversalStopReason::ProjectBoundaryMarker {
                    marker: root.path().canonicalize().expect("canonical")
                }
            );
        }

        #[test]
        fn vault_is_empty_when_stopped_at_boundary_marker() {
            let root = tempfile::tempdir().expect("root");
            std::fs::create_dir(root.path().join(".git")).expect(".git dir");

            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let ascend = advance_to_ascend(&config, &ctx).expect("ascend");

            assert!(ascend.vault.is_empty());
        }

        #[test]
        fn stops_at_ceiling_and_records_stop_reason() {
            let root = tempfile::tempdir().expect("root");
            let sub = root.path().join("sub");
            std::fs::create_dir(&sub).expect("sub dir");
            // marker is in root, but ceiling is root — traversal must not reach
            // it
            write_marker(root.path(), "traces.toml");

            let ceiling =
                root.path().canonicalize().expect("canonical ceiling");
            let raw = ceiling.to_str().expect("utf8").to_owned();
            let env = DiscoveryEnv::new(
                None,
                None,
                Some(std::ffi::OsStr::new(&raw)),
                None,
            )
            .expect("env");
            let config = default_config();
            let ctx = make_context(&sub).expect("ctx").with_env(env);

            let ascend = advance_to_ascend(&config, &ctx).expect("ascend");

            assert_eq!(
                ascend.report.local_traversal_stop_reason,
                LocalTraversalStopReason::CeilingEnforced {
                    ceiling: ceiling.clone(),
                }
            );
        }

        #[test]
        fn vault_is_empty_when_ceiling_stops_traversal_before_marker() {
            let root = tempfile::tempdir().expect("root");
            let sub = root.path().join("sub");
            std::fs::create_dir(&sub).expect("sub dir");
            write_marker(root.path(), "traces.toml");

            let ceiling =
                root.path().canonicalize().expect("canonical ceiling");
            let raw = ceiling.to_str().expect("utf8").to_owned();
            let env = DiscoveryEnv::new(
                None,
                None,
                Some(std::ffi::OsStr::new(&raw)),
                None,
            )
            .expect("env");
            let config = DiscoveryServiceConfig {
                allow_marker_at_ceiling: false,
                ..DiscoveryServiceConfig::default()
            };
            let ctx = make_context(&sub).expect("ctx").with_env(env);

            let ascend = advance_to_ascend(&config, &ctx).expect("ascend");

            assert!(ascend.vault.is_empty());
        }

        #[test]
        fn ceiling_in_report_is_populated_from_env() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let env = DiscoveryEnv::new(
                None,
                None,
                Some(std::ffi::OsStr::new("")),
                None,
            )
            .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let ascend = advance_to_ascend(&config, &ctx).expect("ascend");

            assert!(!ascend.report.skipped_ceilings.is_empty());
        }
    }

    // -----------------------------------------------------------------------
    // GlobalResolution (shared by EnvOverride and AscendingTraversal paths)
    // -----------------------------------------------------------------------

    mod global_resolution {
        use super::*;

        fn advance_to_global_from_env<'a>(
            config: &'a DiscoveryServiceConfig,
            ctx: &'a DiscoveryContext<'a>,
        ) -> DiscoveryProcessor<'a, GlobalResolution> {
            let init = DiscoveryProcessor::new(config, ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            // EnvOverride → GlobalResolution is the VaultProbedRunGlobal path
            env.into()
        }

        #[test]
        fn probes_global_directories_when_not_suppressed() {
            let root = tempfile::tempdir().expect("root");
            let global_dir = tempfile::tempdir().expect("global dir");
            write_marker(global_dir.path(), "traces.toml");

            let config = DiscoveryServiceConfig {
                global_directories: vec![
                    DirPath::try_new(global_dir.path().to_path_buf())
                        .expect("dir"),
                ],
                ..DiscoveryServiceConfig::default()
            };
            let ctx = make_context(root.path()).expect("ctx");

            let global = advance_to_global_from_env(&config, &ctx);

            assert!(!global.global.is_empty());
        }

        #[test]
        fn global_candidates_are_empty_when_suppress_global_is_set() {
            let root = tempfile::tempdir().expect("root");
            let global_dir = tempfile::tempdir().expect("global dir");
            write_marker(global_dir.path(), "traces.toml");

            let config = DiscoveryServiceConfig {
                global_directories: vec![
                    DirPath::try_new(global_dir.path().to_path_buf())
                        .expect("dir"),
                ],
                ..DiscoveryServiceConfig::default()
            };
            let flags = DiscoveryFlags::new(None, None, true).expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let global = advance_to_global_from_env(&config, &ctx);

            assert!(
                global.global.is_empty(),
                "suppressed global must produce no candidates"
            );
        }

        #[test]
        fn sets_suppressed_by_flag_skip_reason_when_suppress_global_set() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let flags = DiscoveryFlags::new(None, None, true).expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let global = advance_to_global_from_env(&config, &ctx);

            assert_eq!(
                global.report.global_resolution_skip_reason,
                Some(GlobalResolutionSkipReason::SuppressedByFlag)
            );
        }

        #[test]
        fn skip_reason_is_none_when_not_suppressed() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let global = advance_to_global_from_env(&config, &ctx);

            assert!(global.report.global_resolution_skip_reason.is_none());
        }
    }

    // -----------------------------------------------------------------------
    // Finalized::finalize
    // -----------------------------------------------------------------------

    mod finalize {
        use super::*;
        use crate::discovery::location::{CacheLocation, GlobalCacheLocation};

        #[test]
        fn returns_empty_vault_when_no_candidates_found() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            let cache: DiscoveryProcessor<CacheResolution> = env.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let result = final_.finalize();

            assert!(result.vault().is_empty());
            assert_eq!(
                result.cache_root().location(),
                &CacheLocation::Global(GlobalCacheLocation::PlatformUserCache)
            );
        }

        #[test]
        fn returns_empty_global_when_no_candidates_found() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            let cache: DiscoveryProcessor<CacheResolution> = env.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let result = final_.finalize();

            assert!(result.global().is_empty());
            assert_eq!(
                result.cache_root().location(),
                &CacheLocation::Global(GlobalCacheLocation::PlatformUserCache)
            );
        }

        #[test]
        fn returns_report_with_filesystem_root_stop_reason() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            let cache: DiscoveryProcessor<CacheResolution> = env.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let report = final_.finalize().report().clone();

            assert_eq!(
                report.local_traversal_stop_reason,
                LocalTraversalStopReason::FilesystemRoot
            );
        }
    }

    // -----------------------------------------------------------------------
    // End-to-end pipeline tests
    // -----------------------------------------------------------------------

    mod pipeline {
        use super::*;

        // --- FlagOverride → Finalized (flag vault + flag config) ---
        // Both paths are known from flags; EnvOverride is never entered.

        #[test]
        fn flag_vault_and_config_vault_is_populated() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            write_marker(vault_dir.path(), "traces.toml");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags = DiscoveryFlags::new(
                Some(config_file.path()),
                Some(vault_dir.path()),
                false,
            )
            .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();
            assert_eq!(flag.flag_branch(), FlagBranch::VaultAndConfigSet);
            let cache: DiscoveryProcessor<CacheResolution> = flag.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let result = final_.finalize();

            assert!(!result.vault().is_empty());
        }

        #[test]
        fn flag_vault_and_config_global_is_empty() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            write_marker(vault_dir.path(), "traces.toml");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags = DiscoveryFlags::new(
                Some(config_file.path()),
                Some(vault_dir.path()),
                false,
            )
            .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();
            let cache: DiscoveryProcessor<CacheResolution> = flag.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let result = final_.finalize();

            assert!(result.global().is_empty());
        }

        #[test]
        fn flag_vault_and_config_skip_reason_is_none() {
            // Pipeline is complete from flags — not suppressed by --no-global.
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            write_marker(vault_dir.path(), "traces.toml");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags = DiscoveryFlags::new(
                Some(config_file.path()),
                Some(vault_dir.path()),
                false,
            )
            .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();
            let cache: DiscoveryProcessor<CacheResolution> = flag.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let report = final_.finalize().report().clone();

            assert!(report.global_resolution_skip_reason.is_none());
        }

        #[test]
        fn flag_vault_and_config_stop_reason_is_explicit_config_file() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags = DiscoveryFlags::new(
                Some(config_file.path()),
                Some(vault_dir.path()),
                false,
            )
            .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();
            let cache: DiscoveryProcessor<CacheResolution> = flag.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let report = final_.finalize().report().clone();

            assert_eq!(
                report.local_traversal_stop_reason,
                LocalTraversalStopReason::ExplicitConfigFile
            );
        }

        // --- FlagOverride → GlobalResolution (flag vault only) ---
        // Vault is known from flag; global still runs; EnvOverride is skipped.

        #[test]
        fn flag_vault_only_vault_is_populated() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            write_marker(vault_dir.path(), "traces.toml");
            let global_dir = tempfile::tempdir().expect("global dir");
            write_marker(global_dir.path(), "traces.toml");

            let config = DiscoveryServiceConfig {
                global_directories: vec![
                    DirPath::try_new(global_dir.path().to_path_buf())
                        .expect("dir"),
                ],
                ..DiscoveryServiceConfig::default()
            };
            let flags =
                DiscoveryFlags::new(None, Some(vault_dir.path()), false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();
            assert_eq!(flag.flag_branch(), FlagBranch::VaultOnlySet);
            let global: DiscoveryProcessor<GlobalResolution> = flag.into();
            let cache: DiscoveryProcessor<CacheResolution> = global.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let result = final_.finalize();

            assert!(!result.vault().is_empty());
        }

        #[test]
        fn flag_vault_only_global_is_populated() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            write_marker(vault_dir.path(), "traces.toml");
            let global_dir = tempfile::tempdir().expect("global dir");
            write_marker(global_dir.path(), "traces.toml");

            let config = DiscoveryServiceConfig {
                global_directories: vec![
                    DirPath::try_new(global_dir.path().to_path_buf())
                        .expect("dir"),
                ],
                ..DiscoveryServiceConfig::default()
            };
            let flags =
                DiscoveryFlags::new(None, Some(vault_dir.path()), false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();
            let global: DiscoveryProcessor<GlobalResolution> = flag.into();
            let cache: DiscoveryProcessor<CacheResolution> = global.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let result = final_.finalize();

            assert!(!result.global().is_empty());
        }

        #[test]
        fn flag_vault_only_skip_reason_is_none() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");

            let config = default_config();
            let flags =
                DiscoveryFlags::new(None, Some(vault_dir.path()), false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();
            let global: DiscoveryProcessor<GlobalResolution> = flag.into();
            let cache: DiscoveryProcessor<CacheResolution> = global.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let report = final_.finalize().report().clone();

            assert!(report.global_resolution_skip_reason.is_none());
        }

        // --- ConsultEnv → AscendSkipGlobal (flag config only; no flag vault)
        // ---

        #[test]
        fn flag_config_only_branch_strategy_is_ascend_then_global() {
            // With only a flag config (no flag vault, no env overrides),
            // branch_strategy checks env vars only and finds none →
            // AscendThenGlobal. The stop reason is still ExplicitConfigFile
            // (set during FlagOverride → EnvOverride). Global resolution
            // still runs because no vault is known.
            let root = tempfile::tempdir().expect("root");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags =
                DiscoveryFlags::new(Some(config_file.path()), None, false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();
            assert_eq!(flag.flag_branch(), FlagBranch::ConsultEnv);
            let env_p: DiscoveryProcessor<EnvOverride> = flag.into();

            assert_eq!(
                env_p.branch_strategy(),
                ExplicitOverrideBranch::AscendThenGlobal
            );
        }

        #[test]
        fn flag_config_only_stop_reason_is_explicit_config_file() {
            let root = tempfile::tempdir().expect("root");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags =
                DiscoveryFlags::new(Some(config_file.path()), None, false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();
            let env_p: DiscoveryProcessor<EnvOverride> = flag.into();
            let ascend: DiscoveryProcessor<AscendingTraversal> =
                env_p.try_into().expect("ascend");
            let cache: DiscoveryProcessor<CacheResolution> = ascend.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let report = final_.finalize().report().clone();

            assert_eq!(
                report.local_traversal_stop_reason,
                LocalTraversalStopReason::ExplicitConfigFile
            );
        }

        // --- ConsultEnv → AscendSkipGlobal (env config only; no flag
        // overrides) ---

        #[test]
        fn env_config_only_ascend_skip_global_global_is_empty() {
            let root = tempfile::tempdir().expect("root");
            write_marker(root.path(), "traces.toml");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let env =
                DiscoveryEnv::new(Some(config_file.path()), None, None, None)
                    .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();
            assert_eq!(flag.flag_branch(), FlagBranch::ConsultEnv);
            let env_p: DiscoveryProcessor<EnvOverride> = flag.into();
            assert_eq!(
                env_p.branch_strategy(),
                ExplicitOverrideBranch::AscendSkipGlobal
            );
            let ascend: DiscoveryProcessor<AscendingTraversal> =
                env_p.try_into().expect("ascend");
            let cache: DiscoveryProcessor<CacheResolution> = ascend.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let result = final_.finalize();

            assert!(result.global().is_empty());
        }

        #[test]
        fn env_config_only_ascend_skip_global_stop_reason_is_explicit_config_file()
         {
            let root = tempfile::tempdir().expect("root");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let env =
                DiscoveryEnv::new(Some(config_file.path()), None, None, None)
                    .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let flag: DiscoveryProcessor<FlagOverride> =
                DiscoveryProcessor::new(&config, &ctx).into();
            let env_p: DiscoveryProcessor<EnvOverride> = flag.into();
            let ascend: DiscoveryProcessor<AscendingTraversal> =
                env_p.try_into().expect("ascend");
            let cache: DiscoveryProcessor<CacheResolution> = ascend.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let report = final_.finalize().report().clone();

            assert_eq!(
                report.local_traversal_stop_reason,
                LocalTraversalStopReason::ExplicitConfigFile
            );
        }

        // --- AscendThenGlobal ---

        #[test]
        fn ascend_then_global_vault_is_populated() {
            let root = tempfile::tempdir().expect("root");
            write_marker(root.path(), "traces.toml");
            let global_dir = tempfile::tempdir().expect("global dir");
            write_marker(global_dir.path(), "traces.toml");

            let config = DiscoveryServiceConfig {
                global_directories: vec![
                    DirPath::try_new(global_dir.path().to_path_buf())
                        .expect("dir"),
                ],
                ..DiscoveryServiceConfig::default()
            };
            let ctx = make_context(root.path()).expect("ctx");

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            assert_eq!(
                env.branch_strategy(),
                ExplicitOverrideBranch::AscendThenGlobal
            );
            let ascend: DiscoveryProcessor<AscendingTraversal> =
                env.try_into().expect("ascend");
            let global: DiscoveryProcessor<GlobalResolution> = ascend.into();
            let cache: DiscoveryProcessor<CacheResolution> = global.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let result = final_.finalize();

            assert!(!result.vault().is_empty());
        }

        #[test]
        fn ascend_then_global_global_is_populated() {
            let root = tempfile::tempdir().expect("root");
            write_marker(root.path(), "traces.toml");
            let global_dir = tempfile::tempdir().expect("global dir");
            write_marker(global_dir.path(), "traces.toml");

            let config = DiscoveryServiceConfig {
                global_directories: vec![
                    DirPath::try_new(global_dir.path().to_path_buf())
                        .expect("dir"),
                ],
                ..DiscoveryServiceConfig::default()
            };
            let ctx = make_context(root.path()).expect("ctx");

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            let ascend: DiscoveryProcessor<AscendingTraversal> =
                env.try_into().expect("ascend");
            let global: DiscoveryProcessor<GlobalResolution> = ascend.into();
            let cache: DiscoveryProcessor<CacheResolution> = global.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let result = final_.finalize();

            assert!(!result.global().is_empty());
        }

        #[test]
        fn ascend_then_global_skip_reason_is_none() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            let ascend: DiscoveryProcessor<AscendingTraversal> =
                env.try_into().expect("ascend");
            let global: DiscoveryProcessor<GlobalResolution> = ascend.into();
            let cache: DiscoveryProcessor<CacheResolution> = global.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let report = final_.finalize().report().clone();

            assert!(report.global_resolution_skip_reason.is_none());
        }

        // --- suppress_global correctness ---

        #[test]
        fn suppress_global_sets_report_skip_reason() {
            let root = tempfile::tempdir().expect("root");
            let global_dir = tempfile::tempdir().expect("global dir");
            write_marker(global_dir.path(), "traces.toml");

            let config = DiscoveryServiceConfig {
                global_directories: vec![
                    DirPath::try_new(global_dir.path().to_path_buf())
                        .expect("dir"),
                ],
                ..DiscoveryServiceConfig::default()
            };
            let flags = DiscoveryFlags::new(None, None, true).expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            let global: DiscoveryProcessor<GlobalResolution> = env.into();
            let cache: DiscoveryProcessor<CacheResolution> = global.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let report = final_.finalize().report().clone();

            assert_eq!(
                report.global_resolution_skip_reason,
                Some(GlobalResolutionSkipReason::SuppressedByFlag)
            );
        }

        #[test]
        fn suppress_global_produces_empty_global_candidates() {
            let root = tempfile::tempdir().expect("root");
            let global_dir = tempfile::tempdir().expect("global dir");
            write_marker(global_dir.path(), "traces.toml");

            let config = DiscoveryServiceConfig {
                global_directories: vec![
                    DirPath::try_new(global_dir.path().to_path_buf())
                        .expect("dir"),
                ],
                ..DiscoveryServiceConfig::default()
            };
            let flags = DiscoveryFlags::new(None, None, true).expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            let global: DiscoveryProcessor<GlobalResolution> = env.into();
            let cache: DiscoveryProcessor<CacheResolution> = global.into();
            let final_: DiscoveryProcessor<Finalized> = cache.into();
            let result = final_.finalize();

            assert!(
                result.global().is_empty(),
                "suppressed global must yield no candidates even when global \
                 directories contain markers"
            );
        }
    }

    // -----------------------------------------------------------------------
    // CacheResolution phase
    // -----------------------------------------------------------------------

    mod cache_resolution {
        use std::path::PathBuf;

        use super::*;
        use crate::discovery::location::{
            CacheLocation, GlobalCacheLocation, LocalCacheLocation,
        };

        /// Helper: advance processor through `FlagOverride` → `EnvOverride` →
        /// `CacheResolution`. Used when a vault needs to be probed (env vault
        /// dir or flag vault dir).
        fn advance_to_cache_resolution_with_vault<'a>(
            config: &'a DiscoveryServiceConfig,
            ctx: &'a DiscoveryContext<'a>,
        ) -> DiscoveryProcessor<'a, CacheResolution> {
            let init = DiscoveryProcessor::new(config, ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            // EnvOverride → CacheResolution (vault already probed in
            // EnvOverride)
            env.into()
        }

        /// Bypasses the production pipeline (`Init` → `FlagOverride` → … →
        /// `CacheResolution`) and goes directly `Init` → `CacheResolution`
        /// using the `#[cfg(test)]`-only `From<Init>` impl. This is
        /// intentional: these tests exercise `resolve_cache_root` in
        /// isolation with controlled `DiscoveryEnv` inputs,
        /// not the full pipeline routing. Use
        /// `advance_to_cache_resolution_with_vault` when vault probing
        /// must occur before cache resolution.
        fn advance_to_cache_resolution<'a>(
            config: &'a DiscoveryServiceConfig,
            ctx: &'a DiscoveryContext<'a>,
        ) -> DiscoveryProcessor<'a, CacheResolution> {
            DiscoveryProcessor::new(config, ctx).into()
        }

        #[test]
        fn returns_environment_override_when_cache_dir_env_is_set() {
            let root = tempfile::tempdir().expect("root");
            let cache_path = PathBuf::from("/custom/cache");
            let config = default_config();
            let env =
                DiscoveryEnv::new(None, None, None, Some(cache_path.clone()))
                    .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let p = advance_to_cache_resolution(&config, &ctx);

            let cache_root = p.cache_root.expect("cache_root must be set");
            assert_eq!(
                cache_root.location(),
                &CacheLocation::Global(
                    GlobalCacheLocation::EnvironmentOverride
                )
            );
            assert_eq!(cache_root.path(), cache_path.as_path());
        }

        #[test]
        fn returns_environment_override_regardless_of_vault_presence() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            write_marker(vault_dir.path(), "traces.toml");
            let cache_path = PathBuf::from("/env/cache");

            let config = default_config();
            let env = DiscoveryEnv::new(
                None,
                Some(vault_dir.path()),
                None,
                Some(cache_path.clone()),
            )
            .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let p = advance_to_cache_resolution_with_vault(&config, &ctx);

            let cache_root = p.cache_root.expect("cache_root must be set");
            assert_eq!(
                cache_root.location(),
                &CacheLocation::Global(
                    GlobalCacheLocation::EnvironmentOverride
                ),
                "env override wins even when vault is present"
            );
        }

        #[test]
        fn returns_project_cache_directory_when_vault_present_and_no_env() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            write_marker(vault_dir.path(), "traces.toml");

            let config = default_config();
            let env =
                DiscoveryEnv::new(None, Some(vault_dir.path()), None, None)
                    .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let p = advance_to_cache_resolution_with_vault(&config, &ctx);

            let cache_root = p.cache_root.expect("cache_root must be set");
            assert_eq!(
                cache_root.location(),
                &CacheLocation::Local(
                    LocalCacheLocation::ProjectCacheDirectory
                )
            );
            assert!(
                cache_root.path().ends_with(".traces/cache"),
                "path should end with .traces/cache, got: {:?}",
                cache_root.path()
            );
        }

        #[test]
        fn returns_platform_user_cache_when_no_vault_and_no_env() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let p = advance_to_cache_resolution(&config, &ctx);

            let cache_root = p.cache_root.expect("cache_root must be set");
            assert_eq!(
                cache_root.location(),
                &CacheLocation::Global(GlobalCacheLocation::PlatformUserCache)
            );
        }

        #[test]
        fn path_is_not_validated_for_existence_at_construction() {
            let root = tempfile::tempdir().expect("root");
            let cache_path =
                PathBuf::from("/absolutely/nonexistent/cache/path");
            let config = default_config();
            let env =
                DiscoveryEnv::new(None, None, None, Some(cache_path.clone()))
                    .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let p = advance_to_cache_resolution(&config, &ctx);

            let cache_root = p.cache_root.expect("cache_root must be set");
            assert_eq!(cache_root.path(), cache_path.as_path());
        }
    }
}
