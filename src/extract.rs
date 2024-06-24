use std::future::Future;

use crate::context::Context;
pub trait ExtractFrom<C>: Sized
where
    C: Context,
{
    fn extract_from(context: &C) -> impl Future<Output = Self> + Send;
}

impl<C, T, E> ExtractFrom<C> for Result<T, E>
where
    T: ExtractFrom<C>,
    C: Context,
{
    async fn extract_from(context: &C) -> Self {
        Ok(T::extract_from(context).await)
    }
}

// we may impl this in the future
// pub struct Parallelized<T>(T);

macro_rules! impl_tuples {
    ($($T:ident,)*) => {
        impl<C, $($T,)*> ExtractFrom<C> for ($($T,)*)
        where
            $($T: ExtractFrom<C> + Send,)*
            C: Context,
        {
            #[allow(clippy::unused_unit, )]
            async fn extract_from(_context: &C) -> Self {
               ( $($T::extract_from(_context).await, )*)
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
