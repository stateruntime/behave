//! Test history tracking and flaky test detection.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::config::FlakyDetectionConfig;
use super::error::CliError;
use super::parser::{TestOutcome, TestResult};

/// Current history file format version.
const HISTORY_VERSION: u32 = 1;

/// Persistent test history for flaky detection.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::history::TestHistory;
///
/// let history = TestHistory::new();
/// assert!(history.tests.is_empty());
/// # }
/// ```
#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TestHistory {
    /// Format version number.
    pub version: u32,
    /// Per-test historical data keyed by full test name.
    pub tests: HashMap<String, TestEntry>,
}

/// Historical data for a single test.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::history::TestEntry;
///
/// let entry = TestEntry::new();
/// assert_eq!(entry.consecutive_passes, 0);
/// # }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct TestEntry {
    /// Number of consecutive passes before the last outcome.
    pub consecutive_passes: u32,
    /// The last recorded outcome (`"pass"`, `"fail"`, `"ignored"`).
    pub last_outcome: String,
    /// Hash of the source file content at last update.
    pub source_hash: String,
}

impl TestEntry {
    /// Creates a new empty test entry.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "cli")]
    /// # {
    /// use behave::cli::history::TestEntry;
    ///
    /// let entry = TestEntry::new();
    /// assert_eq!(entry.consecutive_passes, 0);
    /// # }
    /// ```
    pub const fn new() -> Self {
        Self {
            consecutive_passes: 0,
            last_outcome: String::new(),
            source_hash: String::new(),
        }
    }
}

impl Default for TestEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl TestHistory {
    /// Creates a new empty test history.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "cli")]
    /// # {
    /// use behave::cli::history::TestHistory;
    ///
    /// let history = TestHistory::new();
    /// assert_eq!(history.version, 1);
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            version: HISTORY_VERSION,
            tests: HashMap::new(),
        }
    }
}

impl Default for TestHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// A test detected as flaky.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::history::FlakyTest;
///
/// let flaky = FlakyTest::new("math::add".to_string(), 12);
/// assert_eq!(flaky.name, "math::add");
/// # }
/// ```
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct FlakyTest {
    /// The full test name.
    pub name: String,
    /// Number of consecutive passes before the failure.
    pub consecutive_passes: u32,
}

impl FlakyTest {
    /// Creates a new flaky test record.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "cli")]
    /// # {
    /// use behave::cli::history::FlakyTest;
    ///
    /// let flaky = FlakyTest::new("test::a".to_string(), 5);
    /// assert_eq!(flaky.consecutive_passes, 5);
    /// # }
    /// ```
    pub const fn new(name: String, consecutive_passes: u32) -> Self {
        Self {
            name,
            consecutive_passes,
        }
    }
}

/// Loads test history from the given file path.
///
/// Returns a fresh history if the file does not exist.
///
/// # Errors
///
/// Returns [`CliError::HistoryIo`] if the file exists but cannot be read or parsed.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::history::load_history;
///
/// let history = load_history(std::path::Path::new(".behave/history.json"));
/// # }
/// ```
pub fn load_history(path: &Path) -> Result<TestHistory, CliError> {
    if !path.exists() {
        return Ok(TestHistory::new());
    }

    let contents =
        std::fs::read_to_string(path).map_err(|source| CliError::HistoryIo { source })?;

    serde_json::from_str(&contents).map_err(|err| CliError::HistoryIo {
        source: std::io::Error::other(err.to_string()),
    })
}

/// Saves test history to the given file path.
///
/// Creates parent directories if they do not exist.
///
/// # Errors
///
/// Returns [`CliError::HistoryIo`] if writing fails.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::history::{TestHistory, save_history};
///
/// save_history(std::path::Path::new(".behave/history.json"), &TestHistory::new());
/// # }
/// ```
pub fn save_history(path: &Path, history: &TestHistory) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| CliError::HistoryIo { source })?;
    }

    let json = serde_json::to_string_pretty(history).map_err(|err| CliError::HistoryIo {
        source: std::io::Error::other(err.to_string()),
    })?;

    std::fs::write(path, json).map_err(|source| CliError::HistoryIo { source })
}

