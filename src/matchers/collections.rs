//! Collection matchers for slices, vectors, and other iterables.

use core::fmt::Debug;

use crate::error::MatchError;
use crate::expectation::Expectation;

/// Trait abstracting over `Vec<T>` and `&[T]` for collection matchers.
///
/// This is an implementation detail — do not implement or rely on it.
#[doc(hidden)]
pub trait CollectionLike: Debug {
    /// The element type.
    type Item: Debug;
    /// View the collection as a slice.
    fn __behave_as_slice(&self) -> &[Self::Item];
}

impl<T: Debug> CollectionLike for Vec<T> {
    type Item = T;
    fn __behave_as_slice(&self) -> &[T] {
        self
    }
}

impl<T: Debug> CollectionLike for &[T] {
    type Item = T;
    fn __behave_as_slice(&self) -> &[T] {
        self
    }
}

impl<C: CollectionLike> Expectation<C> {
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
    /// assert!(Expectation::new(v, "v").to_be_empty().is_ok());
    ///
    /// let s: &[i32] = &[];
    /// assert!(Expectation::new(s, "s").to_be_empty().is_ok());
    /// ```
    pub fn to_be_empty(&self) -> Result<(), MatchError> {
        self.check(self.value().__behave_as_slice().is_empty(), "to be empty")
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
    /// assert!(Expectation::new(vec![1], "v").to_not_be_empty().is_ok());
    ///
    /// let s: &[i32] = &[1];
    /// assert!(Expectation::new(s, "s").to_not_be_empty().is_ok());
    /// ```
    pub fn to_not_be_empty(&self) -> Result<(), MatchError> {
        self.check(
            !self.value().__behave_as_slice().is_empty(),
            "to not be empty",
        )
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
    /// assert!(Expectation::new(vec![1, 2, 3], "v").to_have_length(3).is_ok());
    ///
    /// let s: &[i32] = &[1, 2, 3];
    /// assert!(Expectation::new(s, "s").to_have_length(3).is_ok());
    /// ```
    pub fn to_have_length(&self, expected: usize) -> Result<(), MatchError> {
        let actual_len = self.value().__behave_as_slice().len();
        self.check(actual_len == expected, format!("to have length {expected}"))
    }

    /// Asserts every element in the collection satisfies the predicate.
    ///
    /// Returns `Ok` for an empty collection (vacuous truth).
    /// The `desc` argument appears in failure messages.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any element fails the predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// assert!(Expectation::new(vec![2, 4, 6], "v")
    ///     .to_all_satisfy(|x| x % 2 == 0, "to be even")
    ///     .is_ok());
    /// ```
    pub fn to_all_satisfy(
        &self,
        f: impl Fn(&C::Item) -> bool,
        desc: &str,
    ) -> Result<(), MatchError> {
        let is_match = self.value().__behave_as_slice().iter().all(&f);
        self.check(is_match, format!("all elements {desc}"))
    }

    /// Asserts at least one element satisfies the predicate.
    ///
    /// Returns `Err` for an empty collection.
    /// The `desc` argument appears in failure messages.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if no element satisfies the predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// assert!(Expectation::new(vec![1, 2, 3], "v")
    ///     .to_any_satisfy(|x| x % 2 == 0, "to be even")
    ///     .is_ok());
    /// ```
    pub fn to_any_satisfy(
        &self,
        f: impl Fn(&C::Item) -> bool,
        desc: &str,
    ) -> Result<(), MatchError> {
        let is_match = self.value().__behave_as_slice().iter().any(&f);
        self.check(is_match, format!("any element {desc}"))
    }

    /// Asserts no element satisfies the predicate.
    ///
    /// Returns `Ok` for an empty collection (vacuous truth).
    /// The `desc` argument appears in failure messages.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if any element satisfies the predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// assert!(Expectation::new(vec![1, 3, 5], "v")
    ///     .to_none_satisfy(|x| x % 2 == 0, "to be even")
    ///     .is_ok());
    /// ```
    pub fn to_none_satisfy(
        &self,
        f: impl Fn(&C::Item) -> bool,
        desc: &str,
    ) -> Result<(), MatchError> {
        let is_match = !self.value().__behave_as_slice().iter().any(&f);
        self.check(is_match, format!("no element {desc}"))
    }
}

impl<C: CollectionLike> Expectation<C>
where
    C::Item: PartialEq,
{
    /// Asserts the collection contains the given element.
    ///
    /// For checking that *all* of several elements are present, use
    /// [`to_contain_all_of`](Self::to_contain_all_of) instead.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the element is not found.
    ///
    /// ```text
    /// expect!(items)
    ///   actual: [1, 2, 3]
    /// expected: to contain 9
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// assert!(Expectation::new(vec![1, 2, 3], "v").to_contain(2).is_ok());
    ///
    /// let s: &[i32] = &[1, 2, 3];
    /// assert!(Expectation::new(s, "s").to_contain(2).is_ok());
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn to_contain(&self, element: C::Item) -> Result<(), MatchError> {
        let is_match = self.value().__behave_as_slice().contains(&element);
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
    /// assert!(Expectation::new(vec![1, 2, 3], "v")
    ///     .to_contain_all_of(&[1, 3])
    ///     .is_ok());
    /// ```
    pub fn to_contain_all_of(&self, elements: &[C::Item]) -> Result<(), MatchError> {
        let is_match = elements
            .iter()
            .all(|e| self.value().__behave_as_slice().contains(e));
        self.check(is_match, format!("to contain all of {elements:?}"))
    }

    /// Asserts the collection contains at least one of the given elements.
    ///
    /// Returns `Err` when `elements` is empty (no element could match).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if none of the elements are found.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// assert!(Expectation::new(vec![1, 2, 3], "v")
    ///     .to_contain_any_of(&[9, 2])
    ///     .is_ok());
    /// ```
    pub fn to_contain_any_of(&self, elements: &[C::Item]) -> Result<(), MatchError> {
        let is_match = elements
            .iter()
            .any(|e| self.value().__behave_as_slice().contains(e));
        self.check(is_match, format!("to contain any of {elements:?}"))
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

    // --- to_all_satisfy ---

    #[test]
    fn vec_to_all_satisfy_pass() {
        assert!(Expectation::new(vec![2, 4, 6], "v")
            .to_all_satisfy(|x| x % 2 == 0, "to be even")
            .is_ok());
    }

    #[test]
    fn vec_to_all_satisfy_fail() {
        assert!(Expectation::new(vec![2, 3, 6], "v")
            .to_all_satisfy(|x| x % 2 == 0, "to be even")
            .is_err());
    }

    #[test]
    fn vec_to_all_satisfy_empty() {
        let v: Vec<i32> = vec![];
        assert!(Expectation::new(v, "v")
            .to_all_satisfy(|x| x % 2 == 0, "to be even")
            .is_ok());
    }

    #[test]
    fn vec_to_all_satisfy_negated() {
        assert!(Expectation::new(vec![2, 3, 6], "v")
            .negate()
            .to_all_satisfy(|x| x % 2 == 0, "to be even")
            .is_ok());
    }

    // --- to_any_satisfy ---

    #[test]
    fn vec_to_any_satisfy_pass() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_any_satisfy(|x| x % 2 == 0, "to be even")
            .is_ok());
    }

    #[test]
    fn vec_to_any_satisfy_fail() {
        assert!(Expectation::new(vec![1, 3, 5], "v")
            .to_any_satisfy(|x| x % 2 == 0, "to be even")
            .is_err());
    }

    #[test]
    fn vec_to_any_satisfy_negated() {
        assert!(Expectation::new(vec![1, 3, 5], "v")
            .negate()
            .to_any_satisfy(|x| x % 2 == 0, "to be even")
            .is_ok());
    }

    // --- to_none_satisfy ---

    #[test]
    fn vec_to_none_satisfy_pass() {
        assert!(Expectation::new(vec![1, 3, 5], "v")
            .to_none_satisfy(|x| x % 2 == 0, "to be even")
            .is_ok());
    }

    #[test]
    fn vec_to_none_satisfy_fail() {
        assert!(Expectation::new(vec![1, 2, 5], "v")
            .to_none_satisfy(|x| x % 2 == 0, "to be even")
            .is_err());
    }

    #[test]
    fn vec_to_none_satisfy_negated() {
        assert!(Expectation::new(vec![1, 2, 5], "v")
            .negate()
            .to_none_satisfy(|x| x % 2 == 0, "to be even")
            .is_ok());
    }

    // --- to_contain_any_of ---

    #[test]
    fn vec_to_contain_any_of_pass() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_contain_any_of(&[9, 2])
            .is_ok());
    }

