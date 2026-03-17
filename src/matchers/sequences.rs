//! Sequence matchers for ordered collection assertions.

use core::fmt::Debug;

use crate::error::MatchError;
use crate::expectation::Expectation;

fn is_sorted_ascending<T: PartialOrd>(slice: &[T]) -> bool {
    slice.windows(2).all(|w| w[0] <= w[1])
}

// ---------------------------------------------------------------------------
// Vec<T>
// ---------------------------------------------------------------------------

impl<T: Debug + PartialEq> Expectation<Vec<T>> {
    /// Asserts the vector contains exactly the given elements in order.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the elements differ or are in a different order.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(vec![1, 2, 3], "v")
    ///     .to_contain_exactly(&[1, 2, 3]);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_exactly(&self, expected: &[T]) -> Result<(), MatchError> {
        let is_match = self.value().as_slice() == expected;
        self.check(is_match, format!("to contain exactly {expected:?}"))
    }

    /// Asserts the vector contains the same elements in any order.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the elements differ regardless of order.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(vec![3, 1, 2], "v")
    ///     .to_contain_exactly_in_any_order(&[1, 2, 3]);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_exactly_in_any_order(&self, expected: &[T]) -> Result<(), MatchError> {
        let actual = self.value();
        let is_match = actual.len() == expected.len()
            && expected.iter().all(|e| {
                let expected_count = expected.iter().filter(|x| *x == e).count();
                let actual_count = actual.iter().filter(|x| *x == e).count();
                expected_count == actual_count
            });
        self.check(
            is_match,
            format!("to contain exactly in any order {expected:?}"),
        )
    }

    /// Asserts the vector starts with the given elements.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the prefix does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(vec![1, 2, 3], "v")
    ///     .to_start_with_elements(&[1, 2]);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_start_with_elements(&self, prefix: &[T]) -> Result<(), MatchError> {
        let is_match = self.value().starts_with(prefix);
        self.check(is_match, format!("to start with elements {prefix:?}"))
    }

    /// Asserts the vector ends with the given elements.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the suffix does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(vec![1, 2, 3], "v")
    ///     .to_end_with_elements(&[2, 3]);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_end_with_elements(&self, suffix: &[T]) -> Result<(), MatchError> {
        let is_match = self.value().ends_with(suffix);
        self.check(is_match, format!("to end with elements {suffix:?}"))
    }
}

impl<T: Debug> Expectation<Vec<T>> {
    /// Asserts the vector is sorted by a key extraction function.
    ///
    /// The extracted keys must be in non-descending order.
    /// An empty or single-element vector is considered sorted.
    /// The `desc` argument appears in failure messages.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any adjacent pair of keys is out of order.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(vec!["a", "bb", "ccc"], "v")
    ///     .to_be_sorted_by_key(|s| s.len(), "by length");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_sorted_by_key<K: PartialOrd>(
        &self,
        f: impl Fn(&T) -> K,
        desc: &str,
    ) -> Result<(), MatchError> {
        let keys: Vec<K> = self.value().iter().map(&f).collect();
        let is_match = is_sorted_ascending(&keys);
        self.check(is_match, format!("to be sorted {desc}"))
    }
}

impl<T: Debug + PartialOrd> Expectation<Vec<T>> {
    /// Asserts the vector is sorted in non-descending order.
    ///
    /// An empty vector and a single-element vector are considered sorted.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any adjacent pair is out of order.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(vec![1, 2, 3], "v").to_be_sorted();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_sorted(&self) -> Result<(), MatchError> {
        self.check(is_sorted_ascending(self.value()), "to be sorted")
    }
}

// ---------------------------------------------------------------------------
// &[T]
// ---------------------------------------------------------------------------

impl<T: Debug + PartialEq> Expectation<&[T]> {
    /// Asserts the slice contains exactly the given elements in order.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the elements differ or are in a different order.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let s: &[i32] = &[1, 2, 3];
    /// let result = Expectation::new(s, "s").to_contain_exactly(&[1, 2, 3]);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_exactly(&self, expected: &[T]) -> Result<(), MatchError> {
        let is_match = *self.value() == expected;
        self.check(is_match, format!("to contain exactly {expected:?}"))
    }

    /// Asserts the slice contains the same elements in any order.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the elements differ regardless of order.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let s: &[i32] = &[3, 1, 2];
    /// let result = Expectation::new(s, "s")
    ///     .to_contain_exactly_in_any_order(&[1, 2, 3]);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_exactly_in_any_order(&self, expected: &[T]) -> Result<(), MatchError> {
        let actual = *self.value();
        let is_match = actual.len() == expected.len()
            && expected.iter().all(|e| {
                let expected_count = expected.iter().filter(|x| *x == e).count();
                let actual_count = actual.iter().filter(|x| *x == e).count();
                expected_count == actual_count
            });
        self.check(
            is_match,
            format!("to contain exactly in any order {expected:?}"),
        )
    }

    /// Asserts the slice starts with the given elements.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the prefix does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let s: &[i32] = &[1, 2, 3];
    /// let result = Expectation::new(s, "s").to_start_with_elements(&[1, 2]);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_start_with_elements(&self, prefix: &[T]) -> Result<(), MatchError> {
        let is_match = self.value().starts_with(prefix);
        self.check(is_match, format!("to start with elements {prefix:?}"))
    }

    /// Asserts the slice ends with the given elements.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the suffix does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let s: &[i32] = &[1, 2, 3];
    /// let result = Expectation::new(s, "s").to_end_with_elements(&[2, 3]);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_end_with_elements(&self, suffix: &[T]) -> Result<(), MatchError> {
        let is_match = self.value().ends_with(suffix);
        self.check(is_match, format!("to end with elements {suffix:?}"))
    }
}

