//! Matchers for HTTP status codes and headers.

use crate::error::MatchError;
use crate::expectation::Expectation;

// ---------------------------------------------------------------------------
// StatusCode
// ---------------------------------------------------------------------------

#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
impl Expectation<::http::StatusCode> {
    /// Asserts the status code is a success (2xx).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the status code is not in the 200-299 range.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use http::StatusCode;
    ///
    /// let result = Expectation::new(StatusCode::OK, "s").to_be_success();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_success(&self) -> Result<(), MatchError> {
        self.check(self.value().is_success(), "to be success (2xx)")
    }

    /// Asserts the status code is a redirect (3xx).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the status code is not in the 300-399 range.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use http::StatusCode;
    ///
    /// let result = Expectation::new(StatusCode::MOVED_PERMANENTLY, "s")
    ///     .to_be_redirect();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_redirect(&self) -> Result<(), MatchError> {
        self.check(self.value().is_redirection(), "to be redirect (3xx)")
    }

    /// Asserts the status code is a client error (4xx).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the status code is not in the 400-499 range.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use http::StatusCode;
    ///
    /// let result = Expectation::new(StatusCode::NOT_FOUND, "s")
    ///     .to_be_client_error();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_client_error(&self) -> Result<(), MatchError> {
        self.check(self.value().is_client_error(), "to be client error (4xx)")
    }

    /// Asserts the status code is a server error (5xx).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the status code is not in the 500-599 range.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use http::StatusCode;
    ///
    /// let result = Expectation::new(
    ///     StatusCode::INTERNAL_SERVER_ERROR, "s",
    /// ).to_be_server_error();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_server_error(&self) -> Result<(), MatchError> {
        self.check(self.value().is_server_error(), "to be server error (5xx)")
    }

    /// Asserts the status code has the given numeric value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the status code does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use http::StatusCode;
    ///
    /// let result = Expectation::new(StatusCode::OK, "s")
    ///     .to_have_status_code(200);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_status_code(&self, code: u16) -> Result<(), MatchError> {
        self.check(
            self.value().as_u16() == code,
            format!("to have status code {code}"),
        )
    }
}

// ---------------------------------------------------------------------------
// HeaderMap
// ---------------------------------------------------------------------------

