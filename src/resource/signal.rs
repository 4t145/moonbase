use std::pin::Pin;

use futures::Future;

use super::MoonbaseResource;

pub struct AppStopSignal {
    inner: Pin<Box<dyn Future<Output = ()> + Send + Sync>>,
}

impl MoonbaseResource for AppStopSignal {}
