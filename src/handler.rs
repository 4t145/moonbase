//! Handlers allow us to use **Implicit Parameters**
//!
//! Imaging we have a http client  

use std::{future::Future, marker::PhantomData};

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
    ($($T:ident)*) => {
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
    };
}
crate::tuples!(
    impl_handler!
    T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15
);
