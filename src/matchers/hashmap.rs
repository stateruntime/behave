//! Matchers for `HashMap` and `BTreeMap`.

use core::fmt::Debug;
use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};

use crate::error::MatchError;
use crate::expectation::Expectation;

// ---------------------------------------------------------------------------
// HashMap
// ---------------------------------------------------------------------------

impl<K: Debug + Eq + Hash, V: Debug, S: BuildHasher> Expectation<HashMap<K, V, S>> {
    /// Asserts the map contains the given key.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the key is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use behave::Expectation;
    ///
    /// let mut m = HashMap::new();
    /// m.insert("a", 1);
    /// let result = Expectation::new(m, "m").to_contain_key(&"a");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_key(&self, key: &K) -> Result<(), MatchError> {
        self.check(
            self.value().contains_key(key),
            format!("to contain key {key:?}"),
        )
    }

    /// Asserts the map is empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the map is not empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use behave::Expectation;
    ///
    /// let m: HashMap<String, i32> = HashMap::new();
    /// let result = Expectation::new(m, "m").to_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_empty(&self) -> Result<(), MatchError> {
        self.check(self.value().is_empty(), "to be empty")
    }

    /// Asserts the map is not empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the map is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use behave::Expectation;
    ///
    /// let mut m = HashMap::new();
    /// m.insert("a", 1);
    /// let result = Expectation::new(m, "m").to_not_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_not_be_empty(&self) -> Result<(), MatchError> {
        self.check(!self.value().is_empty(), "to not be empty")
    }

    /// Asserts the map has exactly the given number of entries.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the length does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use behave::Expectation;
    ///
    /// let mut m = HashMap::new();
    /// m.insert("a", 1);
    /// m.insert("b", 2);
    /// let result = Expectation::new(m, "m").to_have_length(2);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_length(&self, expected: usize) -> Result<(), MatchError> {
        self.check(
            self.value().len() == expected,
            format!("to have length {expected}"),
        )
    }
}

impl<K: Debug + Eq + Hash, V: Debug + PartialEq, S: BuildHasher> Expectation<HashMap<K, V, S>> {
    /// Asserts the map contains the given value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the value is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use behave::Expectation;
    ///
    /// let mut m = HashMap::new();
    /// m.insert("a", 1);
    /// let result = Expectation::new(m, "m").to_contain_value(&1);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_value(&self, value: &V) -> Result<(), MatchError> {
        let has = self.value().values().any(|v| v == value);
        self.check(has, format!("to contain value {value:?}"))
    }

    /// Asserts the map contains the given key-value pair.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the entry is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use behave::Expectation;
    ///
    /// let mut m = HashMap::new();
    /// m.insert("a", 1);
    /// let result = Expectation::new(m, "m").to_contain_entry(&"a", &1);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_entry(&self, key: &K, value: &V) -> Result<(), MatchError> {
        let has = self.value().get(key).is_some_and(|v| v == value);
        self.check(has, format!("to contain entry ({key:?}, {value:?})"))
    }
}

// ---------------------------------------------------------------------------
// BTreeMap
// ---------------------------------------------------------------------------

impl<K: Debug + Ord, V: Debug> Expectation<BTreeMap<K, V>> {
    /// Asserts the map contains the given key.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the key is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use behave::Expectation;
    ///
    /// let mut m = BTreeMap::new();
    /// m.insert("a", 1);
    /// let result = Expectation::new(m, "m").to_contain_key(&"a");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_key(&self, key: &K) -> Result<(), MatchError> {
        self.check(
            self.value().contains_key(key),
            format!("to contain key {key:?}"),
        )
    }

    /// Asserts the map is empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the map is not empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use behave::Expectation;
    ///
    /// let m: BTreeMap<String, i32> = BTreeMap::new();
    /// let result = Expectation::new(m, "m").to_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_empty(&self) -> Result<(), MatchError> {
        self.check(self.value().is_empty(), "to be empty")
    }

    /// Asserts the map is not empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the map is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use behave::Expectation;
    ///
    /// let mut m = BTreeMap::new();
    /// m.insert("a", 1);
    /// let result = Expectation::new(m, "m").to_not_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_not_be_empty(&self) -> Result<(), MatchError> {
        self.check(!self.value().is_empty(), "to not be empty")
    }

    /// Asserts the map has exactly the given number of entries.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the length does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use behave::Expectation;
    ///
    /// let mut m = BTreeMap::new();
    /// m.insert("a", 1);
    /// m.insert("b", 2);
    /// let result = Expectation::new(m, "m").to_have_length(2);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_length(&self, expected: usize) -> Result<(), MatchError> {
        self.check(
            self.value().len() == expected,
            format!("to have length {expected}"),
        )
    }
}

