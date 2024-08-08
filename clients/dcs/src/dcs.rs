use std::marker::PhantomData;

use crate::{Connector, DcsData, DcsError, DcsProtocol};

type Result<T> = std::result::Result<T, DcsError>;

#[derive(Debug)]
pub struct Dcs<P: DcsProtocol, S: Connector<P>, D: Default + DcsData, const B: usize = 1024> {
    socket: S,
    buffer: [u8; B],
    data: D,
    protocol: PhantomData<P>,
}

impl<P: DcsProtocol, S, D, const B: usize> Dcs<P, S, D, B>
where
    S: Connector<P>,
    D: Default + DcsData,
{
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

mod pull;
mod push;
