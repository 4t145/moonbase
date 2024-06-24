use std::marker::PhantomData;

use crate::{
    extract::ExtractFrom,
    handler::{FallibleFn, Handler},
    module::Module,
};

pub trait Context: Sized + Send + Sync + 'static {
    fn call<Args, Ret, F>(&self, handler: F) -> impl std::future::Future<Output = Ret> + Send
    where
        F: Handler<Self, Args, Ret> + Send,
    {
        handler.apply(self)
    }
    fn extract<T>(&self) -> impl std::future::Future<Output = T> + Send
    where
        T: ExtractFrom<Self> + Send,
    {
        T::extract_from(self)
    }
}

pub trait ContextExt: Context {
    /// Call a fallible function with the context.
    ///
    /// This is a convenience method that calls the [`Context::call`] method on the handler,
    /// with a [`FallibleFn`] adapter.
    fn fallible_call<Args, Ret, Error, F>(
        &self,
        handler: F,
    ) -> impl std::future::Future<Output = Result<Ret, Error>> + Send
    where
        F: Handler<Self, FallibleFn<Args, Error>, Result<Ret, Error>> + Send,
    {
        self.call(handler)
    }

    /// Call an infallible function with the context.
    ///
    /// This is a convenience method that calls the [`Context::call`] method on the handler,
    /// with an [`InfallibleFn`] adapter.
    fn infallible_call<Args, Ret, F>(
        &self,
        handler: F,
    ) -> impl std::future::Future<Output = Ret> + Send
    where
        F: Handler<Self, FallibleFn<Args, ()>, Ret> + Send,
    {
        self.call(handler)
    }

    /// Load a module into the context.
    ///
    /// This is a convenience method that calls the [`Context::call`] method on the module,
    /// with a [`ModuleHandler`](`crate::module::ModuleHandler`) adapter,
    /// in witch [`Module::initialize`] is called.
    fn load_module<M>(&self, module: M) -> impl std::future::Future<Output = anyhow::Result<()>>
    where
        M: Module<Self> + Send,
    {
        self.call(module)
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

impl<T, C, S> ExtractFrom<S> for FromContext<T, C>
where
    S: Context,
    C: Context,
    S: AsRef<C>,
    T: ExtractFrom<C>,
{
    async fn extract_from(context: &S) -> Self {
        FromContext::new(T::extract_from(context.as_ref()).await)
    }
}

pub type FromMoonbase<T> = FromContext<T, crate::Moonbase>;
