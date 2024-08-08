use std::marker::PhantomData;

use crate::{Connector, DcsData, DcsError, DcsProtocol};

type Result<T> = std::result::Result<T, DcsError>;

/**
# Generic Device Control Subystem

The generic DCS is set with the following generic parameters:
 * `P` for the [communication protocol](DcsProtocol)
 * `S` for the communication socket that implements the [Connector] interface
 * `D` for the DCS data type that implements the [DcsData] interface

Pulling the mount trajectory from the OCS:
```no_run
use gmt_dos_clients_dcs::{
    mount_trajectory::MountTrajectory,
    Dcs, Pull,
};
let dcs_pull = Dcs::<Pull, nanomsg::Socket, MountTrajectory>::new("tcp://127.0.0.1:4242")?;
# Ok::<(), anyhow::Error>(())
```

Pushing the mount trajectory to the OCS:
```no_run
use gmt_dos_clients_dcs::{
    mount_trajectory::MountTrajectory,
    Dcs,  Push,
};
let dcs_push = Dcs::<Push, nanomsg::Socket, MountTrajectory>::new("tcp://127.0.0.1:4243")?;
# Ok::<(), anyhow::Error>(())
```
*/

#[derive(Debug)]
pub struct Dcs<P: DcsProtocol, S: Connector<P>, D: DcsData, const B: usize = 1024> {
    socket: S,
    buffer: [u8; B],
    data: D,
    protocol: PhantomData<P>,
}

impl<P: DcsProtocol, S, D, const B: usize> Dcs<P, S, D, B>
where
    S: Connector<P>,
    D: DcsData,
{
    /// Creates a new DCS instance from the socket address
    pub fn new(address: &str) -> Result<Self> {
        let socket = S::new(address)?;
        Ok(Self {
            socket,
            buffer: [0; B],
            data: Default::default(),
            protocol: PhantomData,
        })
    }
}

pub trait DcsIO {}

mod pull;
mod push;
