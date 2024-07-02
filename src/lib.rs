use std::{collections::HashMap, sync::Arc};

use components::ComponentRepository;
use context::Context;
use crossbeam::sync::ShardedLock;
use extract::ExtractFrom;
use resource::ResourceRepository;
use signal::{Signal, SignalKey};

pub mod components;
pub mod context;
pub mod daemon;
pub mod extension;
pub mod extract;
pub mod handler;
pub mod module;
pub mod resource;
pub mod runtime;
pub mod signal;
pub mod utils;

pub mod prelude {
    pub use crate::{
        components::*, context::*, daemon::*, extract::*, module::*, resource::*, signal::*,
        AppContext, Moonbase,
    };
}

#[derive(Debug, Clone, Default)]
pub struct Moonbase {
    id: u64,
    resources: ResourceRepository,
    components: ComponentRepository,
    signals: Arc<ShardedLock<HashMap<SignalKey, Signal>>>,
}

pub type AppContext = Moonbase;

impl Moonbase {
    pub fn id(&self) -> u64 {
        self.id
    }
    // pub fn trigger_signal
    pub fn new() -> Self {
        Moonbase {
            id: 0,
            resources: ResourceRepository::default(),
            components: ComponentRepository::default(),
            signals: Arc::new(ShardedLock::new(Default::default())),
        }
    }
}

impl ExtractFrom<Moonbase> for Moonbase {
    async fn extract_from(context: &Moonbase) -> Self {
        context.clone()
    }
}

impl Context for Moonbase {}

