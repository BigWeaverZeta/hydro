//! A Vec-based SlotMap-esque datastructure and corresponding Key type.

use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

/// A key into a SlotVec.
#[repr(transparent)]
pub struct Key<Tag: ?Sized> {
    index: usize,
    _phantom: PhantomData<Tag>,
}
impl<Tag: ?Sized> Key<Tag> {
    /// Creates a Key from a raw index. Avoid using this function directly.
    pub const fn from_raw(index: usize) -> Self {
        Key {
            index,
            _phantom: PhantomData,
        }
    }
}
impl<Tag: ?Sized> Clone for Key<Tag> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Tag: ?Sized> Copy for Key<Tag> {}
impl<Tag: ?Sized> PartialOrd for Key<Tag> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<Tag: ?Sized> Ord for Key<Tag> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}
impl<Tag: ?Sized> PartialEq for Key<Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<Tag: ?Sized> Eq for Key<Tag> {}
impl<Tag: ?Sized> Hash for Key<Tag> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}
impl<Tag: ?Sized> Debug for Key<Tag> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Key({})", self.index)
    }
}
impl<Tag: ?Sized> Display for Key<Tag> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

/// A Vec-based SlotMap-esque datastructure without removes.
///
/// Analogous to [`slotmap::SlotMap`], but avoids the overhead of tracking removed keys.
#[repr(transparent)]
pub struct SlotVec<Tag: ?Sized, Val> {
    slots: Vec<Val>,
    _phantom: PhantomData<Tag>,
}
impl<Tag: ?Sized, Val> SlotVec<Tag, Val> {
    /// Creates a new `SlotVec`.
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Inserts a value into the `SlotVec` and returns the key.
    pub fn insert(&mut self, value: Val) -> Key<Tag> {
        let key = Key::from_raw(self.slots.len());
        self.slots.push(value);
        key
    }

    /// Use the provided function to generate a value given the key and insert it into the `SlotVec`.
    pub fn insert_with_key<F>(&mut self, func: F) -> Key<Tag>
    where
        F: FnOnce(Key<Tag>) -> Val,
    {
        let key = Key::from_raw(self.slots.len());
        self.slots.push((func)(key));
        key
    }

    /// Returns a reference to the value associated with the key.
    pub fn get(&self, key: Key<Tag>) -> Option<&Val> {
        self.slots.get(key.index)
    }

    /// Returns a mutable reference to the value associated with the key.
    pub fn get_mut(&mut self, key: Key<Tag>) -> Option<&mut Val> {
        self.slots.get_mut(key.index)
    }

    /// Returns the number of elements in the `SlotVec`.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Returns true if the `SlotVec` is empty.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Iterate the key-value pairs, where the value is a shared reference.
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = (Key<Tag>, &'_ Val)> + ExactSizeIterator + FusedIterator + Clone
    {
        self.slots
            .iter()
            .enumerate()
            .map(|(idx, val)| (Key::from_raw(idx), val))
    }

    /// Iterate the key-value pairs, where the value is a exclusive reference.
    pub fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = (Key<Tag>, &'_ mut Val)> + ExactSizeIterator + FusedIterator
    {
        self.slots
            .iter_mut()
            .enumerate()
            .map(|(idx, val)| (Key::from_raw(idx), val))
    }

    /// Iterate over the values by shared reference.
    pub fn values(&self) -> std::slice::Iter<'_, Val> {
        self.slots.iter()
    }

    /// Iterate over the values by exclusive reference.
    pub fn values_mut(&mut self) -> std::slice::IterMut<'_, Val> {
        self.slots.iter_mut()
    }

    /// Iterate over the keys.
    pub fn keys(
        &self,
    ) -> impl '_ + DoubleEndedIterator<Item = Key<Tag>> + ExactSizeIterator + FusedIterator + Clone
    {
        self.iter().map(|(key, _)| key)
    }
}
impl<Tag: ?Sized, Val> Index<Key<Tag>> for SlotVec<Tag, Val> {
    type Output = Val;

    fn index(&self, key: Key<Tag>) -> &Self::Output {
        self.get(key).unwrap()
    }
}
impl<Tag: ?Sized, Val> IndexMut<Key<Tag>> for SlotVec<Tag, Val> {
    fn index_mut(&mut self, key: Key<Tag>) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}
impl<Key: ?Sized, Val> Default for SlotVec<Key, Val> {
    fn default() -> Self {
        Self::new()
    }
}

