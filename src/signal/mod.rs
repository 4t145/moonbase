use std::{
    any::{Any, TypeId},
    borrow::Cow,
    sync::{atomic::AtomicBool, Arc},
    task::Waker,
};

use futures::Future;

use crate::Moonbase;
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct SignalKey {
    id: Cow<'static, [u8]>,
}

impl SignalKey {
    pub fn from_type<T: Any>() -> Self {
        Self {
            id: Cow::Owned(
                crate::utils::hash(&TypeId::of::<T>())
                    .to_be_bytes()
                    .to_vec(),
            ),
        }
    }
    pub const fn from_static_str(id: &'static str) -> Self {
        Self {
            id: Cow::Borrowed(id.as_bytes()),
        }
    }
    pub const fn from_static_bytes(id: &'static [u8]) -> Self {
        Self {
            id: Cow::Borrowed(id),
        }
    }
    pub fn new(id: impl AsRef<[u8]>) -> Self {
        Self {
            id: Cow::Owned(id.as_ref().to_vec()),
        }
    }
}

pub struct TypedSignal<T> {
    marker: std::marker::PhantomData<T>,
    signal: Signal,
}

impl<T: std::any::Any> TypedSignal<T> {
    pub fn key() -> SignalKey {
        SignalKey::from_type::<T>()
    }
}

impl<T> Default for TypedSignal<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData,
            signal: Signal::new(),
        }
    }
}

impl<T> Clone for TypedSignal<T> {
    fn clone(&self) -> Self {
        Self {
            marker: std::marker::PhantomData,
            signal: self.signal.clone(),
        }
    }
}

impl<T> std::fmt::Debug for TypedSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedSignal")
            .field("signal", &self.signal)
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Signal {
    inner: Arc<WaitList>,
}

impl Signal {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(WaitList::new()),
        }
    }

    pub fn wait(&self) -> WaitingSignal {
        WaitingSignal {
            inner: self.inner.clone(),
            waiting: false,
            ok: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_trigger(&self) -> SignalTrigger {
        SignalTrigger {
            inner: self.inner.clone(),
        }
    }
}
#[derive(Debug, Default)]
struct WaitList {
    queue: crossbeam::queue::SegQueue<(Waker, Arc<AtomicBool>)>,
}

impl WaitList {
    pub fn new() -> Self {
        Self {
            queue: crossbeam::queue::SegQueue::new(),
        }
    }

    pub fn push(&self, waker: Waker, state: Arc<AtomicBool>) {
        self.queue.push((waker, state));
    }

    pub fn consume(&self) {
        while let Some((waker, state)) = self.queue.pop() {
            state.store(true, std::sync::atomic::Ordering::Relaxed);
            waker.wake();
        }
    }
}

#[derive(Debug, Clone)]
pub struct WaitingSignal {
    inner: Arc<WaitList>,
    ok: Arc<AtomicBool>,
    waiting: bool,
}

impl Future for WaitingSignal {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if !self.as_ref().waiting {
            self.inner.push(cx.waker().clone(), self.ok.clone());
            self.as_mut().waiting = true;
            std::task::Poll::Pending
        } else if self.ok.load(std::sync::atomic::Ordering::Relaxed) {
            std::task::Poll::Ready(())
        } else {
            std::task::Poll::Pending
        }
    }
}
#[derive(Debug, Clone)]
pub struct SignalTrigger {
    inner: Arc<WaitList>,
}

impl SignalTrigger {
    pub fn trigger(&self) {
        self.inner.consume();
    }
}

impl Moonbase {
    pub fn set_signal(&self, key: SignalKey, signal: Signal) {
        let mut signals = self.signals.write().unwrap();
        signals.insert(key, signal);
    }
    pub fn get_signal(&self, key: &SignalKey) -> Option<Signal> {
        let signals = self.signals.read().unwrap();
        signals.get(key).cloned()
    }
    pub fn remove_signal(&self, key: &SignalKey) -> Option<Signal> {
        let mut signals = self.signals.write().unwrap();
        signals.remove(key)
    }
    pub fn has_signal(&self, key: &SignalKey) -> bool {
        let signals = self.signals.read().unwrap();
        signals.contains_key(key)
    }
    pub fn trigger_signal(&self, key: &SignalKey) {
        let signals = self.signals.read().unwrap();
        if let Some(signal) = signals.get(key) {
            signal.get_trigger().trigger();
        }
    }
}
