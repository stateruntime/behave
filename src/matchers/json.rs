//! Matchers for `serde_json::Value`.

use crate::error::MatchError;
use crate::expectation::Expectation;

/// Recursively checks if `actual` is a superset of `expected`.
///
/// For objects, every key in `expected` must exist in `actual` with a matching
/// value (extra keys in `actual` are allowed). Arrays and primitives require
/// exact equality.
fn json_is_superset(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
        (serde_json::Value::Object(a), serde_json::Value::Object(e)) => e
            .iter()
            .all(|(k, ev)| a.get(k).is_some_and(|av| json_is_superset(av, ev))),
        (a, e) => a == e,
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
impl Expectation<serde_json::Value> {
    /// Asserts the JSON value is an object containing the given field.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the value is not an object or lacks the field.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use serde_json::json;
    ///
    /// let v = json!({"name": "Alice"});
    /// let result = Expectation::new(v, "v").to_have_field("name");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_field(&self, field: &str) -> Result<(), MatchError> {
        let has = self
            .value()
            .as_object()
            .is_some_and(|obj| obj.contains_key(field));
        self.check(has, format!("to have field {field:?}"))
    }

    /// Asserts the JSON object has a field with the given value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the field is missing or has a different value.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use serde_json::json;
    ///
    /// let v = json!({"age": 30});
    /// let result = Expectation::new(v, "v")
    ///     .to_have_field_value("age", &json!(30));
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_field_value(
        &self,
        field: &str,
        expected: &serde_json::Value,
    ) -> Result<(), MatchError> {
        let has = self
            .value()
            .as_object()
            .and_then(|obj| obj.get(field))
            .is_some_and(|v| v == expected);
        self.check(
            has,
            format!("to have field {field:?} with value {expected}"),
        )
    }

    /// Asserts the JSON value is a superset of the expected value.
    ///
    /// For objects, every key in `expected` must exist in the actual value
    /// with a matching value (extra keys in `actual` are allowed). This is
    /// similar to Jest's `toMatchObject`.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the actual value does not contain all
    /// expected fields and values.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use serde_json::json;
    ///
    /// let actual = json!({"name": "Alice", "age": 30, "city": "NYC"});
    /// let expected = json!({"name": "Alice", "age": 30});
    /// let result = Expectation::new(actual, "v")
    ///     .to_be_json_superset_of(&expected);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_json_superset_of(&self, expected: &serde_json::Value) -> Result<(), MatchError> {
        let is_match = json_is_superset(self.value(), expected);
        self.check(is_match, format!("to be a JSON superset of {expected}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::Expectation;
    use serde_json::json;

    // --- to_have_field ---

    #[test]
    fn to_have_field_pass() {
        let v = json!({"name": "Alice"});
        assert!(Expectation::new(v, "v").to_have_field("name").is_ok());
    }

    #[test]
    fn to_have_field_fail() {
        let v = json!({"name": "Alice"});
        assert!(Expectation::new(v, "v").to_have_field("age").is_err());
    }

    #[test]
    fn to_have_field_not_object() {
        let v = json!(42);
        assert!(Expectation::new(v, "v").to_have_field("x").is_err());
    }

    #[test]
    fn to_have_field_negated() {
        let v = json!({"name": "Alice"});
        assert!(Expectation::new(v, "v")
            .negate()
            .to_have_field("age")
            .is_ok());
    }

    // --- to_have_field_value ---

    #[test]
    fn to_have_field_value_pass() {
        let v = json!({"age": 30});
        assert!(Expectation::new(v, "v")
            .to_have_field_value("age", &json!(30))
            .is_ok());
    }

    #[test]
    fn to_have_field_value_fail_wrong_value() {
        let v = json!({"age": 30});
        assert!(Expectation::new(v, "v")
            .to_have_field_value("age", &json!(25))
            .is_err());
    }

    #[test]
    fn to_have_field_value_fail_missing() {
        let v = json!({"age": 30});
        assert!(Expectation::new(v, "v")
            .to_have_field_value("name", &json!("Alice"))
            .is_err());
    }

    #[test]
    fn to_have_field_value_negated() {
        let v = json!({"age": 30});
        assert!(Expectation::new(v, "v")
            .negate()
            .to_have_field_value("age", &json!(25))
            .is_ok());
    }

    // --- to_be_json_superset_of ---

    #[test]
    fn superset_pass() {
        let actual = json!({"name": "Alice", "age": 30, "city": "NYC"});
        let expected = json!({"name": "Alice", "age": 30});
        assert!(Expectation::new(actual, "v")
            .to_be_json_superset_of(&expected)
            .is_ok());
    }

    #[test]
    fn superset_fail_missing_key() {
        let actual = json!({"name": "Alice"});
        let expected = json!({"name": "Alice", "age": 30});
        assert!(Expectation::new(actual, "v")
            .to_be_json_superset_of(&expected)
            .is_err());
    }

    #[test]
    fn superset_fail_wrong_value() {
        let actual = json!({"name": "Alice", "age": 25});
        let expected = json!({"name": "Alice", "age": 30});
        assert!(Expectation::new(actual, "v")
            .to_be_json_superset_of(&expected)
            .is_err());
    }

    #[test]
    fn superset_nested() {
        let actual = json!({"user": {"name": "Alice", "age": 30}});
        let expected = json!({"user": {"name": "Alice"}});
        assert!(Expectation::new(actual, "v")
            .to_be_json_superset_of(&expected)
            .is_ok());
    }

    #[test]
    fn superset_exact_primitives() {
        let actual = json!(42);
        let expected = json!(42);
        assert!(Expectation::new(actual, "v")
            .to_be_json_superset_of(&expected)
            .is_ok());
    }

    #[test]
    fn superset_array_exact() {
        let actual = json!([1, 2, 3]);
        let expected = json!([1, 2, 3]);
        assert!(Expectation::new(actual, "v")
            .to_be_json_superset_of(&expected)
            .is_ok());
    }

    #[test]
    fn superset_array_not_partial() {
        let actual = json!([1, 2, 3]);
        let expected = json!([1, 2]);
        assert!(Expectation::new(actual, "v")
            .to_be_json_superset_of(&expected)
            .is_err());
    }

    #[test]
    fn superset_negated() {
        let actual = json!({"name": "Alice"});
        let expected = json!({"name": "Alice", "age": 30});
        assert!(Expectation::new(actual, "v")
            .negate()
            .to_be_json_superset_of(&expected)
            .is_ok());
    }
}
