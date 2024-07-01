use crate::components::{ComponentName, MoonbaseComponent};
use crate::resource::{MoonbaseResource, Resource};
use crate::Moonbase;

impl MoonbaseComponent for axum::Router<Moonbase> {}

impl Moonbase {
    pub fn insert_axum_router(
        &self,
        component_name: &ComponentName<axum::Router<Moonbase>>,
        router: axum::Router<Moonbase>,
    ) {
        self.set_component(component_name, router)
    }
    pub fn remove_axum_router(
        &self,
        component_name: &ComponentName<axum::Router<Moonbase>>,
    ) -> Option<axum::Router<Moonbase>> {
        self.remove_component(component_name)
    }
    pub fn collect_axum_routes(&self) -> axum::Router<Moonbase> {
        let wg = self.components.write().expect("never poisoned");
        let router = wg
            .iter::<axum::Router<Moonbase>>()
            .fold(axum::Router::<Moonbase>::default(), |router, component| {
                router.merge(component)
            });
        router
    }
}

#[async_trait::async_trait]
impl axum::extract::FromRequestParts<Moonbase> for Moonbase {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &Moonbase,
    ) -> Result<Self, Self::Rejection> {
        Ok(state.clone())
    }
}

#[async_trait::async_trait]
impl<R> axum::extract::FromRequestParts<Moonbase> for Resource<R>
where
    R: MoonbaseResource,
{
    type Rejection = String;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &Moonbase,
    ) -> Result<Self, Self::Rejection> {
        let resource = state.get_resource::<R>();
        match resource {
            Some(resource) => Ok(Resource(resource)),
            None => Err(format!("Resource {} not found", std::any::type_name::<R>())),
        }
    }
}
