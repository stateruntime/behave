//! Collection matchers for slices, vectors, and other iterables.

use core::fmt::Debug;

use crate::error::MatchError;
use crate::expectation::Expectation;

impl<T: Debug> Expectation<Vec<T>> {
    /// Asserts the collection is empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the collection is not empty (or empty when negated).
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let v: Vec<i32> = vec![];
    /// let result = Expectation::new(v, "v").to_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_empty(&self) -> Result<(), MatchError> {
        self.check(self.value().is_empty(), "empty collection")
    }

    /// Asserts the collection is not empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the collection is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(vec![1], "v").to_not_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_not_be_empty(&self) -> Result<(), MatchError> {
        self.check(!self.value().is_empty(), "non-empty collection")
    }

    /// Asserts the collection has exactly the given length.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the length does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(vec![1, 2, 3], "v").to_have_length(3);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_length(&self, expected: usize) -> Result<(), MatchError> {
        let actual_len = self.value().len();
        self.check(actual_len == expected, format!("length {expected}"))
    }
}

impl<T: Debug + PartialEq> Expectation<Vec<T>> {
    /// Asserts the collection contains the given element.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the element is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(vec![1, 2, 3], "v").to_contain(2);
    /// assert!(result.is_ok());
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn to_contain(&self, element: T) -> Result<(), MatchError> {
        let is_match = self.value().contains(&element);
        self.check(is_match, format!("to contain {element:?}"))
    }

    /// Asserts the collection contains all of the given elements.
    ///
    /// Returns `Ok` when `elements` is empty (vacuous truth).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any element is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(vec![1, 2, 3], "v")
    ///     .to_contain_all_of(&[1, 3]);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_all_of(&self, elements: &[T]) -> Result<(), MatchError> {
        let is_match = elements.iter().all(|e| self.value().contains(e));
        self.check(is_match, format!("to contain all of {elements:?}"))
    }
}

impl<T: Debug> Expectation<&[T]> {
    /// Asserts the slice is empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the slice is not empty (or empty when negated).
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let s: &[i32] = &[];
    /// let result = Expectation::new(s, "s").to_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_empty(&self) -> Result<(), MatchError> {
        self.check(self.value().is_empty(), "empty collection")
    }

    /// Asserts the slice is not empty.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the slice is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let s: &[i32] = &[1];
    /// let result = Expectation::new(s, "s").to_not_be_empty();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_not_be_empty(&self) -> Result<(), MatchError> {
        self.check(!self.value().is_empty(), "non-empty collection")
    }

    /// Asserts the slice has exactly the given length.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the length does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let s: &[i32] = &[1, 2, 3];
    /// let result = Expectation::new(s, "s").to_have_length(3);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_length(&self, expected: usize) -> Result<(), MatchError> {
        let actual_len = self.value().len();
        self.check(actual_len == expected, format!("length {expected}"))
    }
}

impl<T: Debug + PartialEq> Expectation<&[T]> {
    /// Asserts the slice contains the given element.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the element is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let s: &[i32] = &[1, 2, 3];
    /// let result = Expectation::new(s, "s").to_contain(2);
    /// assert!(result.is_ok());
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn to_contain(&self, element: T) -> Result<(), MatchError> {
        let is_match = self.value().contains(&element);
        self.check(is_match, format!("to contain {element:?}"))
    }

