use std::{convert::Infallible, future::IntoFuture, pin::Pin, time::Duration};

use futures::Future;
use moonbase::{
    context::ContextExt,
    daemon::Daemon,
    extension::tsuki_scheduler::{TsukiScheduler, TsukiSchedulerClient},
    extract::{ExtractFrom, TryExtractFrom},
    signal::{Signal, SignalKey},
    AppContext, Moonbase,
};
use tsuki_scheduler::{Task, TaskUid};

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async_main()).unwrap();
}

// fn log<C, A, H>(context: &C, handler: &H) -> A::Ret
// where
//     C: Context,
//     A: Adapter,
//     H: Handler<C, A> + Clone,
// {
//     println!("Calling Handler");
//     let result = handler.clone().apply(context);
//     println!("Calling Handler");
//     result
// }

async fn async_main() -> anyhow::Result<()> {
    let moonbase = Moonbase::new();
    // moonbase.load_module(Tokio::default()).await?;
    moonbase.call(init_resource).await?;
    moonbase.call(async_with_result).await?;
    moonbase.call(no_result).await;
    moonbase.run_daemon::<TsukiScheduler>().await?;
    let client = moonbase.get_resource::<TsukiSchedulerClient>().unwrap();

    let handle = moonbase.run_daemon::<MyDaemon>().await?;
    moonbase.set_signal(SignalKey::from_type::<Moonbase>(), Signal::new());
    client.add_task(
        TaskUid::uuid(),
        Task::tokio(None, || async {
            println!("Hello, TsukiScheduler!");
        }),
    );
    handle.wait().await;
    let signal = moonbase
        .get_signal(&SignalKey::from_type::<Moonbase>())
        .unwrap();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        moonbase.trigger_signal(&SignalKey::from_type::<Moonbase>());
    });
    signal.wait().await;
    Ok(())
}

async fn init_resource() -> anyhow::Result<()> {
    Ok(())
}

async fn init_resource_infallible() {}

async fn async_with_result(
    res: MyResource,
    res2: Result<MyFallibleResource, anyhow::Error>,
) -> Result<(), Infallible> {
    Ok(())
}
async fn no_result() {}
fn sync() {}

pub struct MyResource {}
impl ExtractFrom<Moonbase> for MyResource {
    async fn extract_from(_moonbase: &Moonbase) -> Self {
        MyResource {}
    }
}
impl TryExtractFrom<Moonbase> for MyResource {
    type Error = Infallible;
    async fn try_extract_from(_moonbase: &Moonbase) -> Result<Self, Infallible> {
        Ok(MyResource {})
    }
}
pub struct MyFallibleResource {}
impl TryExtractFrom<Moonbase> for MyFallibleResource {
    type Error = anyhow::Error;
    async fn try_extract_from(_moonbase: &Moonbase) -> anyhow::Result<Self> {
        Ok(MyFallibleResource {})
    }
}
pub struct MyDaemon {}

impl ExtractFrom<Moonbase> for MyDaemon {
    async fn extract_from(_moonbase: &Moonbase) -> Self {
        MyDaemon {}
    }
}

impl TryExtractFrom<Moonbase> for MyDaemon {
    type Error = Infallible;

    async fn try_extract_from(_moonbase: &Moonbase) -> Result<Self, Self::Error> {
        Ok(MyDaemon {})
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