/// A secondary map used to associated data with keys from elements in an existing [`SlotVec`].
///
/// Analogous to [`slotmap::SecondaryMap`].
pub struct SecondarySlotVec<Tag: ?Sized, Val> {
    slots: Vec<Option<Val>>,
    _phantom: PhantomData<Tag>,
}
impl<Tag: ?Sized, Val> SecondarySlotVec<Tag, Val> {
    /// Creates a new `SecondarySlotVec`.
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Inserts a value into the `SecondarySlotVec` and returns the previous value associated with the key.
    pub fn insert(&mut self, key: Key<Tag>, value: Val) -> Option<Val> {
        if key.index >= self.slots.len() {
            self.slots.resize_with(key.index + 1, || None);
        }
        self.slots[key.index].replace(value)
    }

    /// Removes a value associated with the key from the `SecondarySlotVec` and returns it.
    pub fn remove(&mut self, key: Key<Tag>) -> Option<Val> {
        // TODO(mingwei): Shrink the vector?
        self.slots[key.index].take()
    }

    /// Returns a reference to the value associated with the key.
    pub fn get(&self, key: Key<Tag>) -> Option<&Val> {
        self.slots.get(key.index).and_then(|v| v.as_ref())
    }

    /// Returns a mutable reference to the value associated with the key.
    pub fn get_mut(&mut self, key: Key<Tag>) -> Option<&mut Val> {
        self.slots.get_mut(key.index).and_then(|v| v.as_mut())
    }

    /// Returns a mutable reference to the value associated with the key, inserting a default value
    /// if it doesn't yet exist.
    pub fn get_or_insert_with(&mut self, key: Key<Tag>, default: impl FnOnce() -> Val) -> &mut Val {
        if key.index >= self.slots.len() {
            self.slots.resize_with(key.index + 1, || None);
        }
        self.slots[key.index].get_or_insert_with(default)
    }

    /// Iterate the key-value pairs, where the value is a shared reference.
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = (Key<Tag>, &'_ Val)> + FusedIterator + Clone {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(idx, opt_val)| Some((Key::from_raw(idx), opt_val.as_ref()?)))
    }

    /// Iterate the key-value pairs, where the value is a exclusive reference.
    pub fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = (Key<Tag>, &'_ mut Val)> + FusedIterator {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(idx, opt_val)| Some((Key::from_raw(idx), opt_val.as_mut()?)))
    }

    /// Iterate over the values by shared reference.
    pub fn values(&self) -> impl DoubleEndedIterator<Item = &'_ Val> + FusedIterator + Clone {
        self.slots.iter().filter_map(Option::as_ref)
    }

    /// Iterate over the values by exclusive reference.
    pub fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &'_ mut Val> + FusedIterator {
        self.slots.iter_mut().filter_map(Option::as_mut)
    }

    /// Iterate over the keys.
    pub fn keys(&self) -> impl '_ + DoubleEndedIterator<Item = Key<Tag>> + FusedIterator + Clone {
        self.iter().map(|(key, _)| key)
    }
}
impl<Tag: ?Sized, Val> Default for SecondarySlotVec<Tag, Val> {
    fn default() -> Self {
        Self::new()
    }
}
impl<Tag: ?Sized, Val> Index<Key<Tag>> for SecondarySlotVec<Tag, Val> {
    type Output = Val;

    fn index(&self, key: Key<Tag>) -> &Self::Output {
        self.get(key).unwrap()
    }
}
impl<Tag: ?Sized, Val> IndexMut<Key<Tag>> for SecondarySlotVec<Tag, Val> {
    fn index_mut(&mut self, key: Key<Tag>) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTag;

    // --- SlotVec tests ---

    #[test]
    fn test_slotvec_new_empty() {
        let sv: SlotVec<TestTag, i32> = SlotVec::new();
        assert!(sv.is_empty());
        assert_eq!(sv.len(), 0);
    }

    #[test]
    fn test_slotvec_insert_get() {
        let mut sv: SlotVec<TestTag, &str> = SlotVec::new();
        let k0 = sv.insert("hello");
        let k1 = sv.insert("world");

        assert_eq!(sv.get(k0), Some(&"hello"));
        assert_eq!(sv.get(k1), Some(&"world"));
        assert_eq!(sv.len(), 2);
        assert!(!sv.is_empty());
    }

    #[test]
    fn test_slotvec_insert_with_key() {
        let mut sv: SlotVec<TestTag, String> = SlotVec::new();
        let k = sv.insert_with_key(|key| format!("key_index_{}", key));

        assert_eq!(sv.get(k), Some(&"key_index_0".to_string()));

        let k2 = sv.insert_with_key(|key| format!("key_index_{}", key));
        assert_eq!(sv.get(k2), Some(&"key_index_1".to_string()));
    }

