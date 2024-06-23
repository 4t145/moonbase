use std::marker::PhantomData;

use crate::{
    extract::Extract,
    handler::{Fallible, Handler},
    module::Module,
    Moonbase,
};

pub trait Context: Sized + Send + Sync + 'static {}

pub trait ContextExt: Context {
    fn call<Args, Ret, F>(&self, handler: F) -> impl std::future::Future<Output = Ret> + Send
    where
        F: Handler<Self, Args, Ret> + Send,
    {
        handler.apply(self)
    }
    fn fallible_call<Args, Ret, Error, F>(
        &self,
        handler: F,
    ) -> impl std::future::Future<Output = Result<Ret, Error>> + Send
    where
        F: Handler<Self, Fallible<Args, Error>, Result<Ret, Error>> + Send,
    {
        handler.apply(self)
    }
    fn extract<T>(&self) -> impl std::future::Future<Output = T> + Send
    where
        T: Extract<Self> + Send,
    {
        T::extract(self)
    }
    fn load_module<M>(&self, module: M) -> impl std::future::Future<Output = anyhow::Result<()>>
    where
        M: Module<Self>,
    {
        module.initialize(self)
    }
}

impl<T> ContextExt for T where T: Context {}

pub struct FromContext<T, C> {
    pub data: T,
    from: PhantomData<fn(&C)>,
}

impl<T, C> FromContext<T, C> {
    fn new(data: T) -> Self {
        Self {
            data,
            from: Default::default(),
        }
    }
}

impl<T, C, S> Extract<S> for FromContext<T, C>
where
    S: Context,
    C: Context,
    S: AsRef<C>,
    T: Extract<C>,
{
    async fn extract(context: &S) -> Self {
        FromContext::new(T::extract(context.as_ref()).await)
    }
}
