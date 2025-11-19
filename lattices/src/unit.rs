use crate::{Atomize, DeepReveal, IsBot, IsTop, LatticeFrom, LatticeOrd, Merge};

impl DeepReveal for () {
    type Revealed = ();

    fn deep_reveal(self) -> Self::Revealed {
        self
    }
}

impl Merge<Self> for () {
    fn merge(&mut self, _other: Self) -> bool {
        false
    }
}

impl LatticeOrd for () {}

impl LatticeFrom<Self> for () {
    fn lattice_from(other: Self) -> Self {
        other
    }
}

impl IsBot for () {
    fn is_bot(&self) -> bool {
        true
    }
}

impl IsTop for () {
    fn is_top(&self) -> bool {
        true
    }
}

impl Atomize for () {
    type Atom = Self;

    type AtomIter = std::iter::Empty<Self>;

    fn atomize(self) -> Self::AtomIter {
        std::iter::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{check_all, check_lattice_ord};

    #[test]
    fn test_unit_lattice_properties() {
        // Unit type should satisfy all lattice properties
        check_all(&[()]);
    }

    #[test]
    fn test_unit_merge() {
        let mut a = ();
        let b = ();
        assert_eq!(a.merge(b), false); // Merging with self returns false (no change)
    }

    #[test]
    fn test_unit_is_bot_and_top() {
        let unit = ();
        assert!(unit.is_bot());
        assert!(unit.is_top());
    }

    #[test]
    fn test_unit_deep_reveal() {
        let unit = ();
        assert_eq!(unit.deep_reveal(), ());
    }

    #[test]
    fn test_unit_atomize() {
        let unit = ();
        let atoms: Vec<()> = unit.atomize().collect();
        assert_eq!(atoms.len(), 0); // Unit atomizes to empty iterator
    }

    #[test]
    fn test_unit_lattice_from() {
        let unit = ();
        let converted = <() as LatticeFrom<()>>::lattice_from(unit);
        assert_eq!(converted, ());
    }

    #[test]
    fn test_unit_lattice_ord() {
        let items = vec![(), (), ()];
        check_lattice_ord(&items);
    }
}
