//! Comprehensive tests for lattice properties and operations.

use lattices::*;
use lattices::ord::*;

#[cfg(test)]
mod max_lattice_tests {
    use super::*;

    /// Test Max lattice basic operations
    #[test]
    fn test_max_basic() {
        let mut a = Max::new(5);
        let b = Max::new(10);
        let c = Max::new(3);
        
        a.merge(b.clone());
        assert_eq!(a.into_reveal(), 10);
        
        let mut d = Max::new(8);
        d.merge(c);
        assert_eq!(d.into_reveal(), 8);
    }

    /// Test Max lattice identity (bottom)
    #[test]
    fn test_max_identity() {
        let mut a = Max::new(5);
        let bot = Max::new(i32::MIN);
        let a_before = a.clone();
        
        a.merge(bot);
        assert_eq!(a, a_before);
    }

    /// Test Max lattice commutativity
    #[test]
    fn test_max_commutativity() {
        let a = Max::new(5);
        let b = Max::new(10);
        
        let mut left = a.clone();
        left.merge(b.clone());
        
        let mut right = b.clone();
        right.merge(a.clone());
        
        assert_eq!(left, right);
    }

    /// Test Max lattice associativity
    #[test]
    fn test_max_associativity() {
        let a = Max::new(5);
        let b = Max::new(10);
        let c = Max::new(3);
        
        // (a ⊔ b) ⊔ c
        let mut left = a.clone();
        left.merge(b.clone());
        left.merge(c.clone());
        
        // a ⊔ (b ⊔ c)
        let mut bc = b.clone();
        bc.merge(c.clone());
        let mut right = a.clone();
        right.merge(bc);
        
        assert_eq!(left, right);
    }

    /// Test Max lattice idempotence
    #[test]
    fn test_max_idempotence() {
        let a = Max::new(5);
        let mut b = a.clone();
        b.merge(a.clone());
        
        assert_eq!(a, b);
    }

    /// Test Max with different types
    #[test]
    fn test_max_types() {
        let mut a = Max::new(5u32);
        a.merge(Max::new(10u32));
        assert_eq!(a.into_reveal(), 10u32);
        
        let mut b = Max::new(5.5f64);
        b.merge(Max::new(10.5f64));
        assert!((b.into_reveal() - 10.5).abs() < 0.001);
    }
}

#[cfg(test)]
mod min_lattice_tests {
    use super::*;

    /// Test Min lattice basic operations
    #[test]
    fn test_min_basic() {
        let mut a = Min::new(5);
        let b = Min::new(10);
        let c = Min::new(3);
        
        a.merge(c.clone());
        assert_eq!(a.into_reveal(), 3);
        
        let mut d = Min::new(8);
        d.merge(b);
        assert_eq!(d.into_reveal(), 8);
    }

    /// Test Min lattice identity (top)
    #[test]
    fn test_min_identity() {
        let mut a = Min::new(5);
        let top = Min::new(i32::MAX);
        let a_before = a.clone();
        
        a.merge(top);
        assert_eq!(a, a_before);
    }

    /// Test Min lattice properties
    #[test]
    fn test_min_properties() {
        let a = Min::new(5);
        let b = Min::new(10);
        let c = Min::new(3);
        
        // Commutativity
        let mut left = a.clone();
        left.merge(b.clone());
        let mut right = b.clone();
        right.merge(a.clone());
        assert_eq!(left, right);
        
        // Associativity
        let mut assoc_left = a.clone();
        assoc_left.merge(b.clone());
        assoc_left.merge(c.clone());
        
        let mut bc = b.clone();
        bc.merge(c.clone());
        let mut assoc_right = a.clone();
        assoc_right.merge(bc);
        assert_eq!(assoc_left, assoc_right);
        
        // Idempotence
        let mut idem = a.clone();
        idem.merge(a.clone());
        assert_eq!(idem, a);
    }
}

#[cfg(test)]
mod pair_lattice_tests {
    use super::*;

    /// Test Pair lattice composition
    #[test]
    fn test_pair_basic() {
        let mut p1 = Pair::new(Max::new(5), Min::new(10));
        let p2 = Pair::new(Max::new(8), Min::new(7));
        
        p1.merge(p2);
        
        // Max component: max(5, 8) = 8
        // Min component: min(10, 7) = 7
        assert_eq!(p1.as_reveal().0.as_reveal(), &8);
        assert_eq!(p1.as_reveal().1.as_reveal(), &7);
    }

    /// Test Pair lattice with different lattice types
    #[test]
    fn test_pair_heterogeneous() {
        let mut p1 = Pair::new(Max::new(5), Max::new(100));
        let p2 = Pair::new(Max::new(10), Max::new(50));
        
        p1.merge(p2);
        
        assert_eq!(p1.as_reveal().0.as_reveal(), &10);
        assert_eq!(p1.as_reveal().1.as_reveal(), &100);
    }

