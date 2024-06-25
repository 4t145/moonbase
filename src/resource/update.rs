use std::{
    any::TypeId,
    collections::{HashMap, HashSet, VecDeque},
    sync::{atomic::AtomicUsize, Arc, Mutex},
    task::Waker,
};

#[derive(Debug, Clone)]
pub struct ResourceUpdateSubscriber {
    local_version: usize,
    notifier: Arc<ResourceUpdateNotifier>,
}

impl ResourceUpdateSubscriber {
    pub async fn next(&mut self) {
        let mut subscribed = false;
        futures::future::poll_fn(move |cx| {
            if !subscribed {
                self.notifier
                    .waiting_list
                    .lock()
                    .unwrap()
                    .push_back(cx.waker().clone());
                subscribed = true;
            }
            let notifier_version = self
                .notifier
                .version
                .load(std::sync::atomic::Ordering::SeqCst);
            if self.local_version != notifier_version {
                self.local_version = notifier_version;
                futures::task::Poll::Ready(notifier_version)
            } else {
                futures::task::Poll::Pending
            }
        })
        .await;
    }
}

#[derive(Debug)]
pub struct ResourceUpdateNotifier {
    version: AtomicUsize,
    waiting_list: Mutex<VecDeque<Waker>>,
}

impl ResourceUpdateNotifier {
    pub fn notify(&self) {
        self.version
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut swap_out = VecDeque::new();
        {
            let mut waiting_list = self.waiting_list.lock().unwrap();
            std::mem::swap(&mut swap_out, &mut *waiting_list);
        }
        for waker in swap_out.into_iter() {
            waker.wake();
        }
    }
}
// A          ->        B        ->     C    ->       D
// notifier
//
//

#[derive(Debug, Default)]
pub struct EffectMap {
    effect: HashMap<TypeId, HashSet<TypeId>>,
    notifier: HashMap<TypeId, ResourceUpdateNotifier>,
}

impl EffectMap {
    pub fn new() -> Self {
        EffectMap {
            effect: HashMap::new(),
            notifier: HashMap::new(),
        }
    }
    pub fn add(&mut self, id: TypeId, effect: TypeId) {
        let secondary = self.effect.entry(effect).or_default().clone();
        assert!(!secondary.contains(&id), "circular effect detected");
        self.effect.entry(id).or_default().extend(secondary);
    }
    pub fn effected(&self, id: TypeId) -> HashSet<TypeId> {
        self.effect.get(&id).cloned().unwrap_or_default()
    }
    pub fn notify_update(&mut self, id: TypeId) {
        let effected = self.effected(id);
        for effect in effected {
            if let Some(notifier) = self.notifier.get(&effect) {
                notifier.notify();
            }
        }
    }
}
