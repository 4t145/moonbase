use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use components::{ComponentName, ComponentRepository, MoonbaseComponent};
use context::Context;
use crossbeam::sync::ShardedLock;
use extract::Extract;
use futures::{Future, FutureExt};
use resource::MoonbaseResource;
use service::{MoonbaseService, MoonbaseServiceRunner, ServiceHandle};
pub mod cluster;
pub mod components;
pub mod context;
pub mod extract;
pub mod handler;
pub mod resource;
pub mod runtime;
pub mod service;
pub mod utils;
pub mod module;
#[derive(Debug, Clone, Default)]
pub struct Moonbase {
    id: u64,
    resources: Arc<ShardedLock<HashMap<TypeId, Arc<dyn Send + Sync + Any>>>>,
    components: ComponentRepository,
}

pub type AppContext = Moonbase;

impl Moonbase {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn set_resource<T: MoonbaseResource>(&self, resource: T) {
        let mut resources = self.resources.write().unwrap();
        resources.insert(TypeId::of::<T>(), Arc::new(resource));
    }
    pub fn get_resource<T: MoonbaseResource>(&self) -> Option<Arc<T>> {
        let resources = self.resources.read().unwrap();
        resources
            .get(&TypeId::of::<T>())
            .cloned()
            .map(|resource| Arc::downcast(resource).expect("fail to downcast resource"))
    }
    pub fn remove_resource<T: MoonbaseResource>(&self) -> Option<Arc<T>> {
        let mut resources = self.resources.write().unwrap();
        resources
            .remove(&TypeId::of::<T>())
            .map(|resource| Arc::downcast(resource).expect("fail to downcast resource"))
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
    pub async fn run_service<Service, Stream, Signal>(&self) -> anyhow::Result<()>
    where
        anyhow::Result<MoonbaseServiceRunner<Service, Stream, Signal>>:
            Extract<Moonbase> + Send + 'static,
        Service: MoonbaseService,
        Stream: futures::Stream<Item = Service::Request> + Unpin + Send,
        Signal: Send + Clone + Future<Output = ()> + Send,
        <Service as tower::Service<<Stream as futures::Stream>::Item>>::Future: Send,
        <Stream as futures::Stream>::Item: std::marker::Send,
    {
        let descriptor = Service::descriptor();
        let handler_name = ComponentName::new(descriptor.name.clone());
        // fetch prev handle
        if let Some(prev_handle) = self.get_component::<ServiceHandle>(&handler_name) {
            anyhow::ensure!(
                !prev_handle.is_guarded,
                "service {} is still running",
                descriptor.name
            );
            prev_handle.kill_guard_and_wait().await;
            self.take_service_handle::<Service>();
        }

        // init
        let runner =
            <anyhow::Result<MoonbaseServiceRunner<Service, Stream, Signal>>>::extract(self).await?;
        let is_guarded = runner.is_guarded();
        let (finish_tx, finish_rx) = futures::channel::oneshot::channel::<()>();
        // spawning the task
        let runtime = self
            .get_resource::<crate::runtime::DefaultRuntime>()
            .expect("no runtime found");
        let handle = if is_guarded {
            let (kill_tx, mut kill_rx) = futures::channel::oneshot::channel::<()>();
            crate::runtime::Runtime::spawn(runtime.as_ref(), async move {
                let mut runner = runner;
                loop {
                    futures::select! {
                        next_runner = runner.run().fuse() => {
                            runner = next_runner;
                        }
                        _ = kill_rx => {
                            break;
                        }
                    };
                }
                let _ = finish_tx.send(());
            });
            ServiceHandle {
                kill: Arc::new(std::sync::Mutex::new(Some(kill_tx))),
                finish: Arc::new(std::sync::Mutex::new(Some(finish_rx))),
                is_guarded: true,
            }
        } else {
            crate::runtime::Runtime::spawn(runtime.as_ref(), async move {
                runner.run().await;
                let _ = finish_tx.send(());
            });
            ServiceHandle {
                kill: Arc::new(std::sync::Mutex::new(None)),
                finish: Arc::new(std::sync::Mutex::new(Some(finish_rx))),
                is_guarded: false,
            }
        };
        // set the handle
        self.set_component(&handler_name, handle);
        Ok(())
    }

    /// remove the handle of a service, and return the handle if it exists
    ///
    /// # Notice
    /// once the handle is dropped, the service guard will be killed
    pub fn take_service_handle<Service>(&self) -> Option<ServiceHandle>
    where
        Service: service::MoonbaseService,
    {
        let handler_name = ComponentName::new(Service::descriptor().name);
        self.remove_component(&handler_name)
    }
    pub fn new() -> Self {
        Moonbase {
            id: 0,
            resources: Arc::new(ShardedLock::new(HashMap::new())),
            components: ComponentRepository::default(),
        }
    }
}

impl Extract<Moonbase> for Moonbase {
    async fn extract(context: &Moonbase) -> Self {
        context.clone()
    }
}
impl Context for Moonbase {

}