    /// Asserts the slice contains all of the given elements.
    ///
    /// Returns `Ok` when `elements` is empty (vacuous truth).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any element is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let s: &[i32] = &[1, 2, 3];
    /// let result = Expectation::new(s, "s").to_contain_all_of(&[1, 3]);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_all_of(&self, elements: &[T]) -> Result<(), MatchError> {
        let is_match = elements.iter().all(|e| self.value().contains(e));
        self.check(is_match, format!("to contain all of {elements:?}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::Expectation;

    #[test]
    fn to_be_empty_pass() {
        let v: Vec<i32> = vec![];
        assert!(Expectation::new(v, "v").to_be_empty().is_ok());
    }

    #[test]
    fn to_be_empty_fail() {
        assert!(Expectation::new(vec![1], "v").to_be_empty().is_err());
    }

    #[test]
    fn to_be_empty_negated() {
        assert!(Expectation::new(vec![1], "v")
            .negate()
            .to_be_empty()
            .is_ok());
    }

    #[test]
    fn to_not_be_empty_pass() {
        assert!(Expectation::new(vec![1], "v").to_not_be_empty().is_ok());
    }

    #[test]
    fn to_not_be_empty_fail() {
        let v: Vec<i32> = vec![];
        assert!(Expectation::new(v, "v").to_not_be_empty().is_err());
    }

    #[test]
    fn to_not_be_empty_negated() {
        let v: Vec<i32> = vec![];
        assert!(Expectation::new(v, "v").negate().to_not_be_empty().is_ok());
    }

    #[test]
    fn to_have_length_pass() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_have_length(3)
            .is_ok());
    }

    #[test]
    fn to_have_length_fail() {
        assert!(Expectation::new(vec![1, 2], "v").to_have_length(3).is_err());
    }

    #[test]
    fn to_have_length_negated() {
        assert!(Expectation::new(vec![1, 2], "v")
            .negate()
            .to_have_length(3)
            .is_ok());
    }

    #[test]
    fn to_have_length_zero() {
        let v: Vec<i32> = vec![];
        assert!(Expectation::new(v, "v").to_have_length(0).is_ok());
    }

    #[test]
    fn to_contain_pass() {
        assert!(Expectation::new(vec![1, 2, 3], "v").to_contain(2).is_ok());
    }

    #[test]
    fn to_contain_fail() {
        assert!(Expectation::new(vec![1, 2, 3], "v").to_contain(9).is_err());
    }

    #[test]
    fn to_contain_negated() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .negate()
            .to_contain(9)
            .is_ok());
    }

    #[test]
    fn to_contain_all_of_pass() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_contain_all_of(&[1, 3])
            .is_ok());
    }

    #[test]
    fn to_contain_all_of_partial_match() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_contain_all_of(&[1, 9])
            .is_err());
    }

    #[test]
    fn to_contain_all_of_none_match() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_contain_all_of(&[8, 9])
            .is_err());
    }

    #[test]
    fn to_contain_all_of_negated() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .negate()
            .to_contain_all_of(&[8, 9])
            .is_ok());
    }

    #[test]
    fn to_contain_all_of_empty_expected() {
        assert!(Expectation::new(vec![1, 2], "v")
            .to_contain_all_of(&[])
            .is_ok());
    }

    // --- Slice matchers ---

    #[test]
    fn slice_to_be_empty_pass() {
        let s: &[i32] = &[];
        assert!(Expectation::new(s, "s").to_be_empty().is_ok());
    }

    #[test]
    fn slice_to_be_empty_fail() {
        let s: &[i32] = &[1];
        assert!(Expectation::new(s, "s").to_be_empty().is_err());
    }

    #[test]
    fn slice_to_contain_pass() {
        let s: &[i32] = &[1, 2, 3];
        assert!(Expectation::new(s, "s").to_contain(2).is_ok());
    }

    #[test]
    fn slice_to_contain_fail() {
        let s: &[i32] = &[1, 2, 3];
        assert!(Expectation::new(s, "s").to_contain(9).is_err());
    }

    #[test]
    fn slice_to_have_length_pass() {
        let s: &[i32] = &[1, 2];
        assert!(Expectation::new(s, "s").to_have_length(2).is_ok());
    }

    #[test]
    fn slice_to_contain_all_of_pass() {
        let s: &[i32] = &[1, 2, 3];
        assert!(Expectation::new(s, "s").to_contain_all_of(&[1, 3]).is_ok());
    }
}
