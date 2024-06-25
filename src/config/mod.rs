use futures::Future;

use crate::context::Context;

pub struct ConfigResolver<Cfg, Ctx> {
    hash: u64,
    context: Ctx,
    config: Option<Cfg>,
    raw: Vec<u8>,
}

pub trait Config<C>: Sized
where
    C: Context,
{
    type Dependency: Config<C>;
    fn resolve(context: &C) -> impl Future<Output = anyhow::Result<Self>> + Send;
}

impl<C> Config<C> for ()
where
    C: Context,
{
    type Dependency = ();

    async fn resolve(context: &C) -> anyhow::Result<Self> {
        Ok(())
    }
}


use futures::future::Select;