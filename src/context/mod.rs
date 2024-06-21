use std::marker::PhantomData;

use crate::{extract::Extract, Moonbase};

pub trait Context: Send + Sync + 'static {}

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

impl Context for Moonbase {}
