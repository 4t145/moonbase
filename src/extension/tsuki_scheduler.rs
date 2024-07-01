use std::{convert::Infallible, future::IntoFuture, pin::Pin};

use futures::Future;
use tsuki_scheduler::prelude::*;

use crate::{
    daemon::Daemon,
    extract::{ExtractFrom, TryExtractFrom},
    resource::MoonbaseResource,
    Moonbase,
};

impl MoonbaseResource for AsyncSchedulerClient<Tokio> {}

pub use tsuki_scheduler;

pub type TsukiSchedulerClient = AsyncSchedulerClient<Tokio>;

#[derive(Debug)]
pub struct TsukiScheduler {
    runner: AsyncSchedulerRunner<Tokio>,
}

impl ExtractFrom<Moonbase> for TsukiScheduler {
    async fn extract_from(context: &Moonbase) -> Self {
        let runner = AsyncSchedulerRunner::tokio();
        context.set_resource(runner.client());

        TsukiScheduler { runner }
    }
}

impl TryExtractFrom<Moonbase> for TsukiScheduler {
    type Error = Infallible;

    async fn try_extract_from(context: &Moonbase) -> Result<Self, Self::Error> {
        Ok(TsukiScheduler::extract_from(context).await)
    }
}

impl IntoFuture for TsukiScheduler {
    type Output = Self;
    type IntoFuture = Pin<Box<dyn Future<Output = Self> + Send>>;

    fn into_future(mut self) -> Pin<Box<dyn Future<Output = Self> + Send>> {
        Box::pin(async {
            self.runner = self.runner.run().await;
            self
        })
    }
}

impl Daemon<Moonbase> for TsukiScheduler {}
