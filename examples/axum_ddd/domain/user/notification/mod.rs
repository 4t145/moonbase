use futures::Future;

pub mod local_signal;
pub trait UserNotification {
    fn notify_user_created(&self, user_id: i32) -> impl Future<Output = Result<(), anyhow::Error>>;
}