/// Computes a hash string for the given file content.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::history::hash_source;
///
/// let h1 = hash_source(b"hello");
/// let h2 = hash_source(b"hello");
/// assert_eq!(h1, h2);
/// # }
/// ```
pub fn hash_source(content: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Updates history with new test results and detects flaky tests.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::history::{TestHistory, update_and_detect};
/// use behave::cli::config::FlakyDetectionConfig;
/// use behave::cli::parser::{TestResult, TestOutcome};
///
/// let mut history = TestHistory::new();
/// let results = vec![TestResult::new("test".to_string(), TestOutcome::Pass)];
/// let config = FlakyDetectionConfig::default();
/// let flaky = update_and_detect(&mut history, &results, &config, "abc");
/// assert!(flaky.is_empty());
/// # }
/// ```
pub fn update_and_detect(
    history: &mut TestHistory,
    results: &[TestResult],
    config: &FlakyDetectionConfig,
    current_source_hash: &str,
) -> Vec<FlakyTest> {
    let mut flaky_tests = Vec::new();

    for result in results {
        let entry = history.tests.entry(result.full_name.clone()).or_default();

        match result.outcome {
            TestOutcome::Pass => {
                entry.consecutive_passes += 1;
                entry.last_outcome = "pass".to_string();
                entry.source_hash = current_source_hash.to_string();
            }
            TestOutcome::Fail => {
                if entry.consecutive_passes >= config.consecutive_passes
                    && entry.source_hash == current_source_hash
                {
                    flaky_tests.push(FlakyTest::new(
                        result.full_name.clone(),
                        entry.consecutive_passes,
                    ));
                }
                entry.consecutive_passes = 0;
                entry.last_outcome = "fail".to_string();
                entry.source_hash = current_source_hash.to_string();
            }
            TestOutcome::Ignored => {
                entry.last_outcome = "ignored".to_string();
            }
        }
    }

    flaky_tests
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::config::FlakyDetectionConfig;
    use crate::cli::parser::{TestOutcome, TestResult};

    #[test]
    fn new_history_is_empty() {
        let history = TestHistory::new();
        assert_eq!(history.version, HISTORY_VERSION);
        assert!(history.tests.is_empty());
    }

    #[test]
    fn new_entry_defaults() {
        let entry = TestEntry::new();
        assert_eq!(entry.consecutive_passes, 0);
        assert!(entry.last_outcome.is_empty());
        assert!(entry.source_hash.is_empty());
    }

    #[test]
    fn hash_source_deterministic() {
        let h1 = hash_source(b"hello world");
        let h2 = hash_source(b"hello world");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_source_different_content() {
        let h1 = hash_source(b"hello");
        let h2 = hash_source(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn update_pass_increments_count() {
        let mut history = TestHistory::new();
        let results = vec![TestResult::new("test::a".to_string(), TestOutcome::Pass)];
        let config = FlakyDetectionConfig::default();

        let flaky = update_and_detect(&mut history, &results, &config, "hash1");
        assert!(flaky.is_empty());
        assert_eq!(history.tests["test::a"].consecutive_passes, 1);
        assert_eq!(history.tests["test::a"].last_outcome, "pass");
    }

    #[test]
    fn update_fail_resets_count() {
        let mut history = TestHistory::new();
        history.tests.insert(
            "test::a".to_string(),
            TestEntry {
                consecutive_passes: 3,
                last_outcome: "pass".to_string(),
                source_hash: "hash1".to_string(),
            },
        );

        let results = vec![TestResult::new("test::a".to_string(), TestOutcome::Fail)];
        let config = FlakyDetectionConfig {
            consecutive_passes: 10,
            ..FlakyDetectionConfig::default()
        };

        let flaky = update_and_detect(&mut history, &results, &config, "hash1");
        assert!(flaky.is_empty());
        assert_eq!(history.tests["test::a"].consecutive_passes, 0);
    }

    #[test]
    fn detects_flaky_test() {
        let mut history = TestHistory::new();
        history.tests.insert(
            "test::flaky".to_string(),
            TestEntry {
                consecutive_passes: 10,
                last_outcome: "pass".to_string(),
                source_hash: "same_hash".to_string(),
            },
        );

        let results = vec![TestResult::new(
            "test::flaky".to_string(),
            TestOutcome::Fail,
        )];
        let config = FlakyDetectionConfig {
            consecutive_passes: 5,
            ..FlakyDetectionConfig::default()
        };

        let flaky = update_and_detect(&mut history, &results, &config, "same_hash");
        assert_eq!(flaky.len(), 1);
        assert_eq!(flaky[0].name, "test::flaky");
        assert_eq!(flaky[0].consecutive_passes, 10);
    }

    #[test]
    fn no_flaky_when_source_changed() {
        let mut history = TestHistory::new();
        history.tests.insert(
            "test::changed".to_string(),
            TestEntry {
                consecutive_passes: 10,
                last_outcome: "pass".to_string(),
                source_hash: "old_hash".to_string(),
            },
        );

        let results = vec![TestResult::new(
            "test::changed".to_string(),
            TestOutcome::Fail,
        )];
        let config = FlakyDetectionConfig {
            consecutive_passes: 5,
            ..FlakyDetectionConfig::default()
        };

        let flaky = update_and_detect(&mut history, &results, &config, "new_hash");
        assert!(flaky.is_empty());
    }

    #[test]
    fn no_flaky_below_threshold() {
        let mut history = TestHistory::new();
        history.tests.insert(
            "test::below".to_string(),
            TestEntry {
                consecutive_passes: 3,
                last_outcome: "pass".to_string(),
                source_hash: "hash".to_string(),
            },
        );

        let results = vec![TestResult::new(
            "test::below".to_string(),
            TestOutcome::Fail,
        )];
        let config = FlakyDetectionConfig {
            consecutive_passes: 5,
            ..FlakyDetectionConfig::default()
        };

        let flaky = update_and_detect(&mut history, &results, &config, "hash");
        assert!(flaky.is_empty());
    }

    #[test]
    fn ignored_tests_unchanged() {
        let mut history = TestHistory::new();
        let results = vec![TestResult::new(
            "test::ignored".to_string(),
            TestOutcome::Ignored,
        )];
        let config = FlakyDetectionConfig::default();

        let flaky = update_and_detect(&mut history, &results, &config, "hash");
        assert!(flaky.is_empty());
        assert_eq!(history.tests["test::ignored"].last_outcome, "ignored");
        assert_eq!(history.tests["test::ignored"].consecutive_passes, 0);
    }

    #[test]
    fn load_history_missing_file() {
        let result = load_history(Path::new("/nonexistent/path/history.json"));
        assert!(result.is_ok());
        assert!(result.unwrap_or_default().tests.is_empty());
    }

    #[test]
    fn roundtrip_serialize() {
        let mut history = TestHistory::new();
        history.tests.insert(
            "test::a".to_string(),
            TestEntry {
                consecutive_passes: 5,
                last_outcome: "pass".to_string(),
                source_hash: "abc123".to_string(),
            },
        );

        let json = serde_json::to_string(&history).unwrap_or_default();
        let deserialized: TestHistory = serde_json::from_str(&json).unwrap_or_default();
        assert_eq!(deserialized.tests.len(), 1);
        assert_eq!(deserialized.tests["test::a"].consecutive_passes, 5);
    }
}
