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

    #[test]
    fn test_unit_deep_reveal() {
        let unit = ();
        let revealed = unit.deep_reveal();
        assert_eq!(revealed, ());
    }

    #[test]
    fn test_unit_merge_no_change() {
        let mut unit1 = ();
        let unit2 = ();
        let changed = unit1.merge(unit2);
        assert!(!changed);
        assert_eq!(unit1, ());
    }

    #[test]
    fn test_unit_is_bot() {
        let unit = ();
        assert!(unit.is_bot());
    }

    #[test]
    fn test_unit_is_top() {
        let unit = ();
        assert!(unit.is_top());
    }

    #[test]
    fn test_unit_lattice_from() {
        let unit = <() as LatticeFrom<()>>::lattice_from(());
        assert_eq!(unit, ());
    }

    #[test]
    fn test_unit_atomize_empty() {
        let unit = ();
        let mut atoms = unit.atomize();
        assert_eq!(atoms.next(), None);
    }
}
