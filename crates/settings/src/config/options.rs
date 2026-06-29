//! Configuration builder options.

/// Options for the configuration builder.
#[derive(Debug, Clone, Default)]
pub struct ConfigBuilderOptions {
    /// Trust mode (e.g. paranoid).
    pub trust_mode: bool,
    /// Auto-confirm prompts.
    pub auto_confirm: bool,
}
