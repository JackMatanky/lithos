---
title: 19-env-dirs-centralization
category: enhancement
label: planned
status: draft
date_created: 2026-06-21
---

# Env/Dirs Centralization — `EnvVars`, Platform XDG Statics, and `AppDirs`

Centralise all environment variable reading and platform directory resolution
into `env.rs` and `dirs.rs` at the crate root (`lithos-core/src/`), replacing
ad-hoc `std::env` calls scattered across `bootstrap.rs`, `processor.rs`, and
`DiscoveryEnv`.

## Motivation

Current state (before this ADR was implemented):
- `env.rs` had a mix of top-level `LazyLock` statics and loose env reads; no
  struct for capturing `LITHOS_*` vars together.
- `bootstrap.rs` read `LITHOS_CACHE_DIR` via `env::var_os()` directly (two
  call sites) and owned `platform_global_directory_candidates()` which read
  `XDG_CONFIG_HOME`, `HOME`, `APPDATA` directly.
- `processor.rs` called `dirs::cache_dir()` directly at the third fallback
  level in `resolve_cache_root()`.
- `DiscoveryEnv` received raw path params — no shared "these are the captured
  env vars" type.

## Architecture — Two-Layer + Platform Statics

```
EnvVars (LITHOS_* only, pure capture from env, no fallbacks)
   │
   ├── config_file / vault_dir → DiscoveryEnv::from_env()
   ├── cache_dir → AppDirs::new()
   ├── ceiling_dirs → (consumed directly from EnvVars by callers)
   └── suppress_global → DiscoveryFlags

Platform XDG statics (HOME, XDG_CACHE/XDG_CONFIG/XDG_DATA/XDG_STATE)
   │
   └── AppDirs::new()  (merge with EnvVars overrides + "lithos" suffix)
```

### Layer 1: `EnvVars` — "what the user set" (`env.rs`)

Pure capture struct. Only `LITHOS_*` vars. No HOME, no XDG_*. No fallbacks, no
platform branching, no filesystem validation.

```rust
pub struct EnvVars {
    vault_dir: Option<PathBuf>,
    config_file: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    ceiling_dirs: Option<Vec<PathBuf>>,
    suppress_global: bool,
}

impl EnvVars {
    pub fn capture() -> Self { ... }
    pub fn new(...) -> Self { ... }

    pub fn vault_dir(&self) -> Option<&PathBuf> { ... }
    pub fn config_file(&self) -> Option<&PathBuf> { ... }
    pub fn cache_dir(&self) -> Option<&PathBuf> { ... }
    pub fn ceiling_dirs(&self) -> Option<&[PathBuf]> { ... }
    pub fn suppress_global(&self) -> bool { ... }
}
```

**Consumers:**
- `vault_dir` → `DiscoveryEnv::from_env()`
- `config_file` → `DiscoveryEnv::from_env()`
- `cache_dir` → `AppDirs::new()`
- `ceiling_dirs` → consumed directly by callers (not stored in `DiscoveryEnv`
  because `DiscoveryEnv` borrows the raw `&OsStr` which `EnvVars` already parsed)
- `suppress_global` → passed alongside `DiscoveryFlags`

### Platform XDG Statics — mise-style lazy statics (`env.rs`)

Instead of a `PlatformDirs` struct, the implementation follows **mise
convention**: platform-specific `#[cfg]` lazy statics at module level, each
with per-platform (macOS / Windows / other) separate definitions.

```rust
// macOS
#[cfg(target_os = "macos")]
pub static XDG_CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CONFIG_HOME")
        .unwrap_or_else(|| HOME.join("Library/Application Support"))
});

// Windows
#[cfg(windows)]
pub static XDG_CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CONFIG_HOME")
        .or_else(|| var_path("APPDATA"))
        .unwrap_or_else(|| HOME.join("AppData/Roaming"))
});

// Other (Linux, BSD, etc.)
#[cfg(not(any(target_os = "macos", windows)))]
pub static XDG_CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CONFIG_HOME").unwrap_or_else(|| HOME.join(".config"))
});
```