    /// Test Pair lattice properties
    #[test]
    fn test_pair_properties() {
        let a = Pair::new(Max::new(5), Min::new(10));
        let b = Pair::new(Max::new(8), Min::new(7));
        
        // Commutativity
        let mut left = a.clone();
        left.merge(b.clone());
        let mut right = b.clone();
        right.merge(a.clone());
        
        assert_eq!(left.as_reveal().0.as_reveal(), right.as_reveal().0.as_reveal());
        assert_eq!(left.as_reveal().1.as_reveal(), right.as_reveal().1.as_reveal());
    }
}

#[cfg(test)]
mod dom_pair_tests {
    use super::*;
    use cc_traits::{SimpleCollectionRef, Collection};

    /// Test DomPair last-writer-wins semantics
    #[test]
    fn test_dom_pair_lww() {
        let mut d1 = DomPair::new_from(vec![1, 2, 3], Max::new("value1"));
        let d2 = DomPair::new_from(vec![1, 2], Max::new("value2"));
        let d3 = DomPair::new_from(vec![1, 2, 3, 4], Max::new("value3"));
        
        // Merge with smaller set - should keep d1's value
        d1.merge(d2.clone());
        assert_eq!(d1.as_reveal().1.as_reveal(), &"value1");
        
        // Merge with larger set - should take d3's value
        d1.merge(d3.clone());
        assert_eq!(d1.as_reveal().1.as_reveal(), &"value3");
    }

    /// Test DomPair with equal keys
    #[test]
    fn test_dom_pair_equal_keys() {
        let mut d1 = DomPair::new_from(vec![1, 2, 3], Max::new(100));
        let d2 = DomPair::new_from(vec![1, 2, 3], Max::new(200));
        
        // With equal keys, values merge
        d1.merge(d2);
        assert_eq!(d1.as_reveal().1.as_reveal(), &200);
    }
}

#[cfg(test)]
mod set_union_tests {
    use super::*;
    use std::collections::HashSet;

    /// Test SetUnion basic operations
    #[test]
    fn test_set_union_basic() {
        let mut s1 = SetUnion::new(HashSet::from([1, 2, 3]));
        let s2 = SetUnion::new(HashSet::from([3, 4, 5]));
        
        s1.merge(s2);
        
        let result = s1.into_reveal();
        assert_eq!(result.len(), 5);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
        assert!(result.contains(&4));
        assert!(result.contains(&5));
    }

    /// Test SetUnion identity
    #[test]
    fn test_set_union_identity() {
        let mut s1 = SetUnion::new(HashSet::from([1, 2, 3]));
        let empty = SetUnion::new(HashSet::new());
        let s1_before = s1.clone();
        
        s1.merge(empty);
        assert_eq!(s1, s1_before);
    }

    /// Test SetUnion properties
    #[test]
    fn test_set_union_properties() {
        let a = SetUnion::new(HashSet::from([1, 2]));
        let b = SetUnion::new(HashSet::from([2, 3]));
        let c = SetUnion::new(HashSet::from([3, 4]));
        
        // Commutativity
        let mut left = a.clone();
        left.merge(b.clone());
        let mut right = b.clone();
        right.merge(a.clone());
        assert_eq!(left, right);
        
        // Associativity
        let mut assoc_left = a.clone();
        assoc_left.merge(b.clone());
        assoc_left.merge(c.clone());
        
        let mut bc = b.clone();
        bc.merge(c.clone());
        let mut assoc_right = a.clone();
        assoc_right.merge(bc);
        assert_eq!(assoc_left, assoc_right);
        
        // Idempotence
        let mut idem = a.clone();
        idem.merge(a.clone());
        assert_eq!(idem, a);
    }
}

#[cfg(test)]
mod map_union_tests {
    use super::*;
    use std::collections::HashMap;
    use lattices::map_union::MapUnionHashMap;

    /// Test MapUnion basic operations
    #[test]
    fn test_map_union_basic() {
        let mut m1 = MapUnionHashMap::new(HashMap::from([
            (1, Max::new(10)),
            (2, Max::new(20)),
        ]));
        
        let m2 = MapUnionHashMap::new(HashMap::from([
            (2, Max::new(25)),
            (3, Max::new(30)),
        ]));
        
        m1.merge(m2);
        
        let result = m1.into_reveal();
        assert_eq!(result.get(&1).map(|v| v.as_reveal()), Some(&10));
        assert_eq!(result.get(&2).map(|v| v.as_reveal()), Some(&25)); // max(20, 25)
        assert_eq!(result.get(&3).map(|v| v.as_reveal()), Some(&30));
    }

