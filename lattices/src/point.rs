use std::marker::PhantomData;

use crate::{DeepReveal, IsBot, IsTop, LatticeFrom, LatticeOrd, Merge};

/// A `Point` lattice, corresponding to a single instance of `T`.
///
/// Will runtime panic if a merge between inequal values is attempted.
///
/// The `Provenance` generic param is a token for the origin of this point. The parameter can be
/// used to differentiate between points with different provenances. This will prevent them from
/// being merged together, avoiding any posibility of panic.
///
/// Like [`Conflict<T>`](crate::Conflict) but will panic instead of going to a "conflict" top
/// state.
///
/// Can be thought of as a lattice with a domain of size one, corresponding to the specific value
/// inside.
///
/// This also can be used to wrap non lattice data into a lattice in a way that typechecks.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point<T, Provenance> {
    /// The value stored inside. This should not be mutated.
    pub val: T,
    _token: PhantomData<*mut Provenance>,
}
impl<T, Provenance> Point<T, Provenance> {
    /// Create a new `Point` lattice instance from a value.
    pub fn new(val: T) -> Self {
        Self {
            val,
            _token: PhantomData,
        }
    }

    /// Create a new `Point` lattice instance from a value using `Into`.
    pub fn new_from(val: impl Into<T>) -> Self {
        Self::new(val.into())
    }
}
impl<T, Provenance> DeepReveal for Point<T, Provenance> {
    type Revealed = T;

    fn deep_reveal(self) -> Self::Revealed {
        self.val
    }
}

impl<T, Provenance> Merge<Point<T, Provenance>> for Point<T, Provenance>
where
    T: PartialEq,
{
    fn merge(&mut self, other: Point<T, Provenance>) -> bool {
        if self.val != other.val {
            panic!("The `Point` lattice cannot merge inequal elements.")
        }
        false
    }
}

impl<T, Provenance> LatticeFrom<Point<T, Provenance>> for Point<T, Provenance> {
    fn lattice_from(other: Point<T, Provenance>) -> Self {
        other
    }
}

impl<T, Provenance> PartialOrd<Point<T, Provenance>> for Point<T, Provenance>
where
    T: PartialEq,
{
    fn partial_cmp(&self, other: &Point<T, Provenance>) -> Option<std::cmp::Ordering> {
        if self.val != other.val {
            panic!("The `Point` lattice does not have a partial order between inequal elements.");
        }
        Some(std::cmp::Ordering::Equal)
    }
}
impl<T, Provenance> LatticeOrd<Point<T, Provenance>> for Point<T, Provenance> where
    Self: PartialOrd<Point<T, Provenance>>
{
}

impl<T, Provenance> PartialEq<Point<T, Provenance>> for Point<T, Provenance>
where
    T: PartialEq,
{
    fn eq(&self, other: &Point<T, Provenance>) -> bool {
        self.val == other.val
    }
}

impl<T, Provenance> IsBot for Point<T, Provenance> {
    fn is_bot(&self) -> bool {
        true
    }
}

impl<T, Provenance> IsTop for Point<T, Provenance> {
    fn is_top(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::{
        check_all, check_lattice_ord, check_lattice_properties, check_partial_ord_properties,
    };

    #[test]
    fn consistency_equal() {
        check_all(&[Point::<_, ()>::new("hello world")])
    }

    #[test]
    fn consistency_inequal() {
        use std::collections::BTreeSet;

        let items: &[Point<_, ()>] = &[
            Point::new(BTreeSet::from_iter([])),
            Point::new(BTreeSet::from_iter([0])),
            Point::new(BTreeSet::from_iter([1])),
            Point::new(BTreeSet::from_iter([0, 1])),
        ];

        // Merged inequal elements panic, therefore `NaiveMerge` panics.
        assert!(std::panic::catch_unwind(|| check_lattice_ord(items)).is_err());
        // `Point` does not have a partial order.
        assert!(std::panic::catch_unwind(|| check_partial_ord_properties(items)).is_err());
        // `Point` is not actually a lattice.
        assert!(std::panic::catch_unwind(|| check_lattice_properties(items)).is_err());
    }

    #[test]
    fn point_merge_equal_no_change() {
        let mut a = Point::<_, ()>::new(42);
        let b = Point::<_, ()>::new(42);
        assert!(!a.merge(b));
        assert_eq!(a.val, 42);
    }

    #[test]
    #[should_panic(expected = "The `Point` lattice cannot merge inequal elements.")]
    fn point_merge_inequal_panics() {
        let mut a = Point::<_, ()>::new(1);
        let b = Point::<_, ()>::new(2);
        a.merge(b);
    }

    #[test]
    fn point_is_always_bot_and_top() {
        use crate::{IsBot, IsTop};
        let p = Point::<_, ()>::new("anything");
        assert!(p.is_bot());
        assert!(p.is_top());
    }

    #[test]
    fn point_eq_equal_values() {
        let a = Point::<_, ()>::new(42);
        let b = Point::<_, ()>::new(42);
        assert_eq!(a, b);
    }

    #[test]
    fn point_eq_inequal_values() {
        let a = Point::<_, ()>::new(1);
        let b = Point::<_, ()>::new(2);
        assert_ne!(a, b);
    }

    #[test]
    fn point_deep_reveal() {
        use crate::DeepReveal;
        let p = Point::<_, ()>::new("hello");
        assert_eq!(p.deep_reveal(), "hello");
    }

    #[test]
    fn point_new_from() {
        let p = Point::<String, ()>::new_from("hello");
        assert_eq!(p.val, "hello");
    }

    #[test]
    fn point_default() {
        let p = Point::<i32, ()>::default();
        assert_eq!(p.val, 0);
    }

    #[test]
    fn point_lattice_from() {
        let a = Point::<_, ()>::new(42);
        let b: Point<_, ()> = LatticeFrom::lattice_from(a);
        assert_eq!(b.val, 42);
    }

    #[test]
    #[should_panic(expected = "The `Point` lattice does not have a partial order")]
    fn point_partial_cmp_inequal_panics() {
        let a = Point::<_, ()>::new(1);
        let b = Point::<_, ()>::new(2);
        a.partial_cmp(&b);
    }

    #[test]
    fn point_partial_cmp_equal() {
        let a = Point::<_, ()>::new(42);
        let b = Point::<_, ()>::new(42);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Equal));
    }
}
