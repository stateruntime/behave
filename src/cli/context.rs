//! Workspace and package resolution for `cargo-behave`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use super::error::CliError;

/// Resolved cargo invocation context.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProjectContext {
    /// The workspace root reported by `cargo metadata`.
    pub workspace_root: PathBuf,
    /// Directory used to resolve `behave.toml` and history paths.
    pub config_dir: PathBuf,
    /// Packages selected by the current cargo invocation.
    pub selected_packages: Vec<SelectedPackage>,
}

/// A selected package with its manifest and root directory.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SelectedPackage {
    /// Package name from Cargo metadata.
    pub name: String,
    /// Absolute path to the package manifest.
    pub manifest_path: PathBuf,
    /// Absolute path to the package root directory.
    pub root_dir: PathBuf,
}

impl SelectedPackage {
    /// Creates a selected package record.
    #[must_use]
    pub const fn new(name: String, manifest_path: PathBuf, root_dir: PathBuf) -> Self {
        Self {
            name,
            manifest_path,
            root_dir,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct SelectionArgs {
    manifest_path: Option<PathBuf>,
    package_specs: Vec<String>,
    exclude_specs: Vec<String>,
    workspace: bool,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    workspace_root: PathBuf,
    workspace_members: Vec<String>,
    workspace_default_members: Vec<String>,
    packages: Vec<MetadataPackage>,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    id: String,
    name: String,
    manifest_path: PathBuf,
}

/// Resolves the workspace and selected packages for a cargo invocation.
///
/// # Errors
///
/// Returns [`CliError`] if Cargo metadata cannot be loaded.
pub fn resolve_project_context(
    current_dir: &Path,
    cargo_args: &[&str],
) -> Result<ProjectContext, CliError> {
    let selection = parse_selection_args(current_dir, cargo_args);
    let metadata = load_metadata(current_dir, selection.manifest_path.as_deref())?;
    build_context(current_dir, metadata, &selection)
}

fn parse_selection_args(current_dir: &Path, cargo_args: &[&str]) -> SelectionArgs {
    let mut selection = SelectionArgs::default();
    let mut index = 0;

    while index < cargo_args.len() {
        index = parse_arg(current_dir, cargo_args, index, &mut selection);
    }

    selection
}

fn parse_arg(
    current_dir: &Path,
    cargo_args: &[&str],
    index: usize,
    selection: &mut SelectionArgs,
) -> usize {
    let arg = cargo_args[index];

    if arg == "--workspace" {
        selection.workspace = true;
        return index + 1;
    }

    if let Some(path) = value_after_flag(arg, cargo_args, index, "--manifest-path") {
        selection.manifest_path = Some(resolve_path(current_dir, path));
        return next_index(arg, index);
    }

    if let Some(spec) = value_after_flag(arg, cargo_args, index, "--package") {
        selection.package_specs.push(spec.to_string());
        return next_index(arg, index);
    }

    if let Some(spec) = value_after_flag(arg, cargo_args, index, "--exclude") {
        selection.exclude_specs.push(spec.to_string());
        return next_index(arg, index);
    }

    if let Some(spec) = short_flag_value(arg, cargo_args, index, "-p") {
        selection.package_specs.push(spec.to_string());
        return short_next_index(arg, index);
    }

    index + 1
}

fn value_after_flag<'a>(
    arg: &'a str,
    args: &'a [&str],
    index: usize,
    flag: &str,
) -> Option<&'a str> {
    if arg == flag {
        return args.get(index + 1).copied();
    }

    arg.strip_prefix(&format!("{flag}="))
}

fn short_flag_value<'a>(
    arg: &'a str,
    args: &'a [&str],
    index: usize,
    flag: &str,
) -> Option<&'a str> {
    if arg == flag {
        return args.get(index + 1).copied();
    }

    arg.strip_prefix(flag).filter(|value| !value.is_empty())
}

fn next_index(arg: &str, index: usize) -> usize {
    if arg.contains('=') {
        index + 1
    } else {
        index + 2
    }
}

fn short_next_index(arg: &str, index: usize) -> usize {
    if arg == "-p" {
        index + 2
    } else {
        index + 1
    }
}

