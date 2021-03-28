use std::any::Any;

/// We need this because `TypeId` can't be deserialized or serialized directly, but this can be done using hashing. However, there is small possibility that different types will have intersection by hashes of their type ids.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct TypeId(u64);

impl TypeId {
    pub fn of<T: Any + 'static>() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::any::TypeId::of::<T>().hash(&mut hasher);
        Self(hasher.finish())
    }
}
