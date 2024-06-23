use crate::{module::Module, resource::MoonbaseResource, Moonbase};

#[derive(Debug, Clone)]
pub struct Tokio {
    pub inner: tokio::runtime::Handle,
}

impl Default for Tokio {
    fn default() -> Self {
        Self {
            inner: tokio::runtime::Handle::current(),
        }
    }
}

impl super::Runtime for Tokio {
    fn spawn<F>(&self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.inner.spawn(future);
    }

    fn sleep(&self, duration: std::time::Duration) -> impl std::future::Future<Output = ()> + Send {
        tokio::time::sleep(duration)
    }
}

impl MoonbaseResource for Tokio {}
impl Module<Moonbase> for Tokio {
    fn initialize(
        self,
        context: &Moonbase,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        context.set_resource(self);
        async move { Ok(()) }
    }
}
