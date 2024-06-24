use std::{
    any::{Any, TypeId},
    borrow::Cow,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{context::Context, daemon::Daemon};
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum ComponentDomain {
    DaemonHandle,
    Custom,
}

impl std::fmt::Display for ComponentDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentDomain::DaemonHandle => write!(f, "daemon"),
            ComponentDomain::Custom => write!(f, "custom"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentName<T: Any> {
    /// domain of the component
    domain: ComponentDomain,
    /// readable name of the component
    readable_name: Cow<'static, str>,
    /// hash cache of the readable name
    type_id: TypeId,
    /// hash cache
    hash: u64,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T: Any> std::fmt::Display for ComponentName<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}::{}::[{}]",
            self.domain,
            self.readable_name,
            std::any::type_name::<T>()
        )
    }
}

impl<T: Any> ComponentName<T> {
    fn hash_fields(id: &str, type_id: TypeId, domain: ComponentDomain) -> u64 {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        type_id.hash(&mut hasher);
        domain.hash(&mut hasher);
        hasher.finish()
    }
    /// create a new ComponentName
    pub(crate) fn new_with_domain(
        id: impl Into<Cow<'static, str>>,
        domain: ComponentDomain,
    ) -> Self {
        let id = id.into();
        let type_id = TypeId::of::<T>();
        let hash = Self::hash_fields(&id, type_id, domain);
        ComponentName {
            readable_name: id,
            hash,
            domain,
            type_id: TypeId::of::<T>(),
            _marker: std::marker::PhantomData,
        }
    }
    pub fn new_daemon_handle<D, C>() -> Self
    where
        C: Context,
        D: Daemon<C>,
    {
        let id = D::descriptor().name;
        Self::new_with_domain(id, ComponentDomain::DaemonHandle)
    }
    pub fn new(id: impl Into<Cow<'static, str>>) -> Self {
        Self::new_with_domain(id, ComponentDomain::Custom)
    }
    /// get the readable name
    pub fn readable_name(&self) -> &str {
        &self.readable_name
    }
    /// get the hash value of the readable name
    pub fn hash(&self) -> u64 {
        self.hash
    }
    /// reset the readable name, and the hash value will be recalculated
    pub fn reset(&mut self, id: impl Into<Cow<'static, str>>) {
        let id = id.into();
        self.hash = Self::hash_fields(&id, self.type_id, self.domain);
        self.readable_name = id;
    }
}

impl<T: Any> Hash for ComponentName<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash)
    }
}
