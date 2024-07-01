use crate::context::Context;
use std::future::Future;
pub trait ExtractFrom<C>: Sized
where
    C: Context,
{
    fn extract_from(context: &C) -> impl Future<Output = Self> + Send;
}

impl<C, T> ExtractFrom<C> for Result<T, T::Error>
where
    T: TryExtractFrom<C>,
    C: Context,
{
    async fn extract_from(context: &C) -> Self {
        T::try_extract_from(context).await
    }
}

pub trait TryExtractFrom<C>: Sized
where
    C: Context,
{
    type Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static;
    fn try_extract_from(context: &C) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}


// we may impl this in the future
// pub struct Parallelized<T>(T);

macro_rules! extract_tuples {
    ($($T:ident)*) => {
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
        impl<C, $($T,)* > TryExtractFrom<C> for ($($T,)*)
        where
            $(
                $T: TryExtractFrom<C> + Send,
                $T::Error: std::error::Error + Send + Sync + 'static,
            )*
            C: Context,
        {
            type Error = anyhow::Error;
            #[allow(clippy::unused_unit, )]
            async fn try_extract_from(_context: &C) -> Result<Self, anyhow::Error> {
               Ok(( $($T::try_extract_from(_context).await?, )*))
            }
        }
    };
}

crate::tuples!(
    extract_tuples!
    T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15
);
