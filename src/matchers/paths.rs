//! Matchers for filesystem paths.

use std::path::{Path, PathBuf};

use crate::error::MatchError;
use crate::expectation::Expectation;

impl Expectation<PathBuf> {
    /// Asserts the path exists on the filesystem.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the path does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use behave::Expectation;
    ///
    /// let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    /// let result = Expectation::new(path, "p").to_exist();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_exist(&self) -> Result<(), MatchError> {
        self.check(self.value().exists(), "to exist")
    }

    /// Asserts the path points to a regular file.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the path is not a file.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use behave::Expectation;
    ///
    /// let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    /// let result = Expectation::new(path, "p").to_be_a_file();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_a_file(&self) -> Result<(), MatchError> {
        self.check(self.value().is_file(), "to be a file")
    }

    /// Asserts the path points to a directory.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the path is not a directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use behave::Expectation;
    ///
    /// let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    /// let result = Expectation::new(path, "p").to_be_a_directory();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_a_directory(&self) -> Result<(), MatchError> {
        self.check(self.value().is_dir(), "to be a directory")
    }

    /// Asserts the path has the given file extension.
    ///
    /// The `ext` parameter should not include the leading dot.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the extension does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use behave::Expectation;
    ///
    /// let path = PathBuf::from("src/main.rs");
    /// let result = Expectation::new(path, "p").to_have_extension("rs");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_extension(&self, ext: &str) -> Result<(), MatchError> {
        let is_match = self.value().extension().is_some_and(|e| e == ext);
        self.check(is_match, format!("to have extension {ext:?}"))
    }

    /// Asserts the path has the given file name (including extension).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the file name does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use behave::Expectation;
    ///
    /// let path = PathBuf::from("src/main.rs");
    /// let result = Expectation::new(path, "p").to_have_file_name("main.rs");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_file_name(&self, name: &str) -> Result<(), MatchError> {
        let is_match = self.value().file_name().is_some_and(|n| n == name);
        self.check(is_match, format!("to have file name {name:?}"))
    }
}

