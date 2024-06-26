use std::{any};

use crate::{
    context::Context,
    handler::{Adapter, Handler},
};

pub trait Module<C: Context>: Send + 'static {
    fn module_name() -> &'static str {
        any::type_name::<Self>()
    }
    fn initialize(self, context: C)
        -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}

pub struct ModuleAdapter<M: Module<C>, C: Context> {
    marker: std::marker::PhantomData<fn(C) -> M>,
}

impl<M: Module<C>, C: Context> Adapter for ModuleAdapter<M, C> {
    type Args = C;
    type Ret = anyhow::Result<()>;
}

impl<M, C> Handler<ModuleAdapter<M, C>> for M
where
    M: Module<C> + Send,
    C: Context,
{
    async fn apply(self, context: C) -> anyhow::Result<()> {
        let fut = self.initialize(context);
        fut.await
    }
}
