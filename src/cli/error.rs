//! CLI-specific error types.

use std::fmt;

/// Errors that can occur during CLI operation.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::error::CliError;
///
/// let err = CliError::OutputParse {
///     line: "bad line".to_string(),
/// };
/// assert!(err.to_string().contains("bad line"));
/// # }
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub enum CliError {
    /// Failed to invoke `cargo test`.
    CargoInvocation {
        /// The underlying IO error.
        source: std::io::Error,
    },
    /// Failed to parse test output.
    ///
    /// Reserved for future use by structured output parsers.
    OutputParse {
        /// The line that could not be parsed.
        line: String,
    },
    /// IO error during rendering.
    Io {
        /// The underlying IO error.
        source: std::io::Error,
    },
    /// Failed to resolve Cargo metadata for the current invocation.
    Metadata {
        /// Description of the metadata error.
        message: String,
    },
    /// Failed to parse `behave.toml` configuration.
    ConfigParse {
        /// Description of the parse error.
        message: String,
    },
    /// IO error reading or writing history file.
    HistoryIo {
        /// The underlying IO error.
        source: std::io::Error,
    },
    /// The selected package spec could not be mapped to a workspace package.
    PackageSelection {
        /// The original package selector.
        spec: String,
    },
    /// The user passed a libtest argument that would break CLI parsing.
    UnsupportedLibtestArg {
        /// The unsupported argument.
        arg: String,
    },
    /// Focused tests found in CI guard mode.
    FocusedTestsFound {
        /// The number of focused tests.
        count: usize,
    },
    /// Watch mode initialization failed.
    WatchInit {
        /// Description of the error.
        message: String,
    },
    /// Failed to parse a `--filter` expression.
    FilterParse {
        /// Description of the parse error.
        message: String,
    },
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CargoInvocation { source } => {
                write!(
                    f,
                    "failed to invoke cargo test: {source}\n  hint: is `cargo` installed and in your PATH?"
                )
            }
            Self::OutputParse { line } => {
                write!(f, "failed to parse test output line: {line}")
            }
            Self::Io { source } => write!(f, "io error: {source}"),
            Self::Metadata { message } => {
                write!(f, "failed to load cargo metadata: {message}")
            }
            Self::ConfigParse { message } => {
                write!(
                    f,
                    "failed to parse behave.toml: {message}\n  hint: check TOML syntax at https://toml.io"
                )
            }
            Self::HistoryIo { source } => write!(f, "history file error: {source}"),
            Self::PackageSelection { spec } => {
                write!(
                    f,
                    "could not resolve package selection for {spec}\n  hint: check that the package name matches a workspace member"
                )
            }
            Self::UnsupportedLibtestArg { arg } => {
                write!(f, "unsupported libtest argument for cargo-behave: {arg}")
            }
            Self::FocusedTestsFound { count } => {
                write!(
                    f,
                    "{count} focused test(s) found; remove `focus` markers before merging"
                )
            }
            Self::WatchInit { message } => {
                write!(f, "failed to initialize watch mode: {message}")
            }
            Self::FilterParse { message } => {
                write!(
                    f,
                    "invalid filter expression: {message}\n  syntax: tag(name), name(pattern), and/or/not, parentheses"
                )
            }
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CargoInvocation { source } | Self::Io { source } | Self::HistoryIo { source } => {
                Some(source)
            }
            Self::OutputParse { .. }
            | Self::Metadata { .. }
            | Self::ConfigParse { .. }
            | Self::PackageSelection { .. }
            | Self::UnsupportedLibtestArg { .. }
            | Self::FocusedTestsFound { .. }
            | Self::WatchInit { .. }
            | Self::FilterParse { .. } => None,
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        Self::Io { source: err }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_cargo_invocation() {
        let err = CliError::CargoInvocation {
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        let msg = err.to_string();
        assert!(msg.contains("failed to invoke cargo test"));
        assert!(msg.contains("not found"));
        assert!(msg.contains("hint:"));
    }

    #[test]
    fn display_output_parse() {
        let err = CliError::OutputParse {
            line: "bad line".to_string(),
        };
        assert!(err.to_string().contains("bad line"));
    }

    #[test]
    fn display_io() {
        let err = CliError::Io {
            source: std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken"),
        };
        assert!(err.to_string().contains("io error"));
    }

    #[test]
    fn source_cargo_invocation() {
        let err = CliError::CargoInvocation {
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "nf"),
        };
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn source_output_parse_is_none() {
        let err = CliError::OutputParse {
            line: "x".to_string(),
        };
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn source_io() {
        let err = CliError::Io {
            source: std::io::Error::other("err"),
        };
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::other("test");
        let cli_err: CliError = io_err.into();
        assert!(matches!(cli_err, CliError::Io { .. }));
    }

    #[test]
    fn display_config_parse() {
        let err = CliError::ConfigParse {
            message: "bad toml".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("failed to parse behave.toml"));
        assert!(msg.contains("bad toml"));
    }

    #[test]
    fn source_config_parse_is_none() {
        let err = CliError::ConfigParse {
            message: "err".to_string(),
        };
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn display_history_io() {
        let err = CliError::HistoryIo {
            source: std::io::Error::other("disk full"),
        };
        let msg = err.to_string();
        assert!(msg.contains("history file error"));
        assert!(msg.contains("disk full"));
    }

    #[test]
    fn source_history_io() {
        let err = CliError::HistoryIo {
            source: std::io::Error::other("err"),
        };
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn display_metadata() {
        let err = CliError::Metadata {
            message: "workspace failed".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("failed to load cargo metadata"));
        assert!(msg.contains("workspace failed"));
    }

    #[test]
    fn source_metadata_is_none() {
        let err = CliError::Metadata {
            message: "bad".to_string(),
        };
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn display_package_selection() {
        let err = CliError::PackageSelection {
            spec: "demo".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("demo"));
        assert!(msg.contains("resolve package"));
    }

    #[test]
    fn display_unsupported_libtest_arg() {
        let err = CliError::UnsupportedLibtestArg {
            arg: "--format".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("unsupported libtest argument"));
        assert!(msg.contains("--format"));
    }

    #[test]
    fn source_unsupported_libtest_arg_is_none() {
        let err = CliError::UnsupportedLibtestArg {
            arg: "--format".to_string(),
        };
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn display_focused_tests_found() {
        let err = CliError::FocusedTestsFound { count: 3 };
        let msg = err.to_string();
        assert!(msg.contains("3 focused test(s) found"));
        assert!(msg.contains("remove `focus` markers"));
    }

    #[test]
    fn source_focused_tests_found_is_none() {
        let err = CliError::FocusedTestsFound { count: 1 };
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn display_watch_init() {
        let err = CliError::WatchInit {
            message: "no directory".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("failed to initialize watch mode"));
        assert!(msg.contains("no directory"));
    }

    #[test]
    fn display_filter_parse() {
        let err = CliError::FilterParse {
            message: "unexpected token".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("invalid filter expression"));
        assert!(msg.contains("unexpected token"));
    }

    #[test]
    fn source_filter_parse_is_none() {
        let err = CliError::FilterParse {
            message: "err".to_string(),
        };
        assert!(std::error::Error::source(&err).is_none());
    }
}
