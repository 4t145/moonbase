use std::{
    any::{Any, TypeId},
    future::IntoFuture,
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use crossbeam::atomic::AtomicCell;
use futures::FutureExt;

use crate::{
    components::{ComponentName, MoonbaseComponent},
    context::Context,
    extract::TryExtractFrom,
    runtime::Runtime,
    Moonbase,
};

pub struct DaemonDescriptor {
    pub type_id: TypeId,
}

pub trait Daemon<C>:
    IntoFuture<Output = Self> + TryExtractFrom<C> + Send + 'static + Any + std::fmt::Debug
where
    C: Context,
{
    fn descriptor() -> DaemonDescriptor {
        DaemonDescriptor {
            type_id: std::any::TypeId::of::<Self>(),
        }
    }
    fn max_restart_time(&self) -> Option<usize> {
        None
    }
    fn cool_down_time(&self) -> Option<Duration> {
        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DaemonStatus {
    Starting,
    Running,
    Terminated,
}

#[derive(Debug, Clone)]
pub struct DaemonHandle {
    pub(crate) kill: Arc<std::sync::Mutex<Option<futures::channel::oneshot::Sender<()>>>>,
    pub(crate) finish: Arc<std::sync::Mutex<Option<futures::channel::oneshot::Receiver<()>>>>,
    pub(crate) state: Arc<crossbeam::atomic::AtomicCell<DaemonStatus>>,
    pub(crate) is_guarded: bool,
    pub(crate) restarted: Arc<AtomicUsize>,
}

impl MoonbaseComponent for DaemonHandle {}

impl DaemonHandle {
    pub fn restarted_times(&self) -> usize {
        self.restarted.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn state(&self) -> DaemonStatus {
        self.state.load()
    }
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

impl Drop for DaemonHandle {
    fn drop(&mut self) {
        if Arc::strong_count(&self.kill) > 1 {
            return;
        }
        if self.state.load() != DaemonStatus::Terminated {
            self.kill_guard();
        }
    }
}

impl Moonbase {
    pub async fn run_daemon<D>(&self) -> anyhow::Result<DaemonHandle>
    where
        D: Daemon<Self>,
        D::IntoFuture: Send + 'static,
        <D as TryExtractFrom<Moonbase>>::Error: std::error::Error,
    {
        // prepare the daemon
        let handler_name = ComponentName::new_daemon_handle::<D, Self>();
        // fetch prev handle
        if let Some(prev_handle) = self.get_component::<DaemonHandle>(&handler_name) {
            anyhow::ensure!(
                !prev_handle.is_guarded,
                "daemon {:?} is still running",
                &self
            );
            prev_handle.kill_guard_and_wait().await;
            self.remove_component(&handler_name);
        }
        let daemon =
            anyhow::Context::context(D::try_extract_from(self).await, "fail to extract daemon")?;
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
        crate::runtime::Runtime::spawn(&runtime.clone(), async move {
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
        let handler_name = ComponentName::new_daemon_handle::<D, Self>();
        self.get_component(&handler_name)
    }
}
