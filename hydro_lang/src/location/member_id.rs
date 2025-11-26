use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

#[repr(transparent)]
pub struct MemberId<Tag> {
    pub raw_id: u32,
    pub(crate) _phantom: PhantomData<Tag>,
}

impl<Tag> MemberId<Tag> {
    pub fn from_raw(id: u32) -> Self {
        MemberId {
            raw_id: id,
            _phantom: PhantomData,
        }
    }
}

impl<Tag> Debug for MemberId<Tag> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MemberId::<{}>({})",
            std::any::type_name::<Tag>(),
            self.raw_id
        )
    }
}

impl<Tag> Display for MemberId<Tag> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MemberId::<{}>({})",
            std::any::type_name::<Tag>(),
            self.raw_id
        )
    }
}

impl<Tag> Clone for MemberId<Tag> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Tag> Copy for MemberId<Tag> {}

impl<Tag> Serialize for MemberId<Tag> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.raw_id.serialize(serializer)
    }
}

impl<'de, Tag> Deserialize<'de> for MemberId<Tag> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        u32::deserialize(deserializer).map(|id| MemberId {
            raw_id: id,
            _phantom: PhantomData,
        })
    }
}

impl<Tag> PartialEq for MemberId<Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.raw_id == other.raw_id
    }
}

impl<Tag> Eq for MemberId<Tag> {}

impl<Tag> Hash for MemberId<Tag> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw_id.hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    struct TestTag;

    #[test]
    fn test_member_id_equality_and_hashing() {
        let id1 = MemberId::<TestTag>::from_raw(42);
        let id2 = MemberId::<TestTag>::from_raw(42);
        let id3 = MemberId::<TestTag>::from_raw(99);
        
        // Test equality
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        
        // Test copy/clone
        let id1_copy = id1;
        assert_eq!(id1, id1_copy);
        
        let id1_clone = id1.clone();
        assert_eq!(id1, id1_clone);
        
        // Test hashing by using in a HashSet
        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2); // Should not increase set size (same as id1)
        set.insert(id3);
        
        assert_eq!(set.len(), 2); // Only id1 and id3 are unique
        assert!(set.contains(&id1));
        assert!(set.contains(&id3));
    }

    #[test]
    fn test_member_id_serialization() {
        let original = MemberId::<TestTag>::from_raw(123);
        
        // Serialize to JSON
        let serialized = serde_json::to_string(&original).unwrap();
        assert_eq!(serialized, "123"); // Should serialize as just the raw_id
        
        // Deserialize back
        let deserialized: MemberId<TestTag> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
        assert_eq!(deserialized.raw_id, 123);
    }

    #[test]
    fn test_member_id_display_and_debug() {
        let id = MemberId::<TestTag>::from_raw(777);
        
        // Test Display formatting
        let display_str = format!("{}", id);
        assert!(display_str.contains("MemberId"));
        assert!(display_str.contains("TestTag"));
        assert!(display_str.contains("777"));
        
        // Test Debug formatting
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("MemberId"));
        assert!(debug_str.contains("TestTag"));
        assert!(debug_str.contains("777"));
    }
}
