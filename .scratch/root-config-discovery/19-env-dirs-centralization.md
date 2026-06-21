---
title: 19-env-dirs-centralization
category: enhancement
label: planned
status: draft
date_created: 2026-06-21
---

# Env/Dirs Centralization — `EnvVars`, `PlatformDirs`, and `AppDirs`

Centralise all environment variable reading and directory resolution into three
structs at the crate root (`lithos-core/src/`), replacing ad-hoc `std::env` calls
scattered across `bootstrap.rs`, `processor.rs`, and `DiscoveryEnv`.

## Motivation

Current state:
- `env.rs` has 7 top-level `LazyLock` statics mixing platform dirs (`HOME`,
  `XDG_CACHE_HOME`, `XDG_CONFIG_HOME`) with LITHOS_* env vars. No struct
  abstraction, no test-only construction without env mutation.
- `bootstrap.rs` reads `LITHOS_CACHE_DIR` via `env::var_os()` directly (two
  call sites) and owns `platform_global_directory_candidates()` which reads
  `XDG_CONFIG_HOME`, `HOME`, `APPDATA` directly.
- `processor.rs` calls `dirs::cache_dir()` directly at the third fallback
  level in `resolve_cache_root()`.
- `DiscoveryEnv` in `context.rs` receives raw path params — there's no shared
  "these are the captured env vars" type.

## Architecture — Three-Layer Split

```
EnvVars (LITHOS_* only, pure capture from env, no fallbacks)
     │
PlatformDirs (HOME + XDG_* + dirs crate, platform-native defaults)
     │
     ▼
AppDirs (merge: PlatformDirs + "lithos" suffix + EnvVars overrides)
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

    pub fn vault_dir(&self) -> Option<&PathBuf> { self.vault_dir.as_ref() }
    pub fn config_file(&self) -> Option<&PathBuf> { self.config_file.as_ref() }
    pub fn cache_dir(&self) -> Option<&PathBuf> { self.cache_dir.as_ref() }
    pub fn ceiling_dirs(&self) -> Option<&[PathBuf]> { self.ceiling_dirs.as_deref() }
    pub fn suppress_global(&self) -> bool { self.suppress_global }
}
```

**Consumers:**
- `vault_dir` → `DiscoveryEnv::from_env()` or passed directly
- `config_file` → `DiscoveryEnv::from_env()`
- `cache_dir` → `AppDirs::new()`
- `ceiling_dirs` → `DiscoveryEnv::from_env()`
- `suppress_global` → `DiscoveryFlags::from_env()`

### Layer 2: `PlatformDirs` — OS-native directory defaults (`dirs.rs`)

Reads HOME + XDG_* + calls `dirs` crate. Owns platform-specific path logic
(macOS, Linux, Windows). No `"lithos"` suffix applied.

```rust
pub struct PlatformDirs {
    config: PathBuf,   // Linux: XDG_CONFIG_HOME → ~/.config
                       // macOS: ~/Library/Application Support
                       // Win:   %APPDATA% (Roaming)
    cache: PathBuf,    // Linux: XDG_CACHE_HOME → ~/.cache
                       // macOS: ~/Library/Caches
                       // Win:   %LOCALAPPDATA%
}

impl PlatformDirs {
    pub fn capture() -> Self { ... }
    pub fn new(config: PathBuf, cache: PathBuf) -> Self { ... }

    pub fn config(&self) -> &PathBuf { &self.config }
    pub fn cache(&self) -> &PathBuf { &self.cache }
}
```

Design notes:
- XDG_* vars checked on **all platforms**, not just Linux.
- `dirs` crate v6 handles the platform-native fallbacks; XDG_* checked first.
- `PlatformDirs::capture()` reads env directly. `PlatformDirs::new()` takes
  explicit paths for testing.

### Layer 3: `AppDirs` — resolved lithos directories (`dirs.rs`)

Thin merge layer. Takes `(&EnvVars, &PlatformDirs)`. Produces the final
application-level paths that lithos uses for cache, global config, etc.

```rust
pub struct AppDirs {
    cache: PathBuf,         // env.cache_dir ?→ platform.cache / "lithos"
    config: PathBuf,        // platform.config / "lithos" (global config probe dir)
    system_config: Option<PathBuf>, // /etc/lithos (unix), None (win)
}

impl AppDirs {
    pub fn new(vars: &EnvVars, platform: &PlatformDirs) -> Self { ... }

    pub fn cache(&self) -> &PathBuf { &self.cache }
    pub fn config(&self) -> &PathBuf { &self.config }
    pub fn system_config(&self) -> Option<&PathBuf> { self.system_config.as_ref() }
}
```

Vault root is **not** in `AppDirs`. The vault root is resolved by the discovery
layer (from `LITHOS_VAULT_DIR`, CLI `--vault`, or ascending walk). It is a
discovery concern, not a platform-dir concern.

## Impact on existing code

### `bootstrap.rs`

- `platform_global_directory_candidates()` and
  `platform_global_directories()` deleted → caller uses `AppDirs::config()`
  and `AppDirs::system_config()` directly to build probe dirs.
- `env::var_os("LITHOS_CACHE_DIR")` → `EnvVars::capture().cache_dir()`
- `build_context()` cache_dir param stays (passthrough to `DiscoveryEnv`),
  but its source becomes `EnvVars::capture().cache_dir()`

### `processor.rs`

- `resolve_cache_root()` third fallback `dirs::cache_dir().join("lithos")` →
  passed `AppDirs::cache` or computed from `PlatformDirs::cache`

### `DiscoveryEnv` (context.rs)

- Constructor still takes individual params (config_file, vault_dir, etc.) but
  adds a `from_env(&EnvVars)` convenience that extracts them.
- `LITHOS_CACHE_DIR` source changes from `env::var_os` in bootstrap.rs to
  `EnvVars::capture().cache_dir`.

### `env.rs` statics

- `HOME`, `XDG_CACHE_HOME`, `XDG_CONFIG_HOME` statics removed → part of
  `PlatformDirs`.
- `LITHOS_*` statics removed → part of `EnvVars` struct.
- No more top-level `LazyLock` statics — `EnvVars::capture()` is the one
  env-read seam.

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
