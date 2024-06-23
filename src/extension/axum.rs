use std::path::Component;
use std::pin::Pin;
use std::task::Poll;

pub use axum::serve::serve;
pub use axum::Router;
use futures::Future;
use hyper::body::Incoming;
use hyper::Request;
use tower::util::Oneshot;
use tower::ServiceExt;

use crate::components::{ComponentName, MoonbaseComponent};
use crate::extract::Extract;
use crate::Moonbase;

impl MoonbaseComponent for axum::Router {}

pub struct AxumServer {
}