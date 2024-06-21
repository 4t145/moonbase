use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    future::Future,
    pin::Pin,
    sync::{Arc, OnceLock},
};

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, Default)]
pub struct Entity {
    entity_id: (u64, u64),
}

pub trait Component: Any + Send + Sync + 'static {}

pub struct ComponentStorage {
    entity_to_component: HashMap<Entity, HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    component_to_entity: HashMap<TypeId, HashSet<Entity>>,
}

impl ComponentStorage {
    pub fn add_relationship<C: Component>(&mut self, entity: Entity, component: C) {
        let component_id = TypeId::of::<C>();
        self.entity_to_component
            .entry(entity)
            .or_default()
            .insert(component_id, Box::new(component));
        self.component_to_entity
            .entry(component_id)
            .or_default()
            .insert(entity);
    }
    pub fn delete_relationship<C: Component>(&mut self, entity: Entity) {
        let component_id = TypeId::of::<C>();
        if let Some(components) = self.entity_to_component.get_mut(&entity) {
            components.remove(&component_id);
        }
        if let Some(entities) = self.component_to_entity.get_mut(&component_id) {
            entities.remove(&entity);
        }
    }
}