fn resolve_path(current_dir: &Path, path: &str) -> PathBuf {
    let candidate = Path::new(path);

    if candidate.is_absolute() {
        return candidate.to_path_buf();
    }

    current_dir.join(candidate)
}

fn load_metadata(current_dir: &Path, manifest_path: Option<&Path>) -> Result<Metadata, CliError> {
    let mut command = Command::new("cargo");
    command.current_dir(current_dir);
    command.arg("metadata");
    command.arg("--format-version");
    command.arg("1");
    command.arg("--no-deps");

    if let Some(path) = manifest_path {
        command.arg("--manifest-path");
        command.arg(path);
    }

    let output = command
        .output()
        .map_err(|source| CliError::CargoInvocation { source })?;

    if !output.status.success() {
        return Err(CliError::Metadata {
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    serde_json::from_slice(&output.stdout).map_err(|err| CliError::Metadata {
        message: err.to_string(),
    })
}

fn build_context(
    current_dir: &Path,
    metadata: Metadata,
    selection: &SelectionArgs,
) -> Result<ProjectContext, CliError> {
    let explicit_manifest = selection.manifest_path.as_deref();
    let explicit_package =
        explicit_manifest.and_then(|path| find_package_by_manifest(&metadata.packages, path));
    let config_dir = explicit_package.as_ref().map_or_else(
        || metadata.workspace_root.clone(),
        |package| package.root_dir.clone(),
    );

    let selected_packages = select_packages(current_dir, &metadata, selection)?;

    Ok(ProjectContext {
        workspace_root: metadata.workspace_root,
        config_dir,
        selected_packages,
    })
}

fn select_packages(
    current_dir: &Path,
    metadata: &Metadata,
    selection: &SelectionArgs,
) -> Result<Vec<SelectedPackage>, CliError> {
    let mut selected = if !selection.package_specs.is_empty() {
        select_named_packages(metadata, &selection.package_specs)?
    } else if selection.workspace {
        packages_from_ids(metadata, &metadata.workspace_members)
    } else if let Some(path) = selection.manifest_path.as_deref() {
        select_for_manifest_path(metadata, path)
    } else if let Some(package) = current_package(metadata, current_dir) {
        vec![package]
    } else {
        default_member_packages(metadata)
    };

    selected = exclude_packages(selected, &selection.exclude_specs);
    selected.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(selected)
}

fn select_named_packages(
    metadata: &Metadata,
    specs: &[String],
) -> Result<Vec<SelectedPackage>, CliError> {
    let mut packages = Vec::new();

    for spec in specs {
        let matches = matching_packages(metadata, spec);
        if matches.is_empty() {
            return Err(CliError::PackageSelection { spec: spec.clone() });
        }
        extend_unique(&mut packages, matches);
    }

    Ok(packages)
}

fn matching_packages(metadata: &Metadata, spec: &str) -> Vec<SelectedPackage> {
    let mut matches = Vec::new();

    for package in &metadata.packages {
        if package_matches_spec(package, spec) {
            extend_unique(&mut matches, vec![selected_package(package)]);
        }
    }

    matches
}

fn package_matches_spec(package: &MetadataPackage, spec: &str) -> bool {
    package.name == spec
        || package.id.starts_with(&format!("{spec} "))
        || package.manifest_path.as_os_str() == OsStr::new(spec)
}

fn select_for_manifest_path(metadata: &Metadata, path: &Path) -> Vec<SelectedPackage> {
    find_package_by_manifest(&metadata.packages, path).map_or_else(
        || default_member_packages(metadata),
        |package| vec![package],
    )
}

fn current_package(metadata: &Metadata, current_dir: &Path) -> Option<SelectedPackage> {
    let mut best_match: Option<SelectedPackage> = None;

    for package in &metadata.packages {
        let root_dir = package_root_dir(package);
        if !current_dir.starts_with(&root_dir) {
            continue;
        }

        let candidate = selected_package(package);
        let replace = best_match.as_ref().map_or(true, |best| {
            candidate.root_dir.components().count() > best.root_dir.components().count()
        });

        if replace {
            best_match = Some(candidate);
        }
    }

    best_match
}

fn default_member_packages(metadata: &Metadata) -> Vec<SelectedPackage> {
    if metadata.workspace_default_members.is_empty() {
        return packages_from_ids(metadata, &metadata.workspace_members);
    }

    packages_from_ids(metadata, &metadata.workspace_default_members)
}

fn packages_from_ids(metadata: &Metadata, package_ids: &[String]) -> Vec<SelectedPackage> {
    let mut packages = Vec::new();

    for package_id in package_ids {
        if let Some(package) = metadata.packages.iter().find(|pkg| pkg.id == *package_id) {
            extend_unique(&mut packages, vec![selected_package(package)]);
        }
    }

    packages
}

fn exclude_packages(packages: Vec<SelectedPackage>, specs: &[String]) -> Vec<SelectedPackage> {
    packages
        .into_iter()
        .filter(|package| {
            !specs
                .iter()
                .any(|spec| package_matches_selection(package, spec))
        })
        .collect()
}

fn package_matches_selection(package: &SelectedPackage, spec: &str) -> bool {
    package.name == spec || package.manifest_path.as_os_str() == OsStr::new(spec)
}

fn find_package_by_manifest(packages: &[MetadataPackage], path: &Path) -> Option<SelectedPackage> {
    packages
        .iter()
        .find(|package| package.manifest_path == path)
        .map(selected_package)
}

fn selected_package(package: &MetadataPackage) -> SelectedPackage {
    SelectedPackage::new(
        package.name.clone(),
        package.manifest_path.clone(),
        package_root_dir(package),
    )
}

fn package_root_dir(package: &MetadataPackage) -> PathBuf {
    package
        .manifest_path
        .parent()
        .map_or_else(PathBuf::new, Path::to_path_buf)
}

fn extend_unique(target: &mut Vec<SelectedPackage>, packages: Vec<SelectedPackage>) {
    for package in packages {
        let exists = target
            .iter()
            .any(|item| item.manifest_path == package.manifest_path);
        if !exists {
            target.push(package);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(path: &str) -> PathBuf {
        PathBuf::from(path)
    }

    fn metadata() -> Metadata {
        Metadata {
            workspace_root: manifest("/workspace"),
            workspace_members: vec![
                "pkg-a 0.1.0 (path+file:///workspace/pkg-a)".to_string(),
                "pkg-b 0.1.0 (path+file:///workspace/pkg-b)".to_string(),
            ],
            workspace_default_members: vec![
                "pkg-a 0.1.0 (path+file:///workspace/pkg-a)".to_string()
            ],
            packages: vec![
                MetadataPackage {
                    id: "pkg-a 0.1.0 (path+file:///workspace/pkg-a)".to_string(),
                    name: "pkg-a".to_string(),
                    manifest_path: manifest("/workspace/pkg-a/Cargo.toml"),
                },
                MetadataPackage {
                    id: "pkg-b 0.1.0 (path+file:///workspace/pkg-b)".to_string(),
                    name: "pkg-b".to_string(),
                    manifest_path: manifest("/workspace/pkg-b/Cargo.toml"),
                },
            ],
        }
    }

    #[test]
    fn parses_selection_args() {
        let args = [
            "--manifest-path",
            "pkg-b/Cargo.toml",
            "--package",
            "pkg-b",
            "--exclude=pkg-a",
            "--workspace",
        ];
        let parsed = parse_selection_args(Path::new("/workspace"), &args);

        assert_eq!(
            parsed.manifest_path,
            Some(PathBuf::from("/workspace/pkg-b/Cargo.toml"))
        );
        assert_eq!(parsed.package_specs, vec!["pkg-b"]);
        assert_eq!(parsed.exclude_specs, vec!["pkg-a"]);
        assert!(parsed.workspace);
    }

    #[test]
    fn selects_current_package_from_member_dir() {
        let selected = select_packages(
            Path::new("/workspace/pkg-b"),
            &metadata(),
            &SelectionArgs::default(),
        );

        assert!(selected.is_ok());
        assert_eq!(selected.unwrap_or_default()[0].name, "pkg-b");
    }

    #[test]
    fn selects_default_members_from_workspace_root() {
        let selected = select_packages(
            Path::new("/workspace"),
            &metadata(),
            &SelectionArgs::default(),
        );

        assert!(selected.is_ok());
        assert_eq!(selected.unwrap_or_default()[0].name, "pkg-a");
    }

    #[test]
    fn selects_explicit_manifest_package() {
        let selected = select_packages(
            Path::new("/workspace"),
            &metadata(),
            &SelectionArgs {
                manifest_path: Some(manifest("/workspace/pkg-b/Cargo.toml")),
                ..SelectionArgs::default()
            },
        );

        assert!(selected.is_ok());
        assert_eq!(selected.unwrap_or_default()[0].name, "pkg-b");
    }

    #[test]
    fn selects_named_packages() {
        let selected = select_packages(
            Path::new("/workspace"),
            &metadata(),
            &SelectionArgs {
                package_specs: vec!["pkg-b".to_string()],
                ..SelectionArgs::default()
            },
        );

        assert!(selected.is_ok());
        assert_eq!(selected.unwrap_or_default()[0].name, "pkg-b");
    }

    #[test]
    fn excludes_named_packages() {
        let selected = select_packages(
            Path::new("/workspace"),
            &metadata(),
            &SelectionArgs {
                workspace: true,
                exclude_specs: vec!["pkg-a".to_string()],
                ..SelectionArgs::default()
            },
        );

        assert!(selected.is_ok());
        assert_eq!(selected.unwrap_or_default()[0].name, "pkg-b");
    }

    #[test]
    fn unknown_package_returns_error() {
        let selected = select_packages(
            Path::new("/workspace"),
            &metadata(),
            &SelectionArgs {
                package_specs: vec!["missing".to_string()],
                ..SelectionArgs::default()
            },
        );

        assert!(matches!(
            selected,
            Err(CliError::PackageSelection { spec }) if spec == "missing"
        ));
    }

    fn temp_workspace_dir(test_name: &str) -> PathBuf {
        let unique = format!(
            "behave-context-{test_name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos())
        );
        std::env::temp_dir().join(unique)
    }

    fn write_package(root: &Path, name: &str) {
        let package_dir = root.join(name);
        let src_dir = package_dir.join("src");

        assert!(std::fs::create_dir_all(&src_dir).is_ok());
        assert!(std::fs::write(
            package_dir.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        )
        .is_ok());
        assert!(std::fs::write(src_dir.join("lib.rs"), "pub fn demo() {}\n").is_ok());
    }

    #[test]
    fn resolves_workspace_packages_via_cargo_metadata() {
        let workspace_dir = temp_workspace_dir("metadata");

        assert!(std::fs::create_dir_all(&workspace_dir).is_ok());
        assert!(std::fs::write(
            workspace_dir.join("Cargo.toml"),
            "[workspace]\nmembers = [\"pkg-a\", \"pkg-b\"]\ndefault-members = [\"pkg-a\"]\nresolver = \"2\"\n",
        )
        .is_ok());
        write_package(&workspace_dir, "pkg-a");
        write_package(&workspace_dir, "pkg-b");

        let default_context = resolve_project_context(&workspace_dir, &[]);
        assert!(default_context.is_ok());
        if let Ok(context) = default_context {
            assert_eq!(context.selected_packages[0].name, "pkg-a");
        }

        let package_context = resolve_project_context(&workspace_dir, &["--package", "pkg-b"]);
        assert!(package_context.is_ok());
        if let Ok(context) = package_context {
            assert_eq!(context.selected_packages[0].name, "pkg-b");
        }

        let manifest_context =
            resolve_project_context(&workspace_dir, &["--manifest-path", "pkg-b/Cargo.toml"]);
        assert!(manifest_context.is_ok());
        if let Ok(context) = manifest_context {
            assert_eq!(context.selected_packages[0].name, "pkg-b");
            assert_eq!(context.config_dir, workspace_dir.join("pkg-b"));
        }

        assert!(std::fs::remove_dir_all(&workspace_dir).is_ok());
    }
}
