#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndpointId(u64);
pub enum EndpointLocation {
    Local,
    Remote(RemoteEndpoint),
}

pub enum RemoteEndpoint {
    Tcp(String),
    Udp(String),
}
