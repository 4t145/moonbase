use std::marker::PhantomData;

use futures::Future;

use crate::{
    extract::ExtractFrom,
    handler::{Adapter, Call, Handler},
    module::Module,
};

pub trait Context: Sized + Send + Sync + 'static {
    fn call_handler<A, H>(&self, handler: H) -> impl Future<Output = A::Ret> + Send
    where
        H: Handler<A> + Send,
        A: Adapter,
        A::Args: ExtractFrom<Self>,
    {
        async move { handler.apply(self.extract().await).await }
    }
    fn extract<T>(&self) -> impl Future<Output = T> + Send
    where
        T: ExtractFrom<Self>,
    {
        T::extract_from(self)
    }
}

pub trait ContextExt: Context {
    /// Call a fallible function with the context.
    ///
    /// This is a convenience method that calls the [`Context::call_handler`] method on the handler,
    /// with a [`Call`] adapter.
    fn call<T, R, H>(&self, handler: H) -> impl Future<Output = R::Output>
    where
        H: Handler<Call<T, R>> + Send,
        T: ExtractFrom<Self>,
        R: Future,
    {
        self.call_handler(handler)
    }

    // /// Call an infallible function with the context.
    // ///
    // /// This is a convenience method that calls the [`Context::call`] method on the handler,
    // /// with an [`InfallibleFn`] adapter.
    // fn infallible_call<Args, Ret, F>(
    //     &self,
    //     handler: F,
    // ) -> impl std::future::Future<Output = Ret> + Send
    // where
    //     F: Handler<Self, Fallible<Args, ()>, Ret> + Send,
    // {
    //     self.call(handler)
    // }

    /// Load a module into the context.

    /// This is a convenience method that calls the [`Context::call`] method on the module,
    /// with a [`ModuleAdapter`](`crate::module::ModuleAdapter`) adapter,
    /// in witch [`Module::initialize`] is called.
    fn load_module<M>(&self, module: M) -> impl Future<Output = anyhow::Result<()>>
    where
        M: Module<Self> + Send,
        Self: ExtractFrom<Self>,
    {
        self.call_handler::<crate::module::ModuleAdapter<M, Self>, M>(module)
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
