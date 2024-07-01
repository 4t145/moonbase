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
    TypeTag,
    Custom,
}

impl std::fmt::Display for ComponentDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentDomain::DaemonHandle => write!(f, "Daemon"),
            ComponentDomain::Custom => write!(f, "Custom"),
            ComponentDomain::TypeTag => write!(f, "TypeTag"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentName<T: Any> {
    /// domain of the component
    domain: ComponentDomain,
    bytes: Cow<'static, [u8]>,
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
    fn hash_fields(bytes: &[u8], type_id: TypeId, domain: ComponentDomain) -> u64 {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        type_id.hash(&mut hasher);
        domain.hash(&mut hasher);
        hasher.finish()
    }
    /// create a new ComponentName
    pub(crate) fn new_with_domain(
        name: impl Into<Cow<'static, str>>,
        bytes: impl Into<Cow<'static, [u8]>>,
        domain: ComponentDomain,
    ) -> Self {
        let name = name.into();
        let bytes = bytes.into();
        let type_id = TypeId::of::<T>();
        let hash = Self::hash_fields(&bytes, type_id, domain);
        ComponentName {
            readable_name: name,
            hash,
            domain,
            type_id: TypeId::of::<T>(),
            _marker: std::marker::PhantomData,
            bytes,
        }
    }
    pub fn new_daemon_handle<D, C>() -> Self
    where
        C: Context,
        D: Daemon<C>,
    {
        let bytes = crate::utils::hash(&TypeId::of::<D>());
        let name = std::any::type_name::<D>();
        Self::new_with_domain(
            name,
            bytes.to_be_bytes().to_vec(),
            ComponentDomain::DaemonHandle,
        )
    }
    pub fn new(id: impl Into<Cow<'static, str>>) -> Self {
        let name = id.into();
        let bytes = name.as_bytes().to_vec();
        Self::new_with_domain(name, bytes, ComponentDomain::Custom)
    }
    pub fn new_type_tag<Tag: Any>() -> Self {
        let type_id = TypeId::of::<Tag>();
        let name = std::any::type_name::<Tag>();
        let hashed = crate::utils::hash(&type_id);
        Self::new_with_domain(
            name,
            hashed.to_be_bytes().to_vec(),
            ComponentDomain::TypeTag,
        )
    }
    /// get the readable name
    pub fn readable_name(&self) -> &str {
        &self.readable_name
    }
    /// get the hash value of the readable name
    pub fn hash(&self) -> u64 {
        self.hash
    }
    /// reset the readable name
    pub fn reset_name(&mut self, name: impl Into<Cow<'static, str>>) {
        self.readable_name = name.into();
    }
}

impl<T: Any> Hash for ComponentName<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash)
    }
}
