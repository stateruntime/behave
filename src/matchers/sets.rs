//! Matchers for `HashSet` and `BTreeSet`.

use core::fmt::Debug;
use std::collections::{BTreeSet, HashSet};
use std::hash::{BuildHasher, Hash};

use crate::error::MatchError;
use crate::expectation::Expectation;

// ---------------------------------------------------------------------------
// HashSet
// ---------------------------------------------------------------------------

impl<T: Debug + Eq + Hash, S: BuildHasher> Expectation<HashSet<T, S>> {
    /// Asserts the set contains the given element.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the element is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use behave::Expectation;
    ///
    /// let s: HashSet<i32> = [1, 2, 3].into_iter().collect();
    /// let result = Expectation::new(s, "s").to_contain(&2);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain(&self, element: &T) -> Result<(), MatchError> {
        self.check(
            self.value().contains(element),
            format!("to contain {element:?}"),
        )
    }

    /// Asserts the set is empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the set is not empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use behave::Expectation;
    ///
    /// let s: HashSet<i32> = HashSet::new();
    /// let result = Expectation::new(s, "s").to_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_empty(&self) -> Result<(), MatchError> {
        self.check(self.value().is_empty(), "to be empty")
    }

    /// Asserts the set is not empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use behave::Expectation;
    ///
    /// let s: HashSet<i32> = std::iter::once(1).collect();
    /// let result = Expectation::new(s, "s").to_not_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_not_be_empty(&self) -> Result<(), MatchError> {
        self.check(!self.value().is_empty(), "to not be empty")
    }

    /// Asserts the set has exactly the given number of elements.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the length does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use behave::Expectation;
    ///
    /// let s: HashSet<i32> = [1, 2].into_iter().collect();
    /// let result = Expectation::new(s, "s").to_have_length(2);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_length(&self, expected: usize) -> Result<(), MatchError> {
        self.check(
            self.value().len() == expected,
            format!("to have length {expected}"),
        )
    }

    /// Asserts this set is a subset of the given set.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any element is missing from `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use behave::Expectation;
    ///
    /// let s: HashSet<i32> = [1, 2].into_iter().collect();
    /// let superset: HashSet<i32> = [1, 2, 3].into_iter().collect();
    /// let result = Expectation::new(s, "s").to_be_subset_of(&superset);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_subset_of(&self, other: &HashSet<T, S>) -> Result<(), MatchError> {
        self.check(self.value().is_subset(other), "to be a subset")
    }

    /// Asserts this set is a superset of the given set.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any element from `other` is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use behave::Expectation;
    ///
    /// let s: HashSet<i32> = [1, 2, 3].into_iter().collect();
    /// let subset: HashSet<i32> = [1, 2].into_iter().collect();
    /// let result = Expectation::new(s, "s").to_be_superset_of(&subset);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_superset_of(&self, other: &HashSet<T, S>) -> Result<(), MatchError> {
        self.check(self.value().is_superset(other), "to be a superset")
    }
}

// ---------------------------------------------------------------------------
// BTreeSet
// ---------------------------------------------------------------------------

