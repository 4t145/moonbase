use futures::Future;

use super::entity::User;
pub mod surreal;
pub trait UserRepository {
    fn create(&self, user: User) -> impl Future<Output = Result<(), anyhow::Error>>;
}