impl Expectation<&Path> {
    /// Asserts the path exists on the filesystem.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the path does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use behave::Expectation;
    ///
    /// let p = Path::new(env!("CARGO_MANIFEST_DIR"));
    /// let result = Expectation::new(p, "p").to_exist();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_exist(&self) -> Result<(), MatchError> {
        self.check(self.value().exists(), "to exist")
    }

    /// Asserts the path points to a regular file.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the path is not a file.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use behave::Expectation;
    ///
    /// let p = Path::new("Cargo.toml");
    /// let result = Expectation::new(p, "p").to_be_a_file();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_a_file(&self) -> Result<(), MatchError> {
        self.check(self.value().is_file(), "to be a file")
    }

    /// Asserts the path points to a directory.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the path is not a directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use behave::Expectation;
    ///
    /// let p = Path::new("src");
    /// let result = Expectation::new(p, "p").to_be_a_directory();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_a_directory(&self) -> Result<(), MatchError> {
        self.check(self.value().is_dir(), "to be a directory")
    }

    /// Asserts the path has the given file extension.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the extension does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use behave::Expectation;
    ///
    /// let p = Path::new("main.rs");
    /// let result = Expectation::new(p, "p").to_have_extension("rs");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_extension(&self, ext: &str) -> Result<(), MatchError> {
        let is_match = self.value().extension().is_some_and(|e| e == ext);
        self.check(is_match, format!("to have extension {ext:?}"))
    }

    /// Asserts the path has the given file name (including extension).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the file name does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use behave::Expectation;
    ///
    /// let p = Path::new("main.rs");
    /// let result = Expectation::new(p, "p").to_have_file_name("main.rs");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_file_name(&self, name: &str) -> Result<(), MatchError> {
        let is_match = self.value().file_name().is_some_and(|n| n == name);
        self.check(is_match, format!("to have file name {name:?}"))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::Expectation;

    fn manifest_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn to_exist_pass() {
        let p = manifest_dir().join("Cargo.toml");
        assert!(Expectation::new(p, "p").to_exist().is_ok());
    }

    #[test]
    fn to_exist_fail() {
        let p = manifest_dir().join("nonexistent_file_xyz");
        assert!(Expectation::new(p, "p").to_exist().is_err());
    }

    #[test]
    fn to_exist_negated() {
        let p = manifest_dir().join("nonexistent_file_xyz");
        assert!(Expectation::new(p, "p").negate().to_exist().is_ok());
    }

    #[test]
    fn to_be_a_file_pass() {
        let p = manifest_dir().join("Cargo.toml");
        assert!(Expectation::new(p, "p").to_be_a_file().is_ok());
    }

    #[test]
    fn to_be_a_file_fail_directory() {
        let p = manifest_dir().join("src");
        assert!(Expectation::new(p, "p").to_be_a_file().is_err());
    }

    #[test]
    fn to_be_a_file_negated() {
        let p = manifest_dir().join("src");
        assert!(Expectation::new(p, "p").negate().to_be_a_file().is_ok());
    }

    #[test]
    fn to_be_a_directory_pass() {
        let p = manifest_dir().join("src");
        assert!(Expectation::new(p, "p").to_be_a_directory().is_ok());
    }

    #[test]
    fn to_be_a_directory_fail_file() {
        let p = manifest_dir().join("Cargo.toml");
        assert!(Expectation::new(p, "p").to_be_a_directory().is_err());
    }

    #[test]
    fn to_be_a_directory_negated() {
        let p = manifest_dir().join("Cargo.toml");
        assert!(Expectation::new(p, "p")
            .negate()
            .to_be_a_directory()
            .is_ok());
    }

    #[test]
    fn to_have_extension_pass() {
        let p = PathBuf::from("src/main.rs");
        assert!(Expectation::new(p, "p").to_have_extension("rs").is_ok());
    }

    #[test]
    fn to_have_extension_fail() {
        let p = PathBuf::from("src/main.rs");
        assert!(Expectation::new(p, "p").to_have_extension("txt").is_err());
    }

    #[test]
    fn to_have_extension_negated() {
        let p = PathBuf::from("src/main.rs");
        assert!(Expectation::new(p, "p")
            .negate()
            .to_have_extension("txt")
            .is_ok());
    }

    #[test]
    fn to_have_extension_none() {
        let p = PathBuf::from("Makefile");
        assert!(Expectation::new(p, "p").to_have_extension("rs").is_err());
    }

    #[test]
    fn to_have_file_name_pass() {
        let p = PathBuf::from("src/main.rs");
        assert!(Expectation::new(p, "p")
            .to_have_file_name("main.rs")
            .is_ok());
    }

    #[test]
    fn to_have_file_name_fail() {
        let p = PathBuf::from("src/main.rs");
        assert!(Expectation::new(p, "p")
            .to_have_file_name("lib.rs")
            .is_err());
    }

    #[test]
    fn to_have_file_name_negated() {
        let p = PathBuf::from("src/main.rs");
        assert!(Expectation::new(p, "p")
            .negate()
            .to_have_file_name("lib.rs")
            .is_ok());
    }

    // --- &Path tests ---

    #[test]
    fn path_ref_to_exist_pass() {
        let p = manifest_dir().join("Cargo.toml");
        assert!(Expectation::new(p.as_path(), "p").to_exist().is_ok());
    }

    #[test]
    fn path_ref_to_exist_fail() {
        let p = manifest_dir().join("nonexistent_xyz");
        assert!(Expectation::new(p.as_path(), "p").to_exist().is_err());
    }

    #[test]
    fn path_ref_to_be_a_file_pass() {
        let p = manifest_dir().join("Cargo.toml");
        assert!(Expectation::new(p.as_path(), "p").to_be_a_file().is_ok());
    }

    #[test]
    fn path_ref_to_be_a_file_fail() {
        let p = manifest_dir().join("src");
        assert!(Expectation::new(p.as_path(), "p").to_be_a_file().is_err());
    }

    #[test]
    fn path_ref_to_be_a_directory_pass() {
        let p = manifest_dir().join("src");
        assert!(Expectation::new(p.as_path(), "p")
            .to_be_a_directory()
            .is_ok());
    }

    #[test]
    fn path_ref_to_have_extension_pass() {
        let p = std::path::Path::new("src/main.rs");
        assert!(Expectation::new(p, "p").to_have_extension("rs").is_ok());
    }

    #[test]
    fn path_ref_to_have_extension_fail() {
        let p = std::path::Path::new("src/main.rs");
        assert!(Expectation::new(p, "p").to_have_extension("txt").is_err());
    }

    #[test]
    fn path_ref_to_have_file_name_pass() {
        let p = std::path::Path::new("src/main.rs");
        assert!(Expectation::new(p, "p")
            .to_have_file_name("main.rs")
            .is_ok());
    }

    #[test]
    fn path_ref_to_have_file_name_fail() {
        let p = std::path::Path::new("src/main.rs");
        assert!(Expectation::new(p, "p")
            .to_have_file_name("lib.rs")
            .is_err());
    }
}