Five statics defined this way: `HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`,
`XDG_DATA_HOME`, `XDG_STATE_HOME`. Each has exactly three `#[cfg]` blocks
(macOS / Windows / other) — no shared helpers, no `cfg_aliases` crate.

For platform fallback conventions, see `env.rs` source.

**Why statics over `PlatformDirs` struct:**
- Matches mise's approach (`~/.local/share/mise`, `~/.cache/mise`)
- Simpler API — no struct to construct/borrow, callers import the static they
  need directly
- `LazyLock` initializes once, avoids repeated `std::env::var` calls
- Test mode redirects `HOME` to a fixture directory via `#[cfg(test)]`

### Layer 3: `AppDirs` — resolved lithos directories (`dirs.rs`)

Thin merge layer. Takes `&EnvVars`, reads XDG statics from `crate::env`.
Produces the final application-level paths that lithos uses for cache, global
config, etc.

```rust
pub struct AppDirs {
    cache: PathBuf,         // vars.cache_dir ?→ XDG_CACHE_HOME / "lithos"
    config: PathBuf,        // XDG_CONFIG_HOME / "lithos"
    system_config: Option<PathBuf>, // /etc/lithos (unix), None (win)
}

impl AppDirs {
    pub fn new(vars: &EnvVars) -> Self { ... }

    pub fn cache(&self) -> &PathBuf { &self.cache }
    pub fn config(&self) -> &PathBuf { &self.config }
    pub fn system_config(&self) -> Option<&PathBuf> { ... }
}
```

Vault root is **not** in `AppDirs`. The vault root is resolved by the discovery
layer (from `LITHOS_VAULT_DIR`, CLI `--vault`, or ascending walk). It is a
discovery concern, not a platform-dir concern.

## What changed

### `env.rs`

- Added `EnvVars` struct with `capture()`, `new()`, and typed accessors.
- Renamed from `envs.rs` (was briefly `envs` for consistency with `context.rs`,
  `processor.rs` naming — reverted).
- Moved `HOME` + all four XDG statics into `env.rs` from `dirs.rs`.
- Each XDG static has explicit per-platform cfg blocks (macOS / Windows / other)
  with appropriate native fallbacks.
- Test `HOME` redirects to `<CARGO_MANIFEST_DIR>/tests/fixtures/`.
- Added `var_is_true` helper for boolean env vars (`LITHOS_SUPPRESS_GLOBAL`).

### `dirs.rs`

- Removed `HOME`, XDG statics, `var_path` helper — all moved to `env.rs`.
- `AppDirs::new()` now takes only `&EnvVars` (reads XDG statics from
  `crate::env` internally).
- `platform_system_config()` stays in `dirs.rs` (unix vs windows logic).

### `bootstrap.rs`

- `platform_global_directory_candidates()` and
  `platform_global_directories()` deleted → caller uses `AppDirs::config()`
  and `AppDirs::system_config()` directly.
- `env::var_os("LITHOS_CACHE_DIR")` → `EnvVars::capture().cache_dir()`
- `build_context()` cache_dir param stays, sourced from `EnvVars::capture()`

### `processor.rs`

- `resolve_cache_root()` third fallback uses `crate::env::XDG_CACHE_HOME`
  static instead of inline platform chain.

### `DiscoveryEnv` (context.rs)

- Constructor still takes individual params (config_file, vault_dir, etc.),
  plus a new `from_env(&EnvVars)` convenience that extracts `config_file`,
  `vault_dir`, and `cache_dir`. `ceiling_dirs_raw` is set to `None` (not
  reconstructable from the already-parsed `EnvVars`).

## What stays in `DiscoveryEnv`

- Filesystem validation (does `config_file` exist? is `vault_dir` a directory?)
- Ceiling dir raw storage (still `&OsStr` — split during traversal setup)
- Cache_dir passthrough (unvalidated, optional)

## Out of scope

- Changing `DiscoveryEnv` validation logic.
- Adding new `LITHOS_*` env vars.
- Removing `DiscoveryEnv` entirely — it still owns ceiling parsing and
  path validation.
- Vault root resolution — vault stays a discovery concept.
- `DiscoveryFlags::from_env()` — suppress_global is passed alongside flags
  at the call site; no dedicated constructor needed.
