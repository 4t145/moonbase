//! Handlers allow us to use **Implicit Parameters**
//!
//! Imaging we have a http client  

use crate::context::Context;
use crate::extract::ExtractFrom;
use std::{future::Future, marker::PhantomData, pin::Pin};

pub trait Handler<Context, Adapter, Ret> {
    fn apply(self, context: &Context) -> impl Future<Output = Ret> + Send;
}

/// Reduced and boxed handler
pub type ReducedHandler<Context, Ret> =
    Box<dyn Fn(&Context) -> Pin<Box<dyn Future<Output = Ret> + Send + 'static>>>;

pub trait HandlerExt<Context, Adapter, Ret>: Handler<Context, Adapter, Ret>
where
    Adapter: 'static,
    Ret: 'static,
    Self: 'static,
{
    fn reduce(self) -> ReducedHandler<Context, Ret>
    where
        Self: Clone + Sized + Send,
        Context: Clone + Send + 'static,
    {
        Box::new(move |context| {
            let context = context.clone();
            let this = self.clone();
            Box::pin(async move { this.apply(&context).await })
        })
    }
}

pub struct Proxy<C, A, R, H, P>
where
    H: Handler<C, A, R>,
    P: Fn(&C, &H),
{
    handler: H,
    proxy: P,
    #[allow(clippy::type_complexity)]
    marker: PhantomData<(fn(&C) -> R, A)>,
}

impl<C, A, R, H, P> Proxy<C, A, R, H, P>
where
    H: Handler<C, A, R>,
    P: Fn(&C, &H),
{
    pub fn new(handler: H, proxy: P) -> Self {
        Self {
            handler,
            proxy,
            marker: Default::default(),
        }
    }
}

impl<C, A, R, H, P> Handler<C, A, R> for Proxy<C, A, R, H, P>
where
    H: Handler<C, A, R>,
    P: Fn(&C, &H),
{
    fn apply(self, context: &C) -> impl Future<Output = R> + Send {
        (self.proxy)(context, &self.handler);
        self.handler.apply(context)
    }
}

pub struct FallibleFn<T, E> {
    marker: PhantomData<fn() -> (T, E)>,
}

pub struct InfallibleFn<T> {
    marker: PhantomData<fn() -> T>,
}

macro_rules! impl_handler {
    ($($T:ident),*) => {
        impl<F, C, $($T,)* Ret, Fut> Handler<C, InfallibleFn<($($T,)*)>, Ret> for F
        where
            $($T: ExtractFrom<C> + Send,)*
            C: Context,
            Fut: Future<Output = Ret> + Send + 'static,
            Self: Fn($($T,)*) -> Fut + Send,
        {
            #[allow(unused_variables, non_snake_case)]
            async fn apply(self, context: &C) -> Ret {
                $(
                    let $T = $T::extract_from(context).await;
                )*
                self($($T,)*).await
            }
        }
        impl<F, C, $($T,)* Ret, Fut, Error> Handler<C, FallibleFn<($($T,)*), Error>, Result<Ret, Error>> for F
        where
            $(Result<$T, Error>: ExtractFrom<C> + Send,)*
            $($T: Send,)*
            C: Context,
            Fut: Future<Output = Result<Ret, Error>> + Send + 'static,
            F: Fn($($T,)*) -> Fut + Send,
        {
            #[allow(unused_variables, non_snake_case)]
            async fn apply(self, context: &C) -> Result<Ret, Error> {
                $(
                    let $T = <Result<$T, Error>>::extract_from(context).await?;
                )*
                (self)($($T,)*).await
            }
        }
    };
}

impl_handler!();
impl_handler!(T0);
impl_handler!(T0, T1);
impl_handler!(T0, T1, T2);
impl_handler!(T0, T1, T2, T3);
impl_handler!(T0, T1, T2, T3, T4);
impl_handler!(T0, T1, T2, T3, T4, T5);
impl_handler!(T0, T1, T2, T3, T4, T5, T6);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
