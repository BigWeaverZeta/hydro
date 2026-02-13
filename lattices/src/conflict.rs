use std::cmp::Ordering::{self, *};

use crate::{DeepReveal, IsBot, IsTop, LatticeFrom, LatticeOrd, Merge};

/// A `Conflict` lattice, stores a single instance of `T` and goes to a "conflict" state (`None`)
/// if inequal `T` instances are merged together.
///
/// Like [`Point<T>`](crate::Point), but will go to "conflict" (top/`None`) instead of panicking.
///
/// Can be thought of as a lattice with a domain of size one, corresponding to the specific value
/// inside.
///
/// This can be used to wrap non-lattice (scalar) data into a lattice type.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Conflict<T>(Option<T>);
impl<T> Conflict<T> {
    /// Create a new `Conflict` lattice instance from a value.
    pub fn new(val: Option<T>) -> Self {
        Self(val)
    }

    /// Create a new `Conflict` lattice instance from a value using `Into`.
    pub fn new_from(val: impl Into<Option<T>>) -> Self {
        Self::new(val.into())
    }

    /// Reveal the inner value as a shared reference.
    pub fn as_reveal_ref(&self) -> Option<&T> {
        self.0.as_ref()
    }

    /// Reveal the inner value as an exclusive reference.
    pub fn as_reveal_mut(&mut self) -> Option<&mut T> {
        self.0.as_mut()
    }

    /// Gets the inner by value, consuming self.
    pub fn into_reveal(self) -> Option<T> {
        self.0
    }
}

impl<T> DeepReveal for Conflict<T> {
    type Revealed = Option<T>;

    fn deep_reveal(self) -> Self::Revealed {
        self.0
    }
}

impl<T, O> Merge<Conflict<O>> for Conflict<T>
where
    T: PartialEq<O>,
{
    fn merge(&mut self, other: Conflict<O>) -> bool {
        if let Some(val_self) = &self.0
            && other.0.is_none_or(|val_other| val_self != &val_other)
        {
            self.0 = None;
            return true;
        }
        false
    }
}

impl<T> LatticeFrom<Conflict<T>> for Conflict<T> {
    fn lattice_from(other: Conflict<T>) -> Self {
        other
    }
}

impl<T, O> PartialOrd<Conflict<O>> for Conflict<T>
where
    T: PartialEq<O>,
{
    fn partial_cmp(&self, other: &Conflict<O>) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (None, None) => Some(Equal),
            (None, Some(_)) => Some(Greater),
            (Some(_), None) => Some(Less),
            (Some(val_self), Some(val_other)) => (val_self == val_other).then_some(Equal),
        }
    }
}
impl<T, O> LatticeOrd<Conflict<O>> for Conflict<T> where Self: PartialOrd<Conflict<O>> {}

impl<T, O> PartialEq<Conflict<O>> for Conflict<T>
where
    T: PartialEq<O>,
{
    fn eq(&self, other: &Conflict<O>) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(val_self), Some(val_other)) => val_self == val_other,
            _ => false,
        }
    }
}

impl<T> IsBot for Conflict<T> {
    fn is_bot(&self) -> bool {
        false
    }
}

impl<T> IsTop for Conflict<T> {
    fn is_top(&self) -> bool {
        self.0.is_none()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::WithBot;
    use crate::test::{
        check_all, check_lattice_is_bot, check_lattice_is_top, check_lattice_ord,
        check_lattice_properties, check_partial_ord_properties,
    };

    #[test]
    fn consistency() {
        let items = &[
            Conflict::new_from("foo"),
            Conflict::new_from("bar"),
            Conflict::new(None),
        ];
        check_lattice_ord(items);
        check_partial_ord_properties(items);
        check_lattice_properties(items);
        check_lattice_is_bot(items);
        check_lattice_is_top(items);
    }

    #[test]
    fn consistency_withbot() {
        let items = &[
            WithBot::new_from(Conflict::new_from("foo")),
            WithBot::new_from(Conflict::new_from("bar")),
            WithBot::new_from(Conflict::new(None)),
            WithBot::new(None),
        ];
        check_all(items);
    }

    #[test]
    fn merge_equal_values_no_change() {
        let mut a = Conflict::new_from(42);
        let b = Conflict::new_from(42);
        assert!(!a.merge(b));
        assert_eq!(a.into_reveal(), Some(42));
    }

    #[test]
    fn merge_inequal_values_becomes_conflict() {
        let mut a = Conflict::new_from(1);
        let b = Conflict::new_from(2);
        assert!(a.merge(b));
        assert_eq!(a.into_reveal(), None);
    }

    #[test]
    fn merge_value_with_conflict_becomes_conflict() {
        let mut a = Conflict::new_from(1);
        let b = Conflict::new(None);
        assert!(a.merge(b));
        assert_eq!(a.into_reveal(), None);
    }

    #[test]
    fn merge_conflict_with_value_no_change() {
        let mut a = Conflict::<i32>::new(None);
        let b = Conflict::new_from(1);
        assert!(!a.merge(b));
        assert_eq!(a.into_reveal(), None);
    }

    #[test]
    fn merge_conflict_with_conflict_no_change() {
        let mut a = Conflict::<i32>::new(None);
        let b = Conflict::<i32>::new(None);
        assert!(!a.merge(b));
        assert_eq!(a.into_reveal(), None);
    }

    #[test]
    fn is_top_is_conflict() {
        assert!(Conflict::<i32>::new(None).is_top());
        assert!(!Conflict::new_from(42).is_top());
    }

    #[test]
    fn is_bot_always_false() {
        assert!(!Conflict::<i32>::new(None).is_bot());
        assert!(!Conflict::new_from(42).is_bot());
    }

    #[test]
    fn partial_ord_equal_values() {
        assert_eq!(
            Conflict::new_from(10).partial_cmp(&Conflict::new_from(10)),
            Some(Equal)
        );
    }

    #[test]
    fn partial_ord_incomparable_values() {
        assert_eq!(
            Conflict::new_from(10).partial_cmp(&Conflict::new_from(20)),
            None
        );
    }

    #[test]
    fn partial_ord_value_vs_conflict() {
        assert_eq!(
            Conflict::new_from(10).partial_cmp(&Conflict::new(None)),
            Some(Less)
        );
        assert_eq!(
            Conflict::<i32>::new(None).partial_cmp(&Conflict::new_from(10)),
            Some(Greater)
        );
    }

    #[test]
    fn deep_reveal() {
        let c = Conflict::new_from("hello");
        assert_eq!(c.deep_reveal(), Some("hello"));

        let c = Conflict::<&str>::new(None);
        assert_eq!(c.deep_reveal(), None);
    }

    #[test]
    fn as_reveal_ref_and_mut() {
        let c = Conflict::new_from(42);
        assert_eq!(c.as_reveal_ref(), Some(&42));

        let mut c = Conflict::new_from(42);
        *c.as_reveal_mut().unwrap() = 100;
        assert_eq!(c.into_reveal(), Some(100));
    }

    #[test]
    fn lattice_from_identity() {
        let c = Conflict::new_from(42);
        let c2: Conflict<i32> = LatticeFrom::lattice_from(c);
        assert_eq!(c2.into_reveal(), Some(42));
    }
}