#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
impl Expectation<::http::HeaderMap> {
    /// Asserts the header map contains the given header name.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the header is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use http::HeaderMap;
    ///
    /// let mut headers = HeaderMap::new();
    /// headers.insert("content-type", "text/plain".parse().unwrap());
    /// let result = Expectation::new(headers, "h").to_have_header("content-type");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_header(&self, name: &str) -> Result<(), MatchError> {
        self.check(
            self.value().contains_key(name),
            format!("to have header {name:?}"),
        )
    }

    /// Asserts the header map contains a header with the given value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the header is missing or has a different value.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use http::HeaderMap;
    ///
    /// let mut headers = HeaderMap::new();
    /// headers.insert("content-type", "text/plain".parse().unwrap());
    /// let result = Expectation::new(headers, "h")
    ///     .to_have_header_value("content-type", "text/plain");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_header_value(&self, name: &str, value: &str) -> Result<(), MatchError> {
        let has = self
            .value()
            .get(name)
            .is_some_and(|v| v.as_bytes() == value.as_bytes());
        self.check(has, format!("to have header {name:?} with value {value:?}"))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use crate::Expectation;
    use http::{HeaderMap, StatusCode};

    // --- StatusCode ---

    #[test]
    fn to_be_success_pass() {
        assert!(Expectation::new(StatusCode::OK, "s")
            .to_be_success()
            .is_ok());
    }

    #[test]
    fn to_be_success_fail() {
        assert!(Expectation::new(StatusCode::NOT_FOUND, "s")
            .to_be_success()
            .is_err());
    }

    #[test]
    fn to_be_success_negated() {
        assert!(Expectation::new(StatusCode::NOT_FOUND, "s")
            .negate()
            .to_be_success()
            .is_ok());
    }

    #[test]
    fn to_be_redirect_pass() {
        assert!(Expectation::new(StatusCode::MOVED_PERMANENTLY, "s")
            .to_be_redirect()
            .is_ok());
    }

    #[test]
    fn to_be_redirect_fail() {
        assert!(Expectation::new(StatusCode::OK, "s")
            .to_be_redirect()
            .is_err());
    }

    #[test]
    fn to_be_redirect_negated() {
        assert!(Expectation::new(StatusCode::OK, "s")
            .negate()
            .to_be_redirect()
            .is_ok());
    }

    #[test]
    fn to_be_client_error_pass() {
        assert!(Expectation::new(StatusCode::NOT_FOUND, "s")
            .to_be_client_error()
            .is_ok());
    }

    #[test]
    fn to_be_client_error_fail() {
        assert!(Expectation::new(StatusCode::OK, "s")
            .to_be_client_error()
            .is_err());
    }

    #[test]
    fn to_be_client_error_negated() {
        assert!(Expectation::new(StatusCode::OK, "s")
            .negate()
            .to_be_client_error()
            .is_ok());
    }

    #[test]
    fn to_be_server_error_pass() {
        assert!(Expectation::new(StatusCode::INTERNAL_SERVER_ERROR, "s")
            .to_be_server_error()
            .is_ok());
    }

    #[test]
    fn to_be_server_error_fail() {
        assert!(Expectation::new(StatusCode::OK, "s")
            .to_be_server_error()
            .is_err());
    }

    #[test]
    fn to_be_server_error_negated() {
        assert!(Expectation::new(StatusCode::OK, "s")
            .negate()
            .to_be_server_error()
            .is_ok());
    }

    #[test]
    fn to_have_status_code_pass() {
        assert!(Expectation::new(StatusCode::OK, "s")
            .to_have_status_code(200)
            .is_ok());
    }

    #[test]
    fn to_have_status_code_fail() {
        assert!(Expectation::new(StatusCode::OK, "s")
            .to_have_status_code(404)
            .is_err());
    }

    #[test]
    fn to_have_status_code_negated() {
        assert!(Expectation::new(StatusCode::OK, "s")
            .negate()
            .to_have_status_code(404)
            .is_ok());
    }

    // --- HeaderMap ---

    #[test]
    fn to_have_header_pass() {
        let mut h = HeaderMap::new();
        h.insert("content-type", "text/plain".parse().expect("valid header"));
        assert!(Expectation::new(h, "h")
            .to_have_header("content-type")
            .is_ok());
    }

    #[test]
    fn to_have_header_fail() {
        let h = HeaderMap::new();
        assert!(Expectation::new(h, "h")
            .to_have_header("content-type")
            .is_err());
    }

    #[test]
    fn to_have_header_negated() {
        let h = HeaderMap::new();
        assert!(Expectation::new(h, "h")
            .negate()
            .to_have_header("content-type")
            .is_ok());
    }

    #[test]
    fn to_have_header_value_pass() {
        let mut h = HeaderMap::new();
        h.insert("content-type", "text/plain".parse().expect("valid header"));
        assert!(Expectation::new(h, "h")
            .to_have_header_value("content-type", "text/plain")
            .is_ok());
    }

    #[test]
    fn to_have_header_value_fail_wrong() {
        let mut h = HeaderMap::new();
        h.insert("content-type", "text/plain".parse().expect("valid header"));
        assert!(Expectation::new(h, "h")
            .to_have_header_value("content-type", "application/json")
            .is_err());
    }

    #[test]
    fn to_have_header_value_fail_missing() {
        let h = HeaderMap::new();
        assert!(Expectation::new(h, "h")
            .to_have_header_value("content-type", "text/plain")
            .is_err());
    }

    #[test]
    fn to_have_header_value_negated() {
        let mut h = HeaderMap::new();
        h.insert("content-type", "text/plain".parse().expect("valid header"));
        assert!(Expectation::new(h, "h")
            .negate()
            .to_have_header_value("content-type", "application/json")
            .is_ok());
    }
}
