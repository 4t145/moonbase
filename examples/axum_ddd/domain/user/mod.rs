use entity::Permission;
use moonbase::{extract::TryExtractFrom, Moonbase};

pub mod entity;
pub mod notification;
pub mod repository;

pub struct Service<R, N> {
    repository: R,
    notification: N,
    permission: Permission,
    app_context: Moonbase,
}

impl<R, N> Service<R, N> {
    pub fn new(app_context: Moonbase, repository: R, notification: N) -> Self {
        Self {
            repository,
            notification,
            app_context,
            permission: Permission::User,
        }
    }
}

impl<R, N> Service<R, N>
where
    R: repository::UserRepository,
    N: notification::UserNotification,
{
    pub async fn create_user(&self, user: entity::User) -> Result<(), anyhow::Error> {
        let user_id = user.id;
        self.repository.create(user).await?;
        self.notification.notify_user_created(user_id).await?;
        Ok(())
    }
}
