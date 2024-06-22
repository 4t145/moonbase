
#[cfg(feature = "rt-tokio")]
mod tokio;
#[cfg(feature = "rt-tokio")]
pub use tokio::Tokio;

#[cfg(feature = "rt-tokio")]
pub type DefaultRuntime = Tokio;
pub trait Runtime {
    fn spawn<F>(&self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static;
}


