use crate::{DeepReveal, Lattice, LatticeBimorphism};

/// Pair compound lattice.
///
/// `LatA` and `LatB` specify the nested lattice types.
///
/// When merging, both sub-lattices are always merged.
#[derive(Copy, Clone, Debug, Default, Eq, Lattice)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pair<LatA, LatB> {
    /// The "left" Lattice of the Pair lattice.
    pub a: LatA,

    /// The "right" Lattice of the Pair lattice.
    pub b: LatB,
}

impl<LatA, LatB> Pair<LatA, LatB> {
    /// Create a `Pair` from the given values.
    pub fn new(a: LatA, b: LatB) -> Self {
        Self { a, b }
    }

    /// Create a `Pair` from the given values, using `Into`.
    pub fn new_from(a: impl Into<LatA>, b: impl Into<LatB>) -> Self {
        Self::new(a.into(), b.into())
    }

    /// Reveal the inner value as a shared reference.
    pub fn as_reveal_ref(&self) -> (&LatA, &LatB) {
        (&self.a, &self.b)
    }

    /// Reveal the inner value as an exclusive reference.
    pub fn as_reveal_mut(&mut self) -> (&mut LatA, &mut LatB) {
        (&mut self.a, &mut self.b)
    }

    /// Gets the inner by value, consuming self.
    pub fn into_reveal(self) -> (LatA, LatB) {
        (self.a, self.b)
    }
}

impl<LatA, LatB> DeepReveal for Pair<LatA, LatB>
where
    LatA: DeepReveal,
    LatB: DeepReveal,
{
    type Revealed = (LatA::Revealed, LatB::Revealed);

    fn deep_reveal(self) -> Self::Revealed {
        (self.a.deep_reveal(), self.b.deep_reveal())
    }
}

/// Bimorphism which pairs up the two input lattices.
#[derive(Default)]
pub struct PairBimorphism;
impl<LatA, LatB> LatticeBimorphism<LatA, LatB> for PairBimorphism {
    type Output = Pair<LatA, LatB>;

    fn call(&mut self, lat_a: LatA, lat_b: LatB) -> Self::Output {
        Pair::new(lat_a, lat_b)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::set_union::{SetUnionBTreeSet, SetUnionHashSet, SetUnionSingletonSet};
    use crate::test::{check_all, check_lattice_bimorphism};
    use crate::{Max, Merge, Min, WithTop};

    #[test]
    fn consistency() {
        let mut test_vec = Vec::new();

        for a in [vec![], vec![0], vec![1], vec![0, 1]] {
            for b in [vec![], vec![0], vec![1], vec![0, 1]] {
                test_vec.push(Pair::new(
                    SetUnionHashSet::new_from(HashSet::from_iter(a.clone())),
                    SetUnionHashSet::new_from(HashSet::from_iter(b.clone())),
                ));
            }
        }

        check_all(&test_vec);
    }

    #[test]
    fn consistency_withtop() {
        let mut test_vec = vec![];

        let sub_items = &[
            Some(&[] as &[usize]),
            Some(&[0]),
            Some(&[1]),
            Some(&[0, 1]),
            None,
        ];

        for a in sub_items {
            for b in sub_items {
                test_vec.push(Pair::new(
                    WithTop::new(
                        a.map(|x| SetUnionHashSet::new_from(HashSet::from_iter(x.iter().cloned()))),
                    ),
                    WithTop::new(
                        b.map(|x| SetUnionHashSet::new_from(HashSet::from_iter(x.iter().cloned()))),
                    ),
                ));
            }
        }

        check_all(&test_vec);
    }

    #[test]
    fn test_merge_direction() {
        let src = Pair::new(
            SetUnionSingletonSet::new_from(5),
            SetUnionSingletonSet::new_from("hello"),
        );
        let mut dst = Pair::new(
            SetUnionHashSet::new_from([1, 2]),
            SetUnionBTreeSet::new_from(["world"]),
        );
        dst.merge(src);
    }

    #[test]
    fn test_pair_bimorphism() {
        let items_a = &[
            SetUnionHashSet::new_from([]),
            SetUnionHashSet::new_from([0]),
            SetUnionHashSet::new_from([1]),
            SetUnionHashSet::new_from([0, 1]),
        ];
        let items_b = &[
            SetUnionBTreeSet::new("hello".chars().collect()),
            SetUnionBTreeSet::new("world".chars().collect()),
        ];

        check_lattice_bimorphism(PairBimorphism, items_a, items_a);
        check_lattice_bimorphism(PairBimorphism, items_a, items_b);
        check_lattice_bimorphism(PairBimorphism, items_b, items_a);
        check_lattice_bimorphism(PairBimorphism, items_b, items_b);
    }

    #[test]
    fn pair_new_and_reveal() {
        let p = Pair::new(Max::new(10), Min::new(20));
        let (a, b) = p.as_reveal_ref();
        assert_eq!(*a.as_reveal_ref(), 10);
        assert_eq!(*b.as_reveal_ref(), 20);
    }

    #[test]
    fn pair_into_reveal() {
        let p = Pair::new(Max::new(10), Min::new(20));
        let (a, b) = p.into_reveal();
        assert_eq!(a.into_reveal(), 10);
        assert_eq!(b.into_reveal(), 20);
    }

    #[test]
    fn pair_as_reveal_mut() {
        let mut p = Pair::new(
            SetUnionHashSet::new_from([1]),
            SetUnionHashSet::new_from([2]),
        );
        let (a, b) = p.as_reveal_mut();
        a.as_reveal_mut().insert(10);
        b.as_reveal_mut().insert(20);
        let (a, b) = p.into_reveal();
        assert!(a.as_reveal_ref().contains(&10));
        assert!(b.as_reveal_ref().contains(&20));
    }

    #[test]
    fn pair_merge_both_sides_change() {
        let mut p1 = Pair::new(
            SetUnionHashSet::new_from([1]),
            SetUnionHashSet::new_from([10]),
        );
        let p2 = Pair::new(
            SetUnionHashSet::new_from([2]),
            SetUnionHashSet::new_from([20]),
        );
        assert!(p1.merge(p2));
        let (a, b) = p1.into_reveal();
        assert!(a.as_reveal_ref().contains(&1));
        assert!(a.as_reveal_ref().contains(&2));
        assert!(b.as_reveal_ref().contains(&10));
        assert!(b.as_reveal_ref().contains(&20));
    }

    #[test]
    fn pair_merge_no_change() {
        let mut p1 = Pair::new(
            SetUnionHashSet::new_from([1, 2]),
            SetUnionHashSet::new_from([10, 20]),
        );
        let p2 = Pair::new(
            SetUnionHashSet::new_from([1]),
            SetUnionHashSet::new_from([10]),
        );
        assert!(!p1.merge(p2));
    }

    #[test]
    fn pair_deep_reveal() {
        use crate::DeepReveal;
        let p = Pair::new(
            SetUnionHashSet::new_from([1, 2]),
            SetUnionHashSet::new_from([3, 4]),
        );
        let (a, b) = p.deep_reveal();
        assert_eq!(a.len(), 2);
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn pair_default_is_bot() {
        use crate::IsBot;
        let p = Pair::<SetUnionHashSet<usize>, SetUnionHashSet<usize>>::default();
        assert!(p.is_bot());
    }

    #[test]
    fn pair_new_from() {
        use crate::IsBot;
        let p = Pair::<SetUnionHashSet<usize>, SetUnionHashSet<usize>>::new_from(
            HashSet::from([1]),
            HashSet::from([2]),
        );
        assert!(!p.is_bot());
    }
}
