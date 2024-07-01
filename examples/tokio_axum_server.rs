use std::{convert::Infallible, future::IntoFuture, pin::Pin, time::Duration};

use axum::extract::State;
use futures::Future;
use moonbase::{
    components::ComponentName,
    context::ContextExt,
    daemon::Daemon,
    extension::tsuki_scheduler::{TsukiScheduler, TsukiSchedulerClient},
    extract::{ExtractFrom, TryExtractFrom},
    module::Module,
    runtime::Tokio,
    signal::{Signal, SignalKey},
    AppContext, Moonbase,
};
use tsuki_scheduler::{Task, TaskUid};

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async_main()).unwrap();
}

async fn async_main() -> anyhow::Result<()> {
    let moonbase = Moonbase::new();
    moonbase.call(init_resource).await?;
    moonbase.call(async_with_result).await?;
    moonbase.call(no_result).await;
    moonbase.load_module(Tokio::default()).await?;
    moonbase.load_module(HelloModule {}).await?;
    moonbase.run_daemon::<AxumServerDaemon>().await?;
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

    tokio::signal::ctrl_c().await.unwrap();
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

#[derive(Debug)]
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

pub struct HelloModule {}

async fn get_handler(State(mb): State<Moonbase>) -> String {
    let debug = format!("{:#?}", mb);
    debug
}

impl Module<Moonbase> for HelloModule {
    async fn initialize(self, context: Moonbase) -> anyhow::Result<()> {
        let router =
            axum::Router::new().route("/", axum::routing::get(get_handler));
        context.insert_axum_router(&ComponentName::new_type_tag::<Self>(), router);
        Ok(())
    }
}

#[derive(Debug)]
pub struct AxumServerDaemon {
    context: Moonbase,
}

impl ExtractFrom<Moonbase> for AxumServerDaemon {
    async fn extract_from(moonbase: &Moonbase) -> Self {
        AxumServerDaemon {
            context: moonbase.clone(),
        }
    }
}

impl TryExtractFrom<Moonbase> for AxumServerDaemon {
    type Error = Infallible;

    async fn try_extract_from(moonbase: &Moonbase) -> Result<Self, Self::Error> {
        Ok(AxumServerDaemon {
            context: moonbase.clone(),
        })
    }
}

pub struct AxumServerState {
    moonbase: Moonbase,
}

impl IntoFuture for AxumServerDaemon {
    type Output = Self;
    type IntoFuture = Pin<Box<dyn Future<Output = Self> + Send>>;

    fn into_future(self) -> Pin<Box<dyn Future<Output = Self> + Send>> {
        Box::pin(async {
            let router = self.context.collect_axum_routes();
            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, router.with_state(self.context.clone())).await;
            self
        })
    }
}

impl Daemon<Moonbase> for AxumServerDaemon {}
