//! Matchers for `url::Url`.

use crate::error::MatchError;
use crate::expectation::Expectation;

#[cfg_attr(docsrs, doc(cfg(feature = "url")))]
impl Expectation<::url::Url> {
    /// Asserts the URL has the given scheme (e.g. `"https"`).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the scheme does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use url::Url;
    ///
    /// let u = Url::parse("https://example.com").unwrap();
    /// let result = Expectation::new(u, "u").to_have_scheme("https");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_scheme(&self, scheme: &str) -> Result<(), MatchError> {
        self.check(
            self.value().scheme() == scheme,
            format!("to have scheme {scheme:?}"),
        )
    }

    /// Asserts the URL has the given host.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the host does not match or is absent.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use url::Url;
    ///
    /// let u = Url::parse("https://example.com/path").unwrap();
    /// let result = Expectation::new(u, "u").to_have_host("example.com");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_host(&self, host: &str) -> Result<(), MatchError> {
        let is_match = self.value().host_str().is_some_and(|h| h == host);
        self.check(is_match, format!("to have host {host:?}"))
    }

    /// Asserts the URL has the given path.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the path does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use url::Url;
    ///
    /// let u = Url::parse("https://example.com/api/v1").unwrap();
    /// let result = Expectation::new(u, "u").to_have_path("/api/v1");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_path(&self, path: &str) -> Result<(), MatchError> {
        self.check(
            self.value().path() == path,
            format!("to have path {path:?}"),
        )
    }

    /// Asserts the URL has a query parameter with the given key.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the parameter is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use url::Url;
    ///
    /// let u = Url::parse("https://example.com?page=1").unwrap();
    /// let result = Expectation::new(u, "u").to_have_query_param("page");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_query_param(&self, key: &str) -> Result<(), MatchError> {
        let has = self.value().query_pairs().any(|(k, _)| k == key);
        self.check(has, format!("to have query param {key:?}"))
    }

    /// Asserts the URL has a query parameter with the given key and value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the parameter is missing or has a different value.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use url::Url;
    ///
    /// let u = Url::parse("https://example.com?page=1").unwrap();
    /// let result = Expectation::new(u, "u")
    ///     .to_have_query_param_value("page", "1");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_query_param_value(&self, key: &str, value: &str) -> Result<(), MatchError> {
        let has = self
            .value()
            .query_pairs()
            .any(|(k, v)| k == key && v == value);
        self.check(
            has,
            format!("to have query param {key:?} with value {value:?}"),
        )
    }

    /// Asserts the URL has the given fragment.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the fragment does not match or is absent.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use url::Url;
    ///
    /// let u = Url::parse("https://example.com#section").unwrap();
    /// let result = Expectation::new(u, "u").to_have_fragment("section");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_fragment(&self, fragment: &str) -> Result<(), MatchError> {
        let is_match = self.value().fragment().is_some_and(|f| f == fragment);
        self.check(is_match, format!("to have fragment {fragment:?}"))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use crate::Expectation;
    use url::Url;

    fn parse(s: &str) -> Url {
        Url::parse(s).expect("valid test URL")
    }

    // --- to_have_scheme ---

    #[test]
    fn to_have_scheme_pass() {
        assert!(Expectation::new(parse("https://example.com"), "u")
            .to_have_scheme("https")
            .is_ok());
    }

    #[test]
    fn to_have_scheme_fail() {
        assert!(Expectation::new(parse("https://example.com"), "u")
            .to_have_scheme("http")
            .is_err());
    }

    #[test]
    fn to_have_scheme_negated() {
        assert!(Expectation::new(parse("https://example.com"), "u")
            .negate()
            .to_have_scheme("http")
            .is_ok());
    }

    // --- to_have_host ---

    #[test]
    fn to_have_host_pass() {
        assert!(Expectation::new(parse("https://example.com/path"), "u")
            .to_have_host("example.com")
            .is_ok());
    }

    #[test]
    fn to_have_host_fail() {
        assert!(Expectation::new(parse("https://example.com"), "u")
            .to_have_host("other.com")
            .is_err());
    }

    #[test]
    fn to_have_host_negated() {
        assert!(Expectation::new(parse("https://example.com"), "u")
            .negate()
            .to_have_host("other.com")
            .is_ok());
    }

    // --- to_have_path ---

    #[test]
    fn to_have_path_pass() {
        assert!(Expectation::new(parse("https://example.com/api/v1"), "u")
            .to_have_path("/api/v1")
            .is_ok());
    }

    #[test]
    fn to_have_path_fail() {
        assert!(Expectation::new(parse("https://example.com/api/v1"), "u")
            .to_have_path("/api/v2")
            .is_err());
    }

    #[test]
    fn to_have_path_negated() {
        assert!(Expectation::new(parse("https://example.com/api/v1"), "u")
            .negate()
            .to_have_path("/api/v2")
            .is_ok());
    }

    // --- to_have_query_param ---

    #[test]
    fn to_have_query_param_pass() {
        assert!(Expectation::new(parse("https://example.com?page=1"), "u")
            .to_have_query_param("page")
            .is_ok());
    }

    #[test]
    fn to_have_query_param_fail() {
        assert!(Expectation::new(parse("https://example.com?page=1"), "u")
            .to_have_query_param("limit")
            .is_err());
    }

    #[test]
    fn to_have_query_param_negated() {
        assert!(Expectation::new(parse("https://example.com?page=1"), "u")
            .negate()
            .to_have_query_param("limit")
            .is_ok());
    }

    // --- to_have_query_param_value ---

    #[test]
    fn to_have_query_param_value_pass() {
        assert!(Expectation::new(parse("https://example.com?page=1"), "u")
            .to_have_query_param_value("page", "1")
            .is_ok());
    }

    #[test]
    fn to_have_query_param_value_fail_wrong() {
        assert!(Expectation::new(parse("https://example.com?page=1"), "u")
            .to_have_query_param_value("page", "2")
            .is_err());
    }

    #[test]
    fn to_have_query_param_value_fail_missing() {
        assert!(Expectation::new(parse("https://example.com"), "u")
            .to_have_query_param_value("page", "1")
            .is_err());
    }

    #[test]
    fn to_have_query_param_value_negated() {
        assert!(Expectation::new(parse("https://example.com?page=1"), "u")
            .negate()
            .to_have_query_param_value("page", "2")
            .is_ok());
    }

    // --- to_have_fragment ---

    #[test]
    fn to_have_fragment_pass() {
        assert!(Expectation::new(parse("https://example.com#section"), "u")
            .to_have_fragment("section")
            .is_ok());
    }

    #[test]
    fn to_have_fragment_fail() {
        assert!(Expectation::new(parse("https://example.com#section"), "u")
            .to_have_fragment("other")
            .is_err());
    }

    #[test]
    fn to_have_fragment_no_fragment() {
        assert!(Expectation::new(parse("https://example.com"), "u")
            .to_have_fragment("section")
            .is_err());
    }

    #[test]
    fn to_have_fragment_negated() {
        assert!(Expectation::new(parse("https://example.com#section"), "u")
            .negate()
            .to_have_fragment("other")
            .is_ok());
    }
}
