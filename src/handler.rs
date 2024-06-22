use crate::context::Context;
use crate::extract::Extract;
use std::{future::Future, marker::PhantomData, pin::Pin};

pub trait Handler<Context, Args, Ret> {
    fn apply(self, context: &Context) -> impl Future<Output = Ret> + Send;
}

pub type ReducedHandler<Context, Ret> =
    Box<dyn Fn(&Context) -> Pin<Box<dyn Future<Output = Ret> + Send + 'static>>>;
pub trait HandlerExt<Context, Args, Ret>: Handler<Context, Args, Ret>
where
    Args: 'static,
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

pub struct Fallible<T, E> {
    marker: PhantomData<(T, E)>,
}

pub struct Infallible<T> {
    marker: PhantomData<T>,
}

macro_rules! impl_handler {
    ($($T:ident),*) => {
        impl<F, C, $($T,)* Ret, Fut> Handler<C, Infallible<($($T,)*)>, Ret> for F
        where
            $($T: Extract<C> + Send,)*
            C: Context,
            Fut: Future<Output = Ret> + Send + 'static,
            Self: Fn($($T,)*) -> Fut + Send,
        {
            #[allow(unused_variables, non_snake_case)]
            async fn apply(self, context: &C) -> Ret {
                $(
                    let $T = $T::extract(context).await;
                )*
                self($($T,)*).await
            }
        }
        impl<F, C, $($T,)* Ret, Fut, Error> Handler<C, Fallible<($($T,)*), Error>, Result<Ret, Error>> for F
        where
            $(Result<$T, Error>: Extract<C> + Send,)*
            $($T: Send,)*
            C: Context,
            Fut: Future<Output = Result<Ret, Error>> + Send + 'static,
            F: Fn($($T,)*) -> Fut + Send,
        {
            #[allow(unused_variables, non_snake_case)]
            async fn apply(self, context: &C) -> Result<Ret, Error> {
                $(
                    let $T = <Result<$T, Error>>::extract(context).await?;
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