    #[test]
    fn vec_to_contain_any_of_fail() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_contain_any_of(&[8, 9])
            .is_err());
    }

    #[test]
    fn vec_to_contain_any_of_empty() {
        assert!(Expectation::new(vec![1, 2], "v")
            .to_contain_any_of(&[])
            .is_err());
    }

    #[test]
    fn vec_to_contain_any_of_negated() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .negate()
            .to_contain_any_of(&[8, 9])
            .is_ok());
    }

    // --- slice predicates ---

    #[test]
    fn slice_to_all_satisfy_pass() {
        let s: &[i32] = &[2, 4, 6];
        assert!(Expectation::new(s, "s")
            .to_all_satisfy(|x| x % 2 == 0, "to be even")
            .is_ok());
    }

    #[test]
    fn slice_to_all_satisfy_fail() {
        let s: &[i32] = &[2, 3, 6];
        assert!(Expectation::new(s, "s")
            .to_all_satisfy(|x| x % 2 == 0, "to be even")
            .is_err());
    }

    #[test]
    fn slice_to_any_satisfy_pass() {
        let s: &[i32] = &[1, 2, 3];
        assert!(Expectation::new(s, "s")
            .to_any_satisfy(|x| x % 2 == 0, "to be even")
            .is_ok());
    }

    #[test]
    fn slice_to_none_satisfy_pass() {
        let s: &[i32] = &[1, 3, 5];
        assert!(Expectation::new(s, "s")
            .to_none_satisfy(|x| x % 2 == 0, "to be even")
            .is_ok());
    }

    #[test]
    fn slice_to_contain_any_of_pass() {
        let s: &[i32] = &[1, 2, 3];
        assert!(Expectation::new(s, "s").to_contain_any_of(&[9, 2]).is_ok());
    }

    #[test]
    fn slice_to_contain_any_of_fail() {
        let s: &[i32] = &[1, 2, 3];
        assert!(Expectation::new(s, "s").to_contain_any_of(&[8, 9]).is_err());
    }
}
