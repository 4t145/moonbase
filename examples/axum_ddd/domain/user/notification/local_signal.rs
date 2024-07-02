use std::sync::Arc;

use anyhow::Context;
use moonbase::{
    signal::{Signal, SignalSymbol},
    Moonbase,
};
use tokio::sync::mpsc;

use super::UserNotification;

pub enum UserEvent {
    Created(i32),
}
pub struct UserCreatedSymbol;
impl UserNotification for Moonbase {
    async fn notify_user_created(&self, user_id: i32) -> Result<(), anyhow::Error> {
        let sender = self
            .get_resource::<mpsc::Sender<UserEvent>>()
            .with_context(|| {
                format!(
                    "Failed to get signal for user created notification: {}",
                    user_id
                )
            })?;
        sender.send(UserEvent::Created(user_id)).await?;
        Ok(())
    }
}
