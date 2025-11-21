//! Comprehensive unit tests for the UnionFind data structure.
//!
//! This module tests all functionality of the UnionFind implementation, including:
//! - Basic union and find operations
//! - Path compression
//! - Equivalence checking with same_set
//! - Edge cases and complex union patterns

use dfir_lang::union_find::UnionFind;
use slotmap::SlotMap;

#[test]
fn test_new_union_find() {
    let _uf: UnionFind<slotmap::DefaultKey> = UnionFind::new();
    // Just verify it compiles and runs
}

#[test]
fn test_with_capacity() {
    let _uf: UnionFind<slotmap::DefaultKey> = UnionFind::with_capacity(100);
    // Just verify it compiles and runs with capacity
}

#[test]
fn test_default() {
    let _uf: UnionFind<slotmap::DefaultKey> = UnionFind::default();
    // Verify default implementation works
}

#[test]
fn test_single_element() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    
    let mut uf = UnionFind::new();
    
    // Element should be in its own set (representative of itself)
    assert_eq!(uf.find(a), a);
}

#[test]
fn test_two_elements_no_union() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    let b = sm.insert(());
    
    let mut uf = UnionFind::new();
    
    // Elements should not be in the same set
    assert!(!uf.same_set(a, b));
    
    // Each should be its own representative
    assert_eq!(uf.find(a), a);
    assert_eq!(uf.find(b), b);
}

#[test]
fn test_basic_union() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    let b = sm.insert(());
    
    let mut uf = UnionFind::new();
    
    // Initially not in same set
    assert!(!uf.same_set(a, b));
    
    // Union them
    uf.union(a, b);
    
    // Now they should be in the same set
    assert!(uf.same_set(a, b));
}

#[test]
fn test_union_is_symmetric() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    let b = sm.insert(());
    
    let mut uf1 = UnionFind::new();
    uf1.union(a, b);
    
    let mut uf2 = UnionFind::new();
    uf2.union(b, a);
    
    // Both orders should produce the same result
    assert!(uf1.same_set(a, b));
    assert!(uf2.same_set(a, b));
}

#[test]
fn test_transitive_union() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    let b = sm.insert(());
    let c = sm.insert(());
    
    let mut uf = UnionFind::new();
    
    // Union a with b, then b with c
    uf.union(a, b);
    uf.union(b, c);
    
    // All three should be in the same set
    assert!(uf.same_set(a, b));
    assert!(uf.same_set(b, c));
    assert!(uf.same_set(a, c));
}

#[test]
fn test_multiple_disjoint_sets() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    let b = sm.insert(());
    let c = sm.insert(());
    let d = sm.insert(());
    
    let mut uf = UnionFind::new();
    
    // Create two separate sets: {a, b} and {c, d}
    uf.union(a, b);
    uf.union(c, d);
    
    // Check within-set relationships
    assert!(uf.same_set(a, b));
    assert!(uf.same_set(c, d));
    
    // Check across-set relationships (should be different)
    assert!(!uf.same_set(a, c));
    assert!(!uf.same_set(a, d));
    assert!(!uf.same_set(b, c));
    assert!(!uf.same_set(b, d));
}

#[test]
fn test_union_already_same_set() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    let b = sm.insert(());
    
    let mut uf = UnionFind::new();
    
    // Union twice
    uf.union(a, b);
    uf.union(a, b); // Should be a no-op
    
    // Should still be in same set
    assert!(uf.same_set(a, b));
}

#[test]
fn test_long_chain() {
    let mut sm = SlotMap::new();
    let keys: Vec<_> = (0..10).map(|_| sm.insert(())).collect();
    
    let mut uf = UnionFind::new();
    
    // Create a long chain: 0-1, 1-2, 2-3, ..., 8-9
    for i in 0..9 {
        uf.union(keys[i], keys[i + 1]);
    }
    
    // All elements should be in the same set
    for i in 0..10 {
        for j in 0..10 {
            assert!(uf.same_set(keys[i], keys[j]));
        }
    }
}

#[test]
fn test_complex_union_pattern() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    let b = sm.insert(());
    let c = sm.insert(());
    let d = sm.insert(());
    let e = sm.insert(());
    
    let mut uf = UnionFind::new();
    
    // Complex pattern: a-b, c-d, then merge via e
    uf.union(a, b);
    uf.union(c, d);
    uf.union(b, e);
    uf.union(e, c);
    
    // All should be in the same set
    assert!(uf.same_set(a, b));
    assert!(uf.same_set(a, c));
    assert!(uf.same_set(a, d));
    assert!(uf.same_set(a, e));
    assert!(uf.same_set(b, c));
    assert!(uf.same_set(b, d));
    assert!(uf.same_set(b, e));
    assert!(uf.same_set(c, d));
    assert!(uf.same_set(c, e));
    assert!(uf.same_set(d, e));
}

