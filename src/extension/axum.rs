use std::borrow::Cow;
use std::convert::Infallible;

use axum::http::request::Parts;
use axum::response::IntoResponse;

use crate::components::{ComponentName, MoonbaseComponent};
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
