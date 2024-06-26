//! Handlers allow us to use **Implicit Parameters**
//!
//! Imaging we have a http client  

use std::{
    future::{Future},
    marker::PhantomData,
    pin::{Pin},
};

pub trait Handler<A>
where
    A: Adapter,
{
    fn apply(self, args: A::Args) -> impl Future<Output = A::Ret> + Send;
}

pub trait Adapter {
    type Ret;
    type Args;
}

/// Reduced and boxed handler
pub type ReducedHandler<Context, Ret> =
    Box<dyn Fn(&Context) -> Pin<Box<dyn Future<Output = Ret> + Send + 'static>>>;

// pub trait HandlerExt<C, A>: Handler<C, A>
// where
//     Self: 'static,
//     A: Adapter,
// {
//     fn proxy<P, R>(self, proxy: P) -> Proxy<C, A, Self, P, R>
//     where
//         P: Fn(&C, &Self) -> R,
//         Self: Sized,
//     {
//         Proxy::new(self, proxy)
//     }
// }

// impl<C, A, H> HandlerExt<C, A> for H
// where
//     H: Handler<C, A> + 'static,
//     A: Adapter + 'static,
// {
// }

// pub struct ProxyAdapter<R> {
//     marker: PhantomData<fn() -> R>,
// }

// impl<R> Adapter for ProxyAdapter<R> {
//     type Ret = R;
// }

// pub struct Proxy<C, A, H, P, R>
// where
//     A: Adapter,
//     H: Handler<C, A>,
//     P: Fn(&C, &H) -> R,
// {
//     handler: H,
//     proxy: P,
//     #[allow(clippy::type_complexity)]
//     marker: PhantomData<(fn(&C) -> A::Ret, A)>,
// }

// impl<C, A, H, P, R> Proxy<C, A, H, P, R>
// where
//     A: Adapter,
//     H: Handler<C, A>,
//     P: Fn(&C, &H) -> R,
// {
//     pub fn new(handler: H, proxy: P) -> Self {
//         Self {
//             handler,
//             proxy,
//             marker: Default::default(),
//         }
//     }
// }

// impl<C, A, P, H, R> Handler<C, ProxyAdapter<R>> for Proxy<C, A, H, P, R>
// where
//     A: Adapter,
//     H: Handler<C, A>,
//     P: Fn(&C, &H) -> R,
// {
//     fn apply(self, context: &C) -> R {
//         (self.proxy)(context, &self.handler)
//     }
// }

pub struct Fallible<A, T, E> {
    marker: PhantomData<(A, fn() -> (T, E))>,
}

impl<A, T, E> Adapter for Fallible<A, T, E>
where
    A: Adapter,
{
    type Args = Result<A::Args, E>;
    type Ret = Result<T, E>;
}

impl<H, A, T, E> Handler<Fallible<A, T, E>> for H
where
    A: Adapter<Ret = Result<T, E>>,
    H: Handler<A> + Send,
    <A as Adapter>::Args: std::marker::Send,
    E: Send,
{
    #[allow(unused_variables, non_snake_case)]
    async fn apply(self, args: Result<A::Args, E>) -> Result<T, E> {
        Handler::<A>::apply(self, args?).await
    }
}

pub struct Call<A, Fut> {
    marker: PhantomData<fn(A) -> Fut>,
}

impl<A, Fut> Adapter for Call<A, Fut>
where
    Fut: Future,
{
    type Ret = Fut::Output;
    type Args = A;
}

macro_rules! impl_handler {
    ($($T:ident),*) => {
        impl<F, $($T,)* Fut> Handler<Call<($($T,)*), Fut>> for F
        where
            $($T: Send,)*
            Self: Fn($($T,)*) -> Fut,
            Fut: Future + Send,
            F: Send
        {
            #[allow(unused_variables, non_snake_case)]
            async fn apply(self, args: ($($T,)*)) -> Fut::Output {
                let ($($T,)*) = args;
                self($($T,)*).await
            }
        }
        // impl<F, C, $($T,)* Ret, Fut, Error> Handler<C, FallibleFn<($($T,)*), Error>, Result<Ret, Error>> for F
        // where
        //     $(Result<$T, Error>: ExtractFrom<C> + Send,)*
        //     $($T: Send,)*
        //     C: Context,
        //     Fut: Future<Output = Result<Ret, Error>> + Send + 'static,
        //     F: Fn($($T,)*) -> Fut + Send,
        // {
        //     #[allow(unused_variables, non_snake_case)]
        //     async fn apply(self, context: &C) -> Result<Ret, Error> {
        //         $(
        //             let $T = <Result<$T, Error>>::extract_from(context).await?;
        //         )*
        //         (self)($($T,)*).await
        //     }
        // }
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
