use std::{any::Any, ops::Deref, sync::Arc};

use crate::{extract::Extract, Moonbase};
pub trait MoonbaseResource: Send + Sync + Any {}

/// Resource is for a global unique data for a moonbase, in other words, it is a singleton.
#[derive(Debug, Clone)]
pub struct Resource<T>(Arc<T>);

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
        Self(Arc::new(data))
    }
    pub fn into_arc(self) -> Arc<T> {
        self.0
    }
}

impl<T> Extract<Moonbase> for Resource<T>
where
    T: MoonbaseResource,
{
    async fn extract(context: &Moonbase) -> Self {
        let resource = context.get_resource::<T>().expect("fail to get resource");
        Resource(resource)
    }
}

impl<T> Extract<Moonbase> for Option<Resource<T>>
where
    T: MoonbaseResource,
{
    async fn extract(context: &Moonbase) -> Self {
        context.get_resource::<T>().map(Resource)
    }
}
