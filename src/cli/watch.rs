//! File-watching loop for `cargo-behave --watch`.

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};

use super::error::CliError;

/// Debounce interval for file change events.
const DEBOUNCE_MS: u64 = 200;

/// Runs a watch loop that re-executes `run_tests` whenever `.rs` files change.
///
/// Watches `src/` and `tests/` directories recursively for Rust file changes.
/// Clears the terminal between runs.
///
/// # Errors
///
/// Returns [`CliError::WatchInit`] if the watcher cannot be created.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::watch::watch_loop;
/// use behave::cli::error::CliError;
///
/// let dir = std::path::Path::new(".");
/// watch_loop(dir, || {
///     Ok(())
/// });
/// # }
/// ```
pub fn watch_loop(
    project_dir: &Path,
    mut run_tests: impl FnMut() -> Result<(), CliError>,
) -> Result<(), CliError> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = create_watcher(tx)?;

    watch_directory(&mut watcher, &project_dir.join("src"));
    watch_directory(&mut watcher, &project_dir.join("tests"));

    // Initial run
    let _ = run_tests();

    #[allow(clippy::print_stderr)]
    {
        eprintln!("\nwatching for changes...");
    }

    loop {
        if wait_for_rust_changes(&rx) {
            clear_terminal();
            let _ = run_tests();

            #[allow(clippy::print_stderr)]
            {
                eprintln!("\nwatching for changes...");
            }
        }
    }
}

fn create_watcher(
    tx: mpsc::Sender<notify::Result<notify::Event>>,
) -> Result<notify::RecommendedWatcher, CliError> {
    notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })
    .map_err(|err| CliError::WatchInit {
        message: err.to_string(),
    })
}

fn watch_directory(watcher: &mut notify::RecommendedWatcher, dir: &Path) {
    if dir.is_dir() {
        let _ = watcher.watch(dir, RecursiveMode::Recursive);
    }
}

/// Waits for at least one Rust file change event, then drains pending events.
fn wait_for_rust_changes(rx: &mpsc::Receiver<notify::Result<notify::Event>>) -> bool {
    // Block until first event
    loop {
        let Ok(Ok(event)) = rx.recv() else {
            return false;
        };

        if event_contains_rust_file(&event) {
            // Debounce: drain pending events
            drain_events(rx);
            return true;
        }
    }
}

fn drain_events(rx: &mpsc::Receiver<notify::Result<notify::Event>>) {
    while rx.recv_timeout(Duration::from_millis(DEBOUNCE_MS)).is_ok() {}
}

fn event_contains_rust_file(event: &notify::Event) -> bool {
    event
        .paths
        .iter()
        .any(|p| is_rust_file(p.to_string_lossy().as_ref()))
}

/// Returns `true` if the path ends with `.rs`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::watch::is_rust_file;
///
/// assert!(is_rust_file("src/main.rs"));
/// assert!(!is_rust_file("src/data.json"));
/// # }
/// ```
pub fn is_rust_file(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
}

fn clear_terminal() {
    #[allow(clippy::print_stdout)]
    {
        // ANSI escape sequence to clear screen and move cursor to top-left
        print!("\x1B[2J\x1B[H");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_rust_file_matches_rs_extension() {
        assert!(is_rust_file("src/main.rs"));
        assert!(is_rust_file("tests/smoke.rs"));
        assert!(is_rust_file("/absolute/path/lib.rs"));
    }

    #[test]
    fn is_rust_file_rejects_non_rs_files() {
        assert!(!is_rust_file("src/data.json"));
        assert!(!is_rust_file("Cargo.toml"));
        assert!(!is_rust_file("README.md"));
        assert!(!is_rust_file("src/lib.rsx"));
    }

    #[test]
    fn is_rust_file_case_insensitive() {
        assert!(is_rust_file("src/main.RS"));
        assert!(is_rust_file("tests/Smoke.Rs"));
    }

    #[test]
    fn is_rust_file_no_extension() {
        assert!(!is_rust_file("Makefile"));
        assert!(!is_rust_file(""));
    }

    #[test]
    fn is_rust_file_hidden_files() {
        assert!(!is_rust_file(".gitignore"));
        assert!(is_rust_file(".hidden.rs"));
    }

    #[test]
    fn is_rust_file_with_dots_in_path() {
        assert!(is_rust_file("src/v1.2/lib.rs"));
        assert!(!is_rust_file("src/v1.2/lib.txt"));
    }

    #[test]
    fn event_contains_rust_file_filters_correctly() {
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Any,
            )),
            paths: vec![
                std::path::PathBuf::from("src/lib.rs"),
                std::path::PathBuf::from("Cargo.toml"),
            ],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(event_contains_rust_file(&event));
    }

    #[test]
    fn event_without_rust_files_rejected() {
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Any,
            )),
            paths: vec![
                std::path::PathBuf::from("Cargo.toml"),
                std::path::PathBuf::from("README.md"),
            ],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(!event_contains_rust_file(&event));
    }

    #[test]
    fn event_with_empty_paths_rejected() {
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Any,
            )),
            paths: vec![],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(!event_contains_rust_file(&event));
    }
}
