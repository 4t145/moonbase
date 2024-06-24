use std::{future::IntoFuture, pin::Pin};

use futures::Future;
use tsuki_scheduler::prelude::*;

use crate::{daemon::Daemon, extract::ExtractFrom, resource::MoonbaseResource, Moonbase};

impl MoonbaseResource for AsyncSchedulerClient<Tokio> {}

pub use tsuki_scheduler;

pub type TsukiSchedulerClient = AsyncSchedulerClient<Tokio>;

pub struct TsukiScheduler {
    context: Moonbase,
    runner: AsyncSchedulerRunner<Tokio>,
}

impl ExtractFrom<Moonbase> for TsukiScheduler {
    async fn extract_from(context: &Moonbase) -> Self {
        let runner = AsyncSchedulerRunner::tokio();
        context.set_resource(runner.client());

        TsukiScheduler {
            runner,
            // shutdown_signal: context.shutdown_signal(),
            context: context.clone(),
        }
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