impl<T: Debug> Expectation<&[T]> {
    /// Asserts the slice is sorted by a key extraction function.
    ///
    /// The extracted keys must be in non-descending order.
    /// An empty or single-element slice is considered sorted.
    /// The `desc` argument appears in failure messages.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any adjacent pair of keys is out of order.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let s: &[&str] = &["a", "bb", "ccc"];
    /// let result = Expectation::new(s, "s")
    ///     .to_be_sorted_by_key(|s| s.len(), "by length");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_sorted_by_key<K: PartialOrd>(
        &self,
        f: impl Fn(&T) -> K,
        desc: &str,
    ) -> Result<(), MatchError> {
        let keys: Vec<K> = self.value().iter().map(&f).collect();
        let is_match = is_sorted_ascending(&keys);
        self.check(is_match, format!("to be sorted {desc}"))
    }
}

impl<T: Debug + PartialOrd> Expectation<&[T]> {
    /// Asserts the slice is sorted in non-descending order.
    ///
    /// An empty slice and a single-element slice are considered sorted.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any adjacent pair is out of order.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let s: &[i32] = &[1, 2, 3];
    /// let result = Expectation::new(s, "s").to_be_sorted();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_sorted(&self) -> Result<(), MatchError> {
        self.check(is_sorted_ascending(self.value()), "to be sorted")
    }
}

#[cfg(test)]
mod tests {
    use crate::Expectation;

    // --- Vec: to_contain_exactly ---

