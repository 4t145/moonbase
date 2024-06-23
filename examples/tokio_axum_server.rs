use std::{future::IntoFuture, pin::Pin};

use futures::Future;
use moonbase::{
    context::ContextExt, daemon::Daemon, extract::Extract, runtime::Tokio, AppContext, Moonbase,
};

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async_main()).unwrap();
}

async fn async_main() -> anyhow::Result<()> {
    let moonbase = Moonbase::new();
    moonbase.load_module(Tokio::default()).await?;
    moonbase.call(init_resource_infallible).await;
    moonbase.fallible_call(init_resource).await?;
    let handle = moonbase.run_daemon::<MyDaemon>().await?;
    handle.wait().await;
    Ok(())
}

async fn init_resource() -> anyhow::Result<()> {
    Ok(())
}

async fn init_resource_infallible() {}

pub struct MyDaemon {}

impl Extract<Moonbase> for MyDaemon {
    async fn extract(_moonbase: &Moonbase) -> Self {
        MyDaemon {}
    }
}

impl IntoFuture for MyDaemon {
    type Output = Self;
    type IntoFuture = Pin<Box<dyn Future<Output = Self> + Send>>;

    fn into_future(self) -> Pin<Box<dyn Future<Output = Self> + Send>> {
        Box::pin(async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            println!("Hello, Moonbase!");
            self
        })
    }
}

impl Daemon<Moonbase> for MyDaemon {
    fn max_restart_time(&self) -> Option<usize> {
        Some(2)
    }

    fn cool_down_time(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs(1))
    }
}
pub struct SomeTransitionContext {
    app: AppContext,
}

impl AsRef<AppContext> for SomeTransitionContext {
    fn as_ref(&self) -> &AppContext {
        &self.app
    }
}