#[test]
fn test_find_updates_path() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    let b = sm.insert(());
    let c = sm.insert(());
    
    let mut uf = UnionFind::new();
    
    // Create chain a -> b -> c
    uf.union(a, b);
    uf.union(b, c);
    
    // Find a (should compress path)
    let repr_a = uf.find(a);
    let repr_b = uf.find(b);
    let repr_c = uf.find(c);
    
    // All should have the same representative
    assert_eq!(repr_a, repr_b);
    assert_eq!(repr_b, repr_c);
}

#[test]
fn test_clone() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    let b = sm.insert(());
    let c = sm.insert(());
    
    let mut uf1 = UnionFind::new();
    uf1.union(a, b);
    
    let mut uf2 = uf1.clone();
    
    // Both should have the same state
    assert!(uf1.same_set(a, b));
    assert!(uf2.same_set(a, b));
    assert!(!uf1.same_set(a, c));
    assert!(!uf2.same_set(a, c));
    
    // Modifications to uf2 shouldn't affect uf1
    uf2.union(b, c);
    assert!(uf2.same_set(a, c));
    assert!(!uf1.same_set(a, c));
}

#[test]
fn test_many_elements() {
    let mut sm = SlotMap::new();
    let keys: Vec<_> = (0..100).map(|_| sm.insert(())).collect();
    
    let mut uf = UnionFind::new();
    
    // Union all even-indexed elements together
    for i in (0..100).step_by(2) {
        if i > 0 {
            uf.union(keys[0], keys[i]);
        }
    }
    
    // Union all odd-indexed elements together
    for i in (1..100).step_by(2) {
        uf.union(keys[1], keys[i]);
    }
    
    // Check even elements are in the same set
    for i in (0..100).step_by(2) {
        assert!(uf.same_set(keys[0], keys[i]));
    }
    
    // Check odd elements are in the same set
    for i in (1..100).step_by(2) {
        assert!(uf.same_set(keys[1], keys[i]));
    }
    
    // Check even and odd are in different sets
    for i in (0..100).step_by(2) {
        for j in (1..100).step_by(2) {
            assert!(!uf.same_set(keys[i], keys[j]));
        }
    }
}

#[test]
fn test_star_pattern() {
    let mut sm = SlotMap::new();
    let center = sm.insert(());
    let satellites: Vec<_> = (0..10).map(|_| sm.insert(())).collect();
    
    let mut uf = UnionFind::new();
    
    // Union all satellites to the center (star pattern)
    for &satellite in &satellites {
        uf.union(center, satellite);
    }
    
    // All satellites should be connected to center
    for &satellite in &satellites {
        assert!(uf.same_set(center, satellite));
    }
    
    // All satellites should be connected to each other
    for i in 0..satellites.len() {
        for j in 0..satellites.len() {
            assert!(uf.same_set(satellites[i], satellites[j]));
        }
    }
}

#[test]
fn test_merge_two_large_sets() {
    let mut sm = SlotMap::new();
    let keys: Vec<_> = (0..20).map(|_| sm.insert(())).collect();
    
    let mut uf = UnionFind::new();
    
    // Create two sets of 10 elements each
    for i in 0..9 {
        uf.union(keys[i], keys[i + 1]);
    }
    for i in 10..19 {
        uf.union(keys[i], keys[i + 1]);
    }
    
    // Verify two separate sets exist
    assert!(uf.same_set(keys[0], keys[9]));
    assert!(uf.same_set(keys[10], keys[19]));
    assert!(!uf.same_set(keys[0], keys[10]));
    
    // Merge the two sets
    uf.union(keys[9], keys[10]);
    
    // Now all should be in one set
    for i in 0..20 {
        for j in 0..20 {
            assert!(uf.same_set(keys[i], keys[j]));
        }
    }
}

#[test]
fn test_same_set_reflexive() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    
    let mut uf = UnionFind::new();
    
    // Element should be in the same set as itself
    assert!(uf.same_set(a, a));
}

#[test]
fn test_same_set_symmetric() {
    let mut sm = SlotMap::new();
    let a = sm.insert(());
    let b = sm.insert(());
    
    let mut uf = UnionFind::new();
    uf.union(a, b);
    
    // same_set should be symmetric
    assert_eq!(uf.same_set(a, b), uf.same_set(b, a));
}
