use std::sync::{atomic::AtomicUsize, Arc};

use crate::runtime::Runtime;
use components::{ComponentName, ComponentRepository, MoonbaseComponent};
use context::Context;
use crossbeam::{atomic::AtomicCell, sync::ShardedLock};
use daemon::{Daemon, DaemonHandle, DaemonStatus};
use extract::Extract;
use futures::FutureExt;
use resource::MoonbaseResource;
use utils::AnyMap;

pub mod components;
pub mod context;
pub mod daemon;
pub mod extension;
pub mod extract;
pub mod handler;
pub mod module;
pub mod net;
pub mod resource;
pub mod runtime;
pub mod utils;

#[derive(Debug, Clone, Default)]
pub struct Moonbase {
    id: u64,
    resources: Arc<ShardedLock<AnyMap>>,
    components: ComponentRepository,
}

pub type AppContext = Moonbase;

impl Moonbase {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn set_resource<T: MoonbaseResource>(&self, resource: T) {
        let mut resources = self.resources.write().unwrap();
        resources.insert(resource);
    }
    pub fn get_resource<T: MoonbaseResource>(&self) -> Option<Arc<T>> {
        let resources = self.resources.read().unwrap();
        resources.get::<T>()
    }
    pub fn remove_resource<T: MoonbaseResource>(&self) -> Option<Arc<T>> {
        let mut resources = self.resources.write().unwrap();
        resources.remove::<T>()
    }
    pub fn set_component<T: MoonbaseComponent>(
        &self,
        name: &components::ComponentName<T>,
        component: T,
    ) {
        let mut components = self.components.write().unwrap();
        components.insert(name, component);
    }
    pub fn get_component<T: MoonbaseComponent>(
        &self,
        name: &components::ComponentName<T>,
    ) -> Option<T> {
        let components = self.components.read().unwrap();
        components.get(name)
    }
    pub fn remove_component<T: MoonbaseComponent>(
        &self,
        name: &components::ComponentName<T>,
    ) -> Option<T> {
        let mut components = self.components.write().unwrap();
        components.remove(name)
    }
    pub fn has_component<T: MoonbaseComponent>(&self, name: &components::ComponentName<T>) -> bool {
        let components = self.components.read().unwrap();
        components.get(name).is_some()
    }
    pub async fn run_daemon<D>(&self) -> anyhow::Result<DaemonHandle>
    where
        D: Daemon<Self>,
        D::IntoFuture: Send + 'static,
    {
        // prepare the daemon
        let descriptor = D::descriptor();
        let handler_name = ComponentName::new(descriptor.name.clone());
        // fetch prev handle
        if let Some(prev_handle) = self.get_component::<DaemonHandle>(&handler_name) {
            anyhow::ensure!(
                !prev_handle.is_guarded,
                "daemon {} is still running",
                descriptor.name
            );
            prev_handle.kill_guard_and_wait().await;
            self.remove_component(&handler_name);
        }
        let daemon = <anyhow::Result<D>>::extract(self).await?;
        let max_restart_time = daemon.max_restart_time();
        let cool_down_time = daemon.cool_down_time();
        let (finish_tx, finish_rx) = futures::channel::oneshot::channel::<()>();
        let (kill_tx, mut kill_rx) = futures::channel::oneshot::channel::<()>();
        let runtime = self
            .get_resource::<crate::runtime::DefaultRuntime>()
            .expect("no runtime found");
        let status = Arc::new(AtomicCell::new(DaemonStatus::Starting));
        let restart_time = Arc::new(AtomicUsize::new(0));
        let handle = DaemonHandle {
            kill: Arc::new(std::sync::Mutex::new(Some(kill_tx))),
            finish: Arc::new(std::sync::Mutex::new(Some(finish_rx))),
            state: status.clone(),
            is_guarded: max_restart_time != Some(0),
            restarted: restart_time.clone(),
        };
        crate::runtime::Runtime::spawn(runtime.clone().as_ref(), async move {
            let status = status.clone();
            let restart_time = restart_time.clone();
            let mut daemon = daemon;
            loop {
                let current_time = restart_time.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if let Some(max_restart_time) = max_restart_time {
                    if current_time > max_restart_time {
                        status.store(DaemonStatus::Terminated);
                        break;
                    }
                }
                status.store(DaemonStatus::Running);
                futures::select! {
                    next_daemon = daemon.into_future().fuse() => {
                        daemon = next_daemon;
                        status.store(DaemonStatus::Starting);
                        if let Some(cool_down_time) = cool_down_time {
                            let _ = runtime.sleep(cool_down_time).await;
                        }
                    }
                    _ = kill_rx => {
                        status.store(DaemonStatus::Terminated);
                        break;
                    }
                };
            }
            let _ = finish_tx.send(());
        });
        self.set_component(&handler_name, handle.clone());
        Ok(handle)
    }
    pub fn get_daemon_handle<D>(&self) -> Option<DaemonHandle>
    where
        D: Daemon<Self>,
    {
        let descriptor = D::descriptor();
        let handler_name = ComponentName::new(descriptor.name.clone());
        self.get_component(&handler_name)
    }
    pub fn new() -> Self {
        Moonbase {
            id: 0,
            resources: Arc::new(ShardedLock::new(AnyMap::new())),
            components: ComponentRepository::default(),
        }
    }
}

impl Extract<Moonbase> for Moonbase {
    async fn extract(context: &Moonbase) -> Self {
        context.clone()
    }
}
impl Context for Moonbase {}
