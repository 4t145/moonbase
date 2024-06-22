use std::{
    any::{Any, TypeId},
    borrow::Cow,
    hash::{DefaultHasher, Hash, Hasher},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentName<T: Any> {
    /// readable name of the component
    readable_name: Cow<'static, str>,
    /// hash cache of the readable name
    type_id: TypeId,
    hash: u64,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T: Any> ComponentName<T> {
    fn hash_id_and_type(id: &str, type_id: TypeId) -> u64 {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        type_id.hash(&mut hasher);
        hasher.finish()
    }
    /// create a new ComponentName
    pub fn new(id: impl Into<Cow<'static, str>>) -> Self {
        let id = id.into();
        let type_id = TypeId::of::<T>();
        let hash = Self::hash_id_and_type(&id, type_id);
        ComponentName {
            readable_name: id,
            hash,
            type_id: TypeId::of::<T>(),
            _marker: std::marker::PhantomData,
        }
    }
    /// get the readable name
    pub fn readable_name(&self) -> &str {
        &self.readable_name
    }
    /// get the hash value of the readable name
    pub fn hash(&self) -> u64 {
        self.hash
    }
    /// reset the readable name, and the hash value will be recalculated
    pub fn reset(&mut self, id: impl Into<Cow<'static, str>>) {
        let id = id.into();
        self.hash = Self::hash_id_and_type(&id, self.type_id);
        self.readable_name = id;
    }
}

impl<T: Any> Hash for ComponentName<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash)
    }
}
