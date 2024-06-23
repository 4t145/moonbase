use std::{
    borrow::Cow,
    future::IntoFuture,
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use crate::{components::MoonbaseComponent, context::Context, extract::Extract};

pub struct DaemonDescriptor {
    pub name: Cow<'static, str>,
}

pub trait Daemon<C>: IntoFuture<Output = Self> + Extract<C> + Send + Sync + 'static
where
    C: Context,
{
    fn descriptor() -> DaemonDescriptor {
        DaemonDescriptor {
            name: std::any::type_name::<Self>().into(),
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
