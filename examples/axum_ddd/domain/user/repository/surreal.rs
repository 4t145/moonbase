use surrealdb::Surreal;

use crate::domain::user::entity::User;

use super::UserRepository;

impl<C> UserRepository for Surreal<C>
where
    C: surrealdb::Connection,
{
    async fn create(&self, user: User) -> Result<(), anyhow::Error> {
        let _: Vec<()> = self.insert("users").content(user).await?;
        Ok(())
    }
}
