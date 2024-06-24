use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

#[derive(Debug, Clone, Default)]
pub struct AnyMap(HashMap<TypeId, Arc<dyn Any + Send + Sync>>);

impl AnyMap {
    pub fn new() -> Self {
        AnyMap(HashMap::new())
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) -> Option<Arc<T>> {
        let replaced = self.0.insert(TypeId::of::<T>(), Arc::new(value));
        replaced.map(|resource| Arc::downcast(resource).expect("fail to downcast resource"))
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.0
            .get(&TypeId::of::<T>())
            .cloned()
            .map(|resource| Arc::downcast(resource).expect("fail to downcast resource"))
    }

    pub fn remove<T: Any + Send + Sync>(&mut self) -> Option<Arc<T>> {
        self.0
            .remove(&TypeId::of::<T>())
            .map(|resource| Arc::downcast(resource).expect("fail to downcast resource"))
    }

    pub fn has<T: Any + Send + Sync>(&self) -> bool {
        self.0.contains_key(&TypeId::of::<T>())
    }
}
