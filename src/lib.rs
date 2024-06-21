use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use extract::Extract;
use resource::MoonbaseResource;

pub mod cluster;
pub mod components;
pub mod context;
pub mod extract;
pub mod handler;
pub mod resource;

#[derive(Debug, Clone, Default)]
pub struct Moonbase {
    id: u64,
    resources: Arc<RwLock<HashMap<TypeId, Arc<dyn Send + Sync + Any>>>>,
}

impl Extract<Moonbase> for Moonbase {
    async fn extract(context: &Moonbase) -> Self {
        context.clone()
    }
}

impl Moonbase {
    pub fn set_resource<T: MoonbaseResource>(&self, resource: T) {
        let mut resources = self.resources.write().unwrap();
        resources.insert(TypeId::of::<T>(), Arc::new(resource));
    }
    pub fn get_resource<T: MoonbaseResource>(&self) -> Option<Arc<T>> {
        let resources = self.resources.read().unwrap();
        resources
            .get(&TypeId::of::<T>())
            .cloned()
            .map(|resource| Arc::downcast(resource).expect("fail to downcast resource"))
    }
}
