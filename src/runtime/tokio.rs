use crate::resource::MoonbaseResource;

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
}

impl MoonbaseResource for Tokio {}