    #[test]
    fn test_slotvec_index() {
        let mut sv: SlotVec<TestTag, i32> = SlotVec::new();
        let k0 = sv.insert(10);
        let k1 = sv.insert(20);

        assert_eq!(sv[k0], 10);
        assert_eq!(sv[k1], 20);

        sv[k0] = 99;
        assert_eq!(sv[k0], 99);
    }

    #[test]
    fn test_slotvec_get_out_of_range() {
        let sv: SlotVec<TestTag, i32> = SlotVec::new();
        let fake_key = Key::<TestTag>::from_raw(5);
        assert_eq!(sv.get(fake_key), None);
    }

    #[test]
    fn test_slotvec_iter() {
        let mut sv: SlotVec<TestTag, &str> = SlotVec::new();
        let k0 = sv.insert("a");
        let k1 = sv.insert("b");
        let k2 = sv.insert("c");

        let items: Vec<(Key<TestTag>, &&str)> = sv.iter().collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].0, k0);
        assert_eq!(*items[0].1, "a");
        assert_eq!(items[1].0, k1);
        assert_eq!(*items[1].1, "b");
        assert_eq!(items[2].0, k2);
        assert_eq!(*items[2].1, "c");
    }

    #[test]
    fn test_slotvec_keys_and_values() {
        let mut sv: SlotVec<TestTag, i32> = SlotVec::new();
        let k0 = sv.insert(100);
        let k1 = sv.insert(200);
        let k2 = sv.insert(300);

        let keys: Vec<Key<TestTag>> = sv.keys().collect();
        assert_eq!(keys, vec![k0, k1, k2]);

        let values: Vec<&i32> = sv.values().collect();
        assert_eq!(values, vec![&100, &200, &300]);
    }

    // --- SecondarySlotVec tests ---

    #[test]
    fn test_secondary_insert_and_get() {
        let mut ssv: SecondarySlotVec<TestTag, &str> = SecondarySlotVec::new();
        let key = Key::<TestTag>::from_raw(0);

        let old = ssv.insert(key, "hello");
        assert_eq!(old, None);
        assert_eq!(ssv.get(key), Some(&"hello"));
    }

    #[test]
    fn test_secondary_overwrite() {
        let mut ssv: SecondarySlotVec<TestTag, i32> = SecondarySlotVec::new();
        let key = Key::<TestTag>::from_raw(2);

        let old1 = ssv.insert(key, 10);
        assert_eq!(old1, None);

        let old2 = ssv.insert(key, 20);
        assert_eq!(old2, Some(10));
        assert_eq!(ssv.get(key), Some(&20));
    }

    #[test]
    fn test_secondary_remove() {
        let mut ssv: SecondarySlotVec<TestTag, i32> = SecondarySlotVec::new();
        let key = Key::<TestTag>::from_raw(1);

        ssv.insert(key, 42);
        let removed = ssv.remove(key);
        assert_eq!(removed, Some(42));
        assert_eq!(ssv.get(key), None);
    }

    #[test]
    fn test_secondary_get_or_insert_with() {
        let mut ssv: SecondarySlotVec<TestTag, String> = SecondarySlotVec::new();
        let key = Key::<TestTag>::from_raw(3);

        // First call inserts the default
        let val = ssv.get_or_insert_with(key, || "default".to_string());
        assert_eq!(val, "default");

        // Second call returns the existing value
        let val = ssv.get_or_insert_with(key, || "other".to_string());
        assert_eq!(val, "default");
    }

    #[test]
    fn test_secondary_sparse_keys() {
        let mut ssv: SecondarySlotVec<TestTag, i32> = SecondarySlotVec::new();
        let key5 = Key::<TestTag>::from_raw(5);

        ssv.insert(key5, 999);

        // Keys 0-4 should return None
        for i in 0..5 {
            assert_eq!(ssv.get(Key::<TestTag>::from_raw(i)), None);
        }
        assert_eq!(ssv.get(key5), Some(&999));
    }

    #[test]
    fn test_secondary_iter() {
        let mut ssv: SecondarySlotVec<TestTag, &str> = SecondarySlotVec::new();
        let k1 = Key::<TestTag>::from_raw(1);
        let k3 = Key::<TestTag>::from_raw(3);

        ssv.insert(k1, "one");
        ssv.insert(k3, "three");

        let items: Vec<(Key<TestTag>, &&str)> = ssv.iter().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, k1);
        assert_eq!(*items[0].1, "one");
        assert_eq!(items[1].0, k3);
        assert_eq!(*items[1].1, "three");
    }
}
