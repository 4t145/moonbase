use moonbase::{context::ContextExt, handler::Fallible, Moonbase};

fn main() {}

async fn async_main() {
    let moonbase = Moonbase::new();
    let init_result = moonbase.call(init_resource_infallible).await;
    let init_result = moonbase.fallible_call(init_resource).await;
}

async fn init_resource() -> anyhow::Result<()> {
    Ok(())
}

async fn init_resource_infallible() {}
