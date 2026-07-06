## Agent Brief

**Category:** enhancement
**Summary:** Migrate Settings to traces-core

**Current behavior:**
The `traces-settings` crate defines the application's configuration logic, including `SettingsService`, `AppConfig`, config types (`VaultConfig`, `GlobalConfig`), discovery logic, and tracking/trust behavior. Because it's a separate crate, downstream contexts (like `traces-note`, `traces-schema`, `traces-template`, etc.) must depend on it directly to access their configurations.

**Desired behavior:**
The `traces-settings` crate should be entirely moved into `traces-core::settings`. All downstream consumers should update their imports to point to `traces_core::settings` instead of `traces_settings`. The original `traces-settings` crate should be deleted.

**Key interfaces:**
- `traces-settings` crate contents move to `traces-core::settings`.
- `SettingsService`, `AppConfig`, `Builder`, discovery structures (`DiscoveryService`, etc.), config structs, and trust/tracker logic are all relocated. No changes to the actual configuration API or behavior are required; this is purely a relocation.
- `Cargo.toml` dependencies in all downstream crates must be updated: remove `traces-settings` and depend on `traces-core` (if they don't already).

**Acceptance criteria:**
- [ ] The `traces-settings` crate is completely removed from the workspace.
- [ ] `SettingsService`, `AppConfig`, and related types live in `traces-core::settings`.
- [ ] All downstream workspace members correctly depend on `traces-core` for settings.
- [ ] The entire workspace compiles and all tests pass with the new dependency structure.

**Out of scope:**
- Modifying the actual settings resolution logic or the shape of the config structs.
- Changing the `RedbRepository` logic for settings storage (beyond moving it).
