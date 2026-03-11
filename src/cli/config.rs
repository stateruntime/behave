//! Configuration file parsing for `behave.toml`.

use std::path::Path;

use serde::Deserialize;

use super::error::CliError;

/// Default path for the test history file.
const DEFAULT_HISTORY_FILE: &str = ".behave/history.json";

/// Default number of consecutive passes before flagging as flaky.
const DEFAULT_CONSECUTIVE_PASSES: u32 = 5;

/// Top-level configuration parsed from `behave.toml`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::config::BehaveConfig;
///
/// let config = BehaveConfig::default();
/// assert!(config.flaky_detection.is_none());
/// # }
/// ```
#[derive(Debug, Deserialize, Default)]
#[non_exhaustive]
pub struct BehaveConfig {
    /// Flaky test detection settings.
    pub flaky_detection: Option<FlakyDetectionConfig>,
}

/// Configuration for flaky test detection.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::config::FlakyDetectionConfig;
///
/// let config = FlakyDetectionConfig::default();
/// assert!(config.enabled);
/// # }
/// ```
#[derive(Debug, Deserialize)]
#[non_exhaustive]
pub struct FlakyDetectionConfig {
    /// Whether flaky detection is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Path to the history file.
    #[serde(default = "default_history_file")]
    pub history_file: String,
    /// Number of consecutive passes before a failure is considered flaky.
    #[serde(default = "default_consecutive_passes")]
    pub consecutive_passes: u32,
}

impl Default for FlakyDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            history_file: default_history_file(),
            consecutive_passes: default_consecutive_passes(),
        }
    }
}

const fn default_enabled() -> bool {
    true
}

fn default_history_file() -> String {
    DEFAULT_HISTORY_FILE.to_string()
}

const fn default_consecutive_passes() -> u32 {
    DEFAULT_CONSECUTIVE_PASSES
}

/// Loads configuration from a `behave.toml` file in the given directory.
///
/// Returns the default config if no file exists.
///
/// # Errors
///
/// Returns [`CliError::ConfigParse`] if the file exists but cannot be parsed.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::config::load_config;
///
/// let config = load_config(std::path::Path::new("."));
/// # }
/// ```
pub fn load_config(dir: &Path) -> Result<BehaveConfig, CliError> {
    let path = dir.join("behave.toml");

    if !path.exists() {
        return Ok(BehaveConfig::default());
    }

    let contents = std::fs::read_to_string(&path).map_err(|source| CliError::Io { source })?;
    toml::from_str(&contents).map_err(|err| CliError::ConfigParse {
        message: err.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_no_flaky_detection() {
        let config = BehaveConfig::default();
        assert!(config.flaky_detection.is_none());
    }

    #[test]
    fn default_flaky_config() {
        let config = FlakyDetectionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.history_file, ".behave/history.json");
        assert_eq!(config.consecutive_passes, 5);
    }

    #[test]
    fn parse_minimal_toml() {
        let toml_str = "[flaky_detection]\nenabled = true\n";
        let config: BehaveConfig = toml::from_str(toml_str).unwrap_or_default();
        assert!(config.flaky_detection.is_some());
    }

    #[test]
    fn parse_full_toml() {
        let toml_str = r#"
[flaky_detection]
enabled = true
history_file = ".custom/history.json"
consecutive_passes = 10
"#;
        let config: BehaveConfig = toml::from_str(toml_str).unwrap_or_default();
        let flaky = config.flaky_detection.as_ref();
        assert!(flaky.is_some());
        if let Some(f) = flaky {
            assert!(f.enabled);
            assert_eq!(f.history_file, ".custom/history.json");
            assert_eq!(f.consecutive_passes, 10);
        }
    }

    #[test]
    fn load_config_missing_file() {
        let config = load_config(Path::new("/nonexistent/path"));
        assert!(config.is_ok());
    }
}
