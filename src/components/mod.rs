use std::{any::Any, collections::HashMap, sync::Arc};
mod name;
use crossbeam::sync::ShardedLock;
pub use name::*;
use crate::Moonbase;
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, Default)]
pub struct Entity {
    entity_id: (u64, u64),
}

pub trait MoonbaseComponent: Any + Clone + Send + Sync + 'static {}
#[derive(Debug, Default)]
pub struct ComponentRepositoryInner {
    components: HashMap<u64, Box<dyn Any + Send + Sync>>,
}

pub type ComponentRepository = Arc<ShardedLock<ComponentRepositoryInner>>;

impl ComponentRepositoryInner {
    pub fn new() -> Self {
        ComponentRepositoryInner::default()
    }

    pub fn insert<T: MoonbaseComponent>(
        &mut self,
        name: &ComponentName<T>,
        component: T,
    ) -> Option<T> {
        let id = name.hash();
        let replaced = self.components.insert(id, Box::new(component));
        replaced.map(|component| *component.downcast::<T>().expect("type mismatch").clone())
    }

    pub fn remove<T: MoonbaseComponent>(&mut self, name: &ComponentName<T>) -> Option<T> {
        let id = name.hash();
        self.components
            .remove(&id)
            .map(|component| *component.downcast::<T>().expect("type mismatch").clone())
    }

    pub fn get<T: MoonbaseComponent>(&self, name: &ComponentName<T>) -> Option<T> {
        let id = name.hash();
        self.components.get(&id).map(|component| {
            component
                .downcast_ref::<T>()
                .expect("type mismatch")
                .clone()
        })
    }

    pub fn iter<T: MoonbaseComponent>(&self) -> impl Iterator<Item = T> + '_ {
        self.components
            .values()
            .filter_map(|component| component.downcast_ref::<T>().cloned())
    }
}



impl Moonbase {
    pub fn set_component<T: MoonbaseComponent>(
        &self,
        name: &ComponentName<T>,
        component: T,
    ) {
        let mut components = self.components.write().unwrap();
        components.insert(name, component);
    }
    pub fn get_component<T: MoonbaseComponent>(
        &self,
        name: &ComponentName<T>,
    ) -> Option<T> {
        let components = self.components.read().unwrap();
        components.get(name)
    }
    pub fn remove_component<T: MoonbaseComponent>(
        &self,
        name: &ComponentName<T>,
    ) -> Option<T> {
        let mut components = self.components.write().unwrap();
        components.remove(name)
    }
    pub fn has_component<T: MoonbaseComponent>(&self, name: &ComponentName<T>) -> bool {
        let components = self.components.read().unwrap();
        components.get(name).is_some()
    }
}
