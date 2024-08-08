use crate::{DcsError, DcsProtocol, Pull, Push};

type Result<T> = std::result::Result<T, DcsError>;

/// Interface to the messaging library
pub trait Connector<P: DcsProtocol> {
    /// Creates a new socket at the given address
    fn new(address: &str) -> Result<Self>
    where
        Self: Sized;
}

impl Connector<Pull> for nanomsg::Socket {
    fn new(address: &str) -> Result<Self> {
        let mut socket = nanomsg::Socket::new(nanomsg::Protocol::Pull)?;
        socket.bind(address)?;
        Ok(socket)
    }
}

impl Connector<Push> for nanomsg::Socket {
    fn new(address: &str) -> Result<Self> {
        let mut socket = nanomsg::Socket::new(nanomsg::Protocol::Push)?;
        socket.connect(address)?;
        Ok(socket)
    }
}