impl<K: Debug + Ord, V: Debug + PartialEq> Expectation<BTreeMap<K, V>> {
    /// Asserts the map contains the given value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the value is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use behave::Expectation;
    ///
    /// let mut m = BTreeMap::new();
    /// m.insert("a", 1);
    /// let result = Expectation::new(m, "m").to_contain_value(&1);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_value(&self, value: &V) -> Result<(), MatchError> {
        let has = self.value().values().any(|v| v == value);
        self.check(has, format!("to contain value {value:?}"))
    }

    /// Asserts the map contains the given key-value pair.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the entry is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use behave::Expectation;
    ///
    /// let mut m = BTreeMap::new();
    /// m.insert("a", 1);
    /// let result = Expectation::new(m, "m").to_contain_entry(&"a", &1);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_entry(&self, key: &K, value: &V) -> Result<(), MatchError> {
        let has = self.value().get(key).is_some_and(|v| v == value);
        self.check(has, format!("to contain entry ({key:?}, {value:?})"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Expectation;

    // -- HashMap --

    #[test]
    fn hashmap_to_contain_key_pass() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_key(&"a").is_ok());
    }

    #[test]
    fn hashmap_to_contain_key_fail() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_key(&"b").is_err());
    }

    #[test]
    fn hashmap_to_contain_key_negated() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m")
            .negate()
            .to_contain_key(&"b")
            .is_ok());
    }

    #[test]
    fn hashmap_to_contain_value_pass() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_value(&1).is_ok());
    }

    #[test]
    fn hashmap_to_contain_value_fail() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_value(&2).is_err());
    }

    #[test]
    fn hashmap_to_contain_entry_pass() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_entry(&"a", &1).is_ok());
    }

    #[test]
    fn hashmap_to_contain_entry_wrong_value() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_entry(&"a", &2).is_err());
    }

    #[test]
    fn hashmap_to_contain_entry_missing_key() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_entry(&"b", &1).is_err());
    }

    #[test]
    fn hashmap_to_be_empty_pass() {
        let m: HashMap<String, i32> = HashMap::new();
        assert!(Expectation::new(m, "m").to_be_empty().is_ok());
    }

    #[test]
    fn hashmap_to_be_empty_fail() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_be_empty().is_err());
    }

    #[test]
    fn hashmap_to_not_be_empty_pass() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_not_be_empty().is_ok());
    }

    #[test]
    fn hashmap_to_not_be_empty_fail() {
        let m: HashMap<String, i32> = HashMap::new();
        assert!(Expectation::new(m, "m").to_not_be_empty().is_err());
    }

    #[test]
    fn hashmap_to_have_length_pass() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        m.insert("b", 2);
        assert!(Expectation::new(m, "m").to_have_length(2).is_ok());
    }

    #[test]
    fn hashmap_to_have_length_fail() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_have_length(2).is_err());
    }

    // -- BTreeMap --

    #[test]
    fn btreemap_to_contain_key_pass() {
        let mut m = BTreeMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_key(&"a").is_ok());
    }

    #[test]
    fn btreemap_to_contain_key_fail() {
        let mut m = BTreeMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_key(&"b").is_err());
    }

    #[test]
    fn btreemap_to_contain_value_pass() {
        let mut m = BTreeMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_value(&1).is_ok());
    }

    #[test]
    fn btreemap_to_contain_value_fail() {
        let mut m = BTreeMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_value(&2).is_err());
    }

    #[test]
    fn btreemap_to_contain_entry_pass() {
        let mut m = BTreeMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_entry(&"a", &1).is_ok());
    }

    #[test]
    fn btreemap_to_contain_entry_fail() {
        let mut m = BTreeMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_contain_entry(&"a", &2).is_err());
    }

    #[test]
    fn btreemap_to_be_empty_pass() {
        let m: BTreeMap<String, i32> = BTreeMap::new();
        assert!(Expectation::new(m, "m").to_be_empty().is_ok());
    }

    #[test]
    fn btreemap_to_be_empty_fail() {
        let mut m = BTreeMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_be_empty().is_err());
    }

    #[test]
    fn btreemap_to_not_be_empty_pass() {
        let mut m = BTreeMap::new();
        m.insert("a", 1);
        assert!(Expectation::new(m, "m").to_not_be_empty().is_ok());
    }

    #[test]
    fn btreemap_to_have_length_pass() {
        let mut m = BTreeMap::new();
        m.insert("a", 1);
        m.insert("b", 2);
        assert!(Expectation::new(m, "m").to_have_length(2).is_ok());
    }
}
