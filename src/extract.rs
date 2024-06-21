use std::future::Future;

use crate::context::Context;
pub trait Extract<C>: Sized
where
    C: Context,
{
    fn extract(context: &C) -> impl Future<Output = Self> + Send;
}

impl<C, T, E> Extract<C> for Result<T, E>
where
    T: Extract<C>,
    C: Context,
{
    async fn extract(context: &C) -> Self {
        Ok(T::extract(context).await)
    }
}

// we may impl this in the future
// pub struct Parallelized<T>(T);

macro_rules! impl_tuples {
    ($($T:ident,)*) => {
        impl<C, $($T,)*> Extract<C> for ($($T,)*)
        where
            $($T: Extract<C> + Send,)*
            C: Context,
        {
            #[allow(clippy::unused_unit, )]
            async fn extract(_context: &C) -> Self {
               ( $($T::extract(_context).await, )*)
            }
        }
    };
}

impl_tuples!();
impl_tuples!(T0,);
impl_tuples!(T0, T1,);
impl_tuples!(T0, T1, T2,);
impl_tuples!(T0, T1, T2, T3,);
impl_tuples!(T0, T1, T2, T3, T4,);
impl_tuples!(T0, T1, T2, T3, T4, T5,);
impl_tuples!(T0, T1, T2, T3, T4, T5, T6,);
impl_tuples!(T0, T1, T2, T3, T4, T5, T6, T7,);