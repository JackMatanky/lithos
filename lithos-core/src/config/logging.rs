//! Logging configuration types.
//!
//! This module contains types related to logging configuration,
//! including log levels and logging settings.

#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv::Archive derive generates exhaustive archived enums"
)]

use super::error::ConfigError;

/// Raw logging configuration (unvalidated input from config files).
///
/// This is a serde-only DTO that accepts flexible input from TOML/YAML/JSON.
/// The log level is a string that will be validated during conversion to
/// [`Logging`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawLogging {
    /// Logging verbosity level.
    pub log_level: Option<String>,
}

/// Logging verbosity level.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum LogLevel {
    /// Error-level logging only.
    Error,
    /// Warning and error logging.
    Warn,
    /// Informational logging.
    #[default]
    Info,
    /// Debug logging.
    Debug,
    /// Trace-level logging.
    Trace,
}

/// Logging configuration with validation.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct Logging {
    /// Log level (debug, info, warn, error).
    log_level: LogLevel,
}

impl LogLevel {
    #[inline]
    #[must_use]
    /// Return the lowercase string form.
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

impl TryFrom<String> for LogLevel {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, ConfigError> {
        match value.as_str() {
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err(ConfigError::InvalidEnumValue {
                field: "log_level".to_owned().into(),
                value: value.into(),
                allowed: ["error", "warn", "info", "debug", "trace"]
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
            }),
        }
    }
}

impl From<LogLevel> for String {
    #[inline]
    fn from(value: LogLevel) -> Self {
        value.as_str().to_owned()
    }
}

impl Default for Logging {
    #[inline]
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
        }
    }
}

impl Logging {
    #[inline]
    #[must_use]
    /// Create logging configuration.
    pub fn new(log_level: LogLevel) -> Self {
        Self {
            log_level,
        }
    }

    #[inline]
    #[must_use]
    /// Return the log level.
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    #[inline]
    #[must_use]
    /// Return the log level as a string.
    pub fn log_level_str(&self) -> &'static str {
        self.log_level.as_str()
    }
}

impl TryFrom<RawLogging> for Logging {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawLogging) -> Result<Self, Self::Error> {
        match raw.log_level {
            Some(value) => Ok(Logging::new(LogLevel::try_from(value)?)),
            None => Ok(Logging::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        use super::*;

        pub fn sample_logging() -> Logging {
            Logging::new(LogLevel::Debug)
        }
    }

    /// 3.3-UNIT-027: `logging_rejects_invalid_levels`.
    /// Priority: P0.
    #[test]
    fn logging_rejects_invalid_levels() {
        let result = LogLevel::try_from("verbose".to_owned());
        assert!(result.is_err(), "Expected validation error");
    }

    #[test]
    fn logging_accepts_valid_levels() {
        let logging = fixtures::sample_logging();
        assert_eq!(logging.log_level_str(), "debug");
    }
}
