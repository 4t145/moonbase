pub use axum::serve::serve;
pub use axum::Router;

use crate::components::MoonbaseComponent;

impl MoonbaseComponent for axum::Router {}

pub struct AxumServer {}
