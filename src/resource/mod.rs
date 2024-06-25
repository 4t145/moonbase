pub mod update;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

use crossbeam::sync::ShardedLock;
// e.g. signal -> config -> web server 
use crate::{extract::ExtractFrom, Moonbase};
pub trait MoonbaseResource: Send + Sync + Any + Clone {
    fn subscribe_changes(&self) {}
}

/// Resource is for a global unique data for a moonbase, in other words, it is a singleton.
#[derive(Debug, Clone)]
pub struct Resource<T>(T);

impl<T> AsRef<T> for Resource<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Resource<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Resource<T> {
    pub fn new(data: T) -> Self {
        Self(data)
    }
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> ExtractFrom<Moonbase> for Resource<T>
where
    T: MoonbaseResource,
{
    async fn extract_from(context: &Moonbase) -> Self {
        let resource = context.get_resource::<T>().expect("fail to get resource");
        Resource(resource)
    }
}

impl<T> ExtractFrom<Moonbase> for Option<Resource<T>>
where
    T: MoonbaseResource,
{
    async fn extract_from(context: &Moonbase) -> Self {
        context.get_resource::<T>().map(Resource)
    }
}

#[derive(Debug, Default)]
pub struct ResourceRepositoryInner {
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

pub type ResourceRepository = Arc<ShardedLock<ResourceRepositoryInner>>;

impl ResourceRepositoryInner {
    pub fn new() -> Self {
        ResourceRepositoryInner::default()
    }

    pub fn insert<T: MoonbaseResource>(&mut self, component: T) -> Option<T> {
        let id = TypeId::of::<T>();
        let replaced = self.components.insert(id, Box::new(component));
        replaced.map(|component| *component.downcast::<T>().expect("type mismatch").clone())
    }

    pub fn remove<T: MoonbaseResource>(&mut self) -> Option<T> {
        let id = TypeId::of::<T>();

        self.components
            .remove(&id)
            .map(|component| *component.downcast::<T>().expect("type mismatch").clone())
    }

    pub fn get<T: MoonbaseResource>(&self) -> Option<T> {
        let id = TypeId::of::<T>();

        self.components.get(&id).map(|component| {
            component
                .downcast_ref::<T>()
                .cloned()
                .expect("type mismatch")
        })
    }

    pub fn has<T: MoonbaseResource>(&self) -> bool {
        let id = TypeId::of::<T>();
        self.components.contains_key(&id)
    }
}

impl Moonbase {
    pub fn set_resource<T: MoonbaseResource>(&self, resource: T) {
        let mut resources = self.resources.write().unwrap();
        resources.insert(resource);
    }
    pub fn get_resource<T: MoonbaseResource>(&self) -> Option<T> {
        let resources = self.resources.read().unwrap();
        resources.get::<T>()
    }
    pub fn remove_resource<T: MoonbaseResource>(&self) -> Option<T> {
        let mut resources = self.resources.write().unwrap();
        resources.remove::<T>()
    }
    pub fn has_resource<T: MoonbaseResource>(&self) -> bool {
        let resources = self.resources.read().unwrap();
        resources.has::<T>()
    }
}