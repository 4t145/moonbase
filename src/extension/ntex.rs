use anyhow::Context;
pub use ntex::service::ServiceFactory;
use ntex::web;

pub use ntex::service::Service;

use crate::{resource::MoonbaseResource, Moonbase};

impl<E, R> ntex::web::FromRequest<E> for crate::resource::Resource<R>
where
    R: MoonbaseResource,
{
    type Error = anyhow::Error;

    async fn from_request(
        req: &web::HttpRequest,
        _payload: &mut ntex::http::Payload,
    ) -> Result<Self, Self::Error> {
        let moonbase = req
            .app_state::<crate::Moonbase>()
            .with_context(|| "Moonbase not found")?;
        let resource = moonbase
            .get_resource::<R>()
            .with_context(|| format!("Resource {} not found", std::any::type_name::<R>()))?;
        Ok(crate::resource::Resource(resource))
    }
}

impl<E> ntex::web::FromRequest<E> for Moonbase
{
    type Error = anyhow::Error;

    async fn from_request(
        req: &web::HttpRequest,
        _payload: &mut ntex::http::Payload,
    ) -> Result<Self, Self::Error> {
        let moonbase = req
            .app_state::<crate::Moonbase>()
            .with_context(|| "Moonbase not found")?;
        Ok(moonbase.clone())
    }
}