impl<T: Debug + Ord> Expectation<BTreeSet<T>> {
    /// Asserts the set contains the given element.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the element is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    /// use behave::Expectation;
    ///
    /// let s: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
    /// let result = Expectation::new(s, "s").to_contain(&2);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain(&self, element: &T) -> Result<(), MatchError> {
        self.check(
            self.value().contains(element),
            format!("to contain {element:?}"),
        )
    }

    /// Asserts the set is empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the set is not empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    /// use behave::Expectation;
    ///
    /// let s: BTreeSet<i32> = BTreeSet::new();
    /// let result = Expectation::new(s, "s").to_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_empty(&self) -> Result<(), MatchError> {
        self.check(self.value().is_empty(), "to be empty")
    }

    /// Asserts the set is not empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    /// use behave::Expectation;
    ///
    /// let s: BTreeSet<i32> = std::iter::once(1).collect();
    /// let result = Expectation::new(s, "s").to_not_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_not_be_empty(&self) -> Result<(), MatchError> {
        self.check(!self.value().is_empty(), "to not be empty")
    }

    /// Asserts the set has exactly the given number of elements.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the length does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    /// use behave::Expectation;
    ///
    /// let s: BTreeSet<i32> = [1, 2].into_iter().collect();
    /// let result = Expectation::new(s, "s").to_have_length(2);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_length(&self, expected: usize) -> Result<(), MatchError> {
        self.check(
            self.value().len() == expected,
            format!("to have length {expected}"),
        )
    }

    /// Asserts this set is a subset of the given set.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any element is missing from `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    /// use behave::Expectation;
    ///
    /// let s: BTreeSet<i32> = [1, 2].into_iter().collect();
    /// let superset: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
    /// let result = Expectation::new(s, "s").to_be_subset_of(&superset);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_subset_of(&self, other: &BTreeSet<T>) -> Result<(), MatchError> {
        self.check(self.value().is_subset(other), "to be a subset")
    }

    /// Asserts this set is a superset of the given set.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any element from `other` is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    /// use behave::Expectation;
    ///
    /// let s: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
    /// let subset: BTreeSet<i32> = [1, 2].into_iter().collect();
    /// let result = Expectation::new(s, "s").to_be_superset_of(&subset);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_superset_of(&self, other: &BTreeSet<T>) -> Result<(), MatchError> {
        self.check(self.value().is_superset(other), "to be a superset")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Expectation;

    // -- HashSet --

    #[test]
    fn hashset_to_contain_pass() {
        let s: HashSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s").to_contain(&2).is_ok());
    }

    #[test]
    fn hashset_to_contain_fail() {
        let s: HashSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s").to_contain(&9).is_err());
    }

    #[test]
    fn hashset_to_contain_negated() {
        let s: HashSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s").negate().to_contain(&9).is_ok());
    }

    #[test]
    fn hashset_to_be_empty_pass() {
        let s: HashSet<i32> = HashSet::new();
        assert!(Expectation::new(s, "s").to_be_empty().is_ok());
    }

    #[test]
    fn hashset_to_be_empty_fail() {
        let s: HashSet<i32> = std::iter::once(1).collect();
        assert!(Expectation::new(s, "s").to_be_empty().is_err());
    }

    #[test]
    fn hashset_to_not_be_empty_pass() {
        let s: HashSet<i32> = std::iter::once(1).collect();
        assert!(Expectation::new(s, "s").to_not_be_empty().is_ok());
    }

    #[test]
    fn hashset_to_not_be_empty_fail() {
        let s: HashSet<i32> = HashSet::new();
        assert!(Expectation::new(s, "s").to_not_be_empty().is_err());
    }

    #[test]
    fn hashset_to_have_length_pass() {
        let s: HashSet<i32> = [1, 2].into_iter().collect();
        assert!(Expectation::new(s, "s").to_have_length(2).is_ok());
    }

    #[test]
    fn hashset_to_have_length_fail() {
        let s: HashSet<i32> = std::iter::once(1).collect();
        assert!(Expectation::new(s, "s").to_have_length(2).is_err());
    }

    #[test]
    fn hashset_to_be_subset_of_pass() {
        let s: HashSet<i32> = [1, 2].into_iter().collect();
        let sup: HashSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s").to_be_subset_of(&sup).is_ok());
    }

    #[test]
    fn hashset_to_be_subset_of_fail() {
        let s: HashSet<i32> = [1, 4].into_iter().collect();
        let sup: HashSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s").to_be_subset_of(&sup).is_err());
    }

    #[test]
    fn hashset_to_be_subset_of_negated() {
        let s: HashSet<i32> = [1, 4].into_iter().collect();
        let sup: HashSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s")
            .negate()
            .to_be_subset_of(&sup)
            .is_ok());
    }

    #[test]
    fn hashset_to_be_superset_of_pass() {
        let s: HashSet<i32> = [1, 2, 3].into_iter().collect();
        let sub: HashSet<i32> = [1, 2].into_iter().collect();
        assert!(Expectation::new(s, "s").to_be_superset_of(&sub).is_ok());
    }

    #[test]
    fn hashset_to_be_superset_of_fail() {
        let s: HashSet<i32> = [1, 2].into_iter().collect();
        let sub: HashSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s").to_be_superset_of(&sub).is_err());
    }

    #[test]
    fn hashset_to_be_superset_of_negated() {
        let s: HashSet<i32> = [1, 2].into_iter().collect();
        let sub: HashSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s")
            .negate()
            .to_be_superset_of(&sub)
            .is_ok());
    }

    // -- BTreeSet --

    #[test]
    fn btreeset_to_contain_pass() {
        let s: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s").to_contain(&2).is_ok());
    }

    #[test]
    fn btreeset_to_contain_fail() {
        let s: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s").to_contain(&9).is_err());
    }

    #[test]
    fn btreeset_to_be_empty_pass() {
        let s: BTreeSet<i32> = BTreeSet::new();
        assert!(Expectation::new(s, "s").to_be_empty().is_ok());
    }

    #[test]
    fn btreeset_to_be_empty_fail() {
        let s: BTreeSet<i32> = std::iter::once(1).collect();
        assert!(Expectation::new(s, "s").to_be_empty().is_err());
    }

    #[test]
    fn btreeset_to_not_be_empty_pass() {
        let s: BTreeSet<i32> = std::iter::once(1).collect();
        assert!(Expectation::new(s, "s").to_not_be_empty().is_ok());
    }

    #[test]
    fn btreeset_to_have_length_pass() {
        let s: BTreeSet<i32> = [1, 2].into_iter().collect();
        assert!(Expectation::new(s, "s").to_have_length(2).is_ok());
    }

    #[test]
    fn btreeset_to_be_subset_of_pass() {
        let s: BTreeSet<i32> = [1, 2].into_iter().collect();
        let sup: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s").to_be_subset_of(&sup).is_ok());
    }

    #[test]
    fn btreeset_to_be_subset_of_fail() {
        let s: BTreeSet<i32> = [1, 4].into_iter().collect();
        let sup: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s").to_be_subset_of(&sup).is_err());
    }

    #[test]
    fn btreeset_to_be_superset_of_pass() {
        let s: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
        let sub: BTreeSet<i32> = [1, 2].into_iter().collect();
        assert!(Expectation::new(s, "s").to_be_superset_of(&sub).is_ok());
    }

    #[test]
    fn btreeset_to_be_superset_of_fail() {
        let s: BTreeSet<i32> = [1, 2].into_iter().collect();
        let sub: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(Expectation::new(s, "s").to_be_superset_of(&sub).is_err());
    }
}
