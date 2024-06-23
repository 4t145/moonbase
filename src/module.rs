use std::any;

use crate::{context::Context, handler::Handler};

pub trait Module<C: Context> {
    fn module_name() -> &'static str {
        any::type_name::<Self>()
    }
    fn initialize(
        self,
        context: &C,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}

pub struct ModuleHandler<M: Module<C>, C: Context> {
    marker: std::marker::PhantomData<fn(C) -> M>,
}

impl<M, C> Handler<C, ModuleHandler<M, C>, anyhow::Result<()>> for M
where
    M: Module<C> + Send,
    C: Context,
{
    async fn apply(self, context: &C) -> anyhow::Result<()> {
        self.initialize(context).await
    }
}