    /// Test MapUnion with disjoint keys
    #[test]
    fn test_map_union_disjoint() {
        let mut m1 = MapUnionHashMap::new(HashMap::from([
            (1, Max::new(10)),
            (2, Max::new(20)),
        ]));
        
        let m2 = MapUnionHashMap::new(HashMap::from([
            (3, Max::new(30)),
            (4, Max::new(40)),
        ]));
        
        m1.merge(m2);
        
        let result = m1.into_reveal();
        assert_eq!(result.len(), 4);
    }

    /// Test MapUnion properties
    #[test]
    fn test_map_union_properties() {
        let a = MapUnionHashMap::new(HashMap::from([(1, Max::new(10))]));
        let b = MapUnionHashMap::new(HashMap::from([(1, Max::new(20))]));
        
        // Commutativity
        let mut left = a.clone();
        left.merge(b.clone());
        let mut right = b.clone();
        right.merge(a.clone());
        
        assert_eq!(
            left.as_reveal().get(&1).map(|v| v.as_reveal()),
            right.as_reveal().get(&1).map(|v| v.as_reveal())
        );
        
        // Idempotence
        let mut idem = a.clone();
        idem.merge(a.clone());
        assert_eq!(idem, a);
    }
}

#[cfg(test)]
mod with_bot_tests {
    use super::*;

    /// Test WithBot adds bottom element
    #[test]
    fn test_with_bot_basic() {
        let bot: WithBot<Max<i32>> = WithBot::new_bot();
        let val = WithBot::new(Max::new(10));
        
        let mut m1 = bot.clone();
        m1.merge(val.clone());
        
        // bot ⊔ val = val
        assert_eq!(m1, val);
        
        let mut m2 = val.clone();
        m2.merge(bot.clone());
        
        // val ⊔ bot = val
        assert_eq!(m2, val);
    }

    /// Test WithBot is true bottom
    #[test]
    fn test_with_bot_identity() {
        let bot: WithBot<Max<i32>> = WithBot::new_bot();
        
        assert!(bot.is_bot());
        
        let val = WithBot::new(Max::new(10));
        assert!(!val.is_bot());
    }
}

#[cfg(test)]
mod with_top_tests {
    use super::*;

    /// Test WithTop adds top element
    #[test]
    fn test_with_top_basic() {
        let top: WithTop<Min<i32>> = WithTop::new_top();
        let val = WithTop::new(Min::new(10));
        
        let mut m1 = top.clone();
        m1.merge(val.clone());
        
        // top ⊔ val = top
        assert_eq!(m1, top);
        
        let mut m2 = val.clone();
        m2.merge(top.clone());
        
        // val ⊔ top = top
        assert_eq!(m2, top);
    }

    /// Test WithTop is true top
    #[test]
    fn test_with_top_identity() {
        let top: WithTop<Min<i32>> = WithTop::new_top();
        
        assert!(top.is_top());
        
        let val = WithTop::new(Min::new(10));
        assert!(!val.is_top());
    }
}

#[cfg(test)]
mod lattice_ord_tests {
    use super::*;

    /// Test partial ordering on Max lattice
    #[test]
    fn test_max_partial_ord() {
        let a = Max::new(5);
        let b = Max::new(10);
        let c = Max::new(5);
        
        assert!(a.less_equal(&b));
        assert!(!b.less_equal(&a));
        assert!(a.less_equal(&c));
        assert!(c.less_equal(&a));
    }

    /// Test partial ordering on SetUnion
    #[test]
    fn test_set_union_partial_ord() {
        use std::collections::HashSet;
        
        let a = SetUnion::new(HashSet::from([1, 2]));
        let b = SetUnion::new(HashSet::from([1, 2, 3]));
        let c = SetUnion::new(HashSet::from([2, 3]));
        
        // a ⊑ b (a is subset of b)
        assert!(a.less_equal(&b));
        
        // b ⋢ a (b is not subset of a)
        assert!(!b.less_equal(&a));
        
        // a and c are incomparable
        assert!(!a.less_equal(&c));
        assert!(!c.less_equal(&a));
    }
}

#[cfg(test)]
mod lattice_from_tests {
    use super::*;

    /// Test LatticeFrom trait
    #[test]
    fn test_lattice_from() {
        let max_val: Max<i32> = Lattice::lattice_from(42);
        assert_eq!(max_val.as_reveal(), &42);
        
        let min_val: Min<i32> = Lattice::lattice_from(42);
        assert_eq!(min_val.as_reveal(), &42);
    }

    /// Test LatticeFrom with collections
    #[test]
    fn test_lattice_from_collections() {
        use std::collections::HashSet;
        
        let set_val: SetUnion<HashSet<i32>> = Lattice::lattice_from(HashSet::from([1, 2, 3]));
        assert_eq!(set_val.as_reveal().len(), 3);
    }
}