    #[test]
    fn vec_to_contain_exactly_pass() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_contain_exactly(&[1, 2, 3])
            .is_ok());
    }

    #[test]
    fn vec_to_contain_exactly_fail_order() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_contain_exactly(&[3, 2, 1])
            .is_err());
    }

    #[test]
    fn vec_to_contain_exactly_fail_length() {
        assert!(Expectation::new(vec![1, 2], "v")
            .to_contain_exactly(&[1, 2, 3])
            .is_err());
    }

    #[test]
    fn vec_to_contain_exactly_negated() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .negate()
            .to_contain_exactly(&[3, 2, 1])
            .is_ok());
    }

    #[test]
    fn vec_to_contain_exactly_empty() {
        let v: Vec<i32> = vec![];
        assert!(Expectation::new(v, "v").to_contain_exactly(&[]).is_ok());
    }

    // --- Vec: to_contain_exactly_in_any_order ---

    #[test]
    fn vec_to_contain_exactly_in_any_order_pass() {
        assert!(Expectation::new(vec![3, 1, 2], "v")
            .to_contain_exactly_in_any_order(&[1, 2, 3])
            .is_ok());
    }

    #[test]
    fn vec_to_contain_exactly_in_any_order_fail() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_contain_exactly_in_any_order(&[1, 2, 4])
            .is_err());
    }

    #[test]
    fn vec_to_contain_exactly_in_any_order_diff_length() {
        assert!(Expectation::new(vec![1, 2], "v")
            .to_contain_exactly_in_any_order(&[1, 2, 3])
            .is_err());
    }

    #[test]
    fn vec_to_contain_exactly_in_any_order_duplicates() {
        assert!(Expectation::new(vec![1, 1, 2], "v")
            .to_contain_exactly_in_any_order(&[1, 2, 1])
            .is_ok());
    }

    #[test]
    fn vec_to_contain_exactly_in_any_order_dup_mismatch() {
        assert!(Expectation::new(vec![1, 1, 2], "v")
            .to_contain_exactly_in_any_order(&[1, 2, 2])
            .is_err());
    }

    #[test]
    fn vec_to_contain_exactly_in_any_order_negated() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .negate()
            .to_contain_exactly_in_any_order(&[1, 2, 4])
            .is_ok());
    }

    // --- Vec: to_start_with_elements ---

    #[test]
    fn vec_to_start_with_elements_pass() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_start_with_elements(&[1, 2])
            .is_ok());
    }

    #[test]
    fn vec_to_start_with_elements_fail() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_start_with_elements(&[2, 3])
            .is_err());
    }

    #[test]
    fn vec_to_start_with_elements_negated() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .negate()
            .to_start_with_elements(&[2, 3])
            .is_ok());
    }

    #[test]
    fn vec_to_start_with_elements_empty() {
        assert!(Expectation::new(vec![1, 2], "v")
            .to_start_with_elements(&[])
            .is_ok());
    }

    // --- Vec: to_end_with_elements ---

    #[test]
    fn vec_to_end_with_elements_pass() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_end_with_elements(&[2, 3])
            .is_ok());
    }

    #[test]
    fn vec_to_end_with_elements_fail() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_end_with_elements(&[1, 2])
            .is_err());
    }

    #[test]
    fn vec_to_end_with_elements_negated() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .negate()
            .to_end_with_elements(&[1, 2])
            .is_ok());
    }

    // --- Vec: to_be_sorted ---

    #[test]
    fn vec_to_be_sorted_pass() {
        assert!(Expectation::new(vec![1, 2, 3], "v").to_be_sorted().is_ok());
    }

    #[test]
    fn vec_to_be_sorted_equal() {
        assert!(Expectation::new(vec![1, 1, 2], "v").to_be_sorted().is_ok());
    }

    #[test]
    fn vec_to_be_sorted_fail() {
        assert!(Expectation::new(vec![3, 1, 2], "v").to_be_sorted().is_err());
    }

    #[test]
    fn vec_to_be_sorted_empty() {
        let v: Vec<i32> = vec![];
        assert!(Expectation::new(v, "v").to_be_sorted().is_ok());
    }

    #[test]
    fn vec_to_be_sorted_single() {
        assert!(Expectation::new(vec![42], "v").to_be_sorted().is_ok());
    }

    #[test]
    fn vec_to_be_sorted_negated() {
        assert!(Expectation::new(vec![3, 1, 2], "v")
            .negate()
            .to_be_sorted()
            .is_ok());
    }

    // --- Slice variants ---

    #[test]
    fn slice_to_contain_exactly_pass() {
        let s: &[i32] = &[1, 2, 3];
        assert!(Expectation::new(s, "s")
            .to_contain_exactly(&[1, 2, 3])
            .is_ok());
    }

    #[test]
    fn slice_to_contain_exactly_fail() {
        let s: &[i32] = &[1, 2, 3];
        assert!(Expectation::new(s, "s")
            .to_contain_exactly(&[3, 2, 1])
            .is_err());
    }

    #[test]
    fn slice_to_contain_exactly_in_any_order_pass() {
        let s: &[i32] = &[3, 1, 2];
        assert!(Expectation::new(s, "s")
            .to_contain_exactly_in_any_order(&[1, 2, 3])
            .is_ok());
    }

    #[test]
    fn slice_to_start_with_elements_pass() {
        let s: &[i32] = &[1, 2, 3];
        assert!(Expectation::new(s, "s")
            .to_start_with_elements(&[1, 2])
            .is_ok());
    }

    #[test]
    fn slice_to_end_with_elements_pass() {
        let s: &[i32] = &[1, 2, 3];
        assert!(Expectation::new(s, "s")
            .to_end_with_elements(&[2, 3])
            .is_ok());
    }

    #[test]
    fn slice_to_be_sorted_pass() {
        let s: &[i32] = &[1, 2, 3];
        assert!(Expectation::new(s, "s").to_be_sorted().is_ok());
    }

    #[test]
    fn slice_to_be_sorted_fail() {
        let s: &[i32] = &[3, 1, 2];
        assert!(Expectation::new(s, "s").to_be_sorted().is_err());
    }

    // --- to_be_sorted_by_key ---

    #[test]
    fn vec_to_be_sorted_by_key_pass() {
        assert!(Expectation::new(vec!["a", "bb", "ccc"], "v")
            .to_be_sorted_by_key(|s| s.len(), "by length")
            .is_ok());
    }

    #[test]
    fn vec_to_be_sorted_by_key_fail() {
        assert!(Expectation::new(vec!["ccc", "a", "bb"], "v")
            .to_be_sorted_by_key(|s| s.len(), "by length")
            .is_err());
    }

    #[test]
    fn vec_to_be_sorted_by_key_empty() {
        let v: Vec<&str> = vec![];
        assert!(Expectation::new(v, "v")
            .to_be_sorted_by_key(|s| s.len(), "by length")
            .is_ok());
    }

    #[test]
    fn vec_to_be_sorted_by_key_negated() {
        assert!(Expectation::new(vec!["ccc", "a"], "v")
            .negate()
            .to_be_sorted_by_key(|s| s.len(), "by length")
            .is_ok());
    }

    #[test]
    fn slice_to_be_sorted_by_key_pass() {
        let s: &[&str] = &["a", "bb", "ccc"];
        assert!(Expectation::new(s, "s")
            .to_be_sorted_by_key(|s| s.len(), "by length")
            .is_ok());
    }

    #[test]
    fn slice_to_be_sorted_by_key_fail() {
        let s: &[&str] = &["ccc", "a"];
        assert!(Expectation::new(s, "s")
            .to_be_sorted_by_key(|s| s.len(), "by length")
            .is_err());
    }
}
