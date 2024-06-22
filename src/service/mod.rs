use crate::{components::MoonbaseComponent, extract::Extract, Moonbase};
use std::{any::Any, borrow::Cow, collections::HashMap, future::Future, sync::Arc};
use tower::Service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ServiceStatusKind {
    #[default]
    Ready,
    Starting,
    Running,
    Stopping,
    Stopped,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ServicePolicy {
    #[default]
    OneShot,
    Guarded,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ServiceDescriptor {
    pub name: Cow<'static, str>,
}

impl ServiceDescriptor {
    pub fn from_type<T: Any>() -> Self {
        ServiceDescriptor {
            name: std::any::type_name::<T>().into(),
        }
    }
    pub fn from_static(name: &'static str) -> Self {
        ServiceDescriptor { name: name.into() }
    }
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        ServiceDescriptor { name: name.into() }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceHandle {
    pub(crate) kill: Arc<std::sync::Mutex<Option<futures::channel::oneshot::Sender<()>>>>,
    pub(crate) finish: Arc<std::sync::Mutex<Option<futures::channel::oneshot::Receiver<()>>>>,
    pub(crate) is_guarded: bool,
}

impl ServiceHandle {
    pub fn is_guarded(&self) -> bool {
        self.is_guarded
    }
    pub fn kill_guard(&self) {
        let mut kill = self.kill.lock().unwrap();
        if let Some(kill) = kill.take() {
            let _ = kill.send(());
        }
    }
    pub async fn wait(&self) {
        let finish = self.finish.lock().unwrap().take();
        if let Some(finish) = finish {
            let _ = finish.await;
        }
    }
    pub async fn kill_guard_and_wait(&self) {
        self.kill_guard();
        self.wait().await;
    }
}

impl Drop for ServiceHandle {
    fn drop(&mut self) {
        if Arc::strong_count(&self.kill) == 1 {
            self.kill_guard();
        }
    }
}

impl MoonbaseComponent for ServiceHandle {}
pub trait MoonbaseService
where
    Self: Service<Self::Request> + Sized + Any + Send + Sync,
{
    type Request;
    fn handle_error(&self, error: <Self as Service<Self::Request>>::Error) {
        drop(error);
    }
    fn descriptor() -> ServiceDescriptor {
        ServiceDescriptor::from_type::<Self>()
    }
    fn policy(&self) -> ServicePolicy;
}

pub struct MoonbaseServiceRunner<Service, Stream, Signal> {
    pub(crate) service: Service,
    pub(crate) stream: Stream,
    pub(crate) signal: Signal,
}

impl<Service, Stream, Signal> MoonbaseServiceRunner<Service, Stream, Signal> {
    pub fn new(service: Service, stream: Stream, signal: Signal) -> Self {
        MoonbaseServiceRunner {
            service,
            stream,
            signal,
        }
    }
}

impl<Service, Stream, Signal> MoonbaseServiceRunner<Service, Stream, Signal>
where
    Service: MoonbaseService,
    Stream: futures::Stream<Item = Service::Request> + Unpin + Send,
    Signal: Send + Clone + Future<Output = ()>,
    <Service as tower::Service<<Stream as futures::Stream>::Item>>::Future: std::marker::Send,
{
    pub fn is_guarded(&self) -> bool {
        self.service.policy() == ServicePolicy::Guarded
    }
    pub async fn run(mut self) -> Self {
        use futures::FutureExt;
        use futures::StreamExt;
        let mut stream = self.stream.fuse();
        loop {
            let req = futures::select! {
                req = stream.next() => {
                    req
                },
                _ = self.signal.clone().fuse() => {
                    break;
                },
            };
            let Some(req) = req else {
                break;
            };
            let res = self.service.call(req).await;
            if let Err(error) = res {
                self.service.handle_error(error);
            }
        }
        Self {
            service: self.service,
            signal: self.signal,
            stream: stream.into_inner(),
        }
    }
}

impl<Service, Stream, Signal> Extract<Moonbase>
    for anyhow::Result<MoonbaseServiceRunner<Service, Stream, Signal>>
where
    anyhow::Result<Service>: Send + Extract<Moonbase>,
    anyhow::Result<Signal>: Send + Extract<Moonbase>,
    anyhow::Result<Stream>: Send + Extract<Moonbase>,
{
    async fn extract(context: &Moonbase) -> Self {
        let (service, stream, signal) = futures::try_join!(
            <anyhow::Result<Service>>::extract(context),
            <anyhow::Result<Stream>>::extract(context),
            <anyhow::Result<Signal>>::extract(context),
        )?;

        Ok(MoonbaseServiceRunner::new(service, stream, signal))
    }
}

pub struct ServiceRepository {
    services: HashMap<ServiceDescriptor, Arc<dyn Any + Send + Sync + 'static>>,
}

impl Default for ServiceRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceRepository {
    pub fn new() -> Self {
        ServiceRepository {
            services: HashMap::new(),
        }
    }

    pub fn insert<T, R>(&mut self, service: T)
    where
        T: MoonbaseService + 'static,
        R: Send + 'static,
    {
        let descriptor = T::descriptor();
        let service = Arc::new(service);
        self.services.insert(descriptor, service);
    }
}
