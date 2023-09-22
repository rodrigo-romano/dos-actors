/*!
# gmt_dos-actors-clients_interface

Interface definition betweeen an [actor] and an [actor]'s client.

Data is passed from the [actor] to the client by invoking [Read::read] from the client.

Data is passed from the client to the [actor] by invoking [Write::write] from the client.

The client state may be updated by invoking [Update::update] from the client

[actor]: https://docs.rs/gmt_dos-actors
*/

use std::any::type_name;

mod data;
pub use data::Data;
pub use dos_uid_derive::UID;
pub mod units;

pub type Assoc<U> = <U as UniqueIdentifier>::DataType;

/// Units conversion marker trait for clients
pub trait Units {}

/// Timer heartbeat identifier
pub enum Tick {}
impl UniqueIdentifier for Tick {
    type DataType = ();
}
/// Timer marker trait
pub trait TimerMarker {}
impl<T> Read<Tick> for T
where
    T: TimerMarker,
{
    fn read(&mut self, _: Data<Tick>) {}
}

/// Defines the data type associated with unique identifier data type
pub trait UniqueIdentifier: Send + Sync {
    const PORT: u32 = 50_000;
    type DataType;
}

/// Actor client state update interface
pub trait Update {
    fn update(&mut self) {}
}
/// Client input data reader interface
pub trait Read<U: UniqueIdentifier> {
    /// Read data from an input
    fn read(&mut self, data: Data<U>);
}
/// Client output data writer interface
pub trait Write<U: UniqueIdentifier> {
    fn write(&mut self) -> Option<Data<U>>;
}
/// Interface for IO data sizes
pub trait Size<U: UniqueIdentifier> {
    fn len(&self) -> usize;
}

pub trait Who<T> {
    /// Returns type name
    fn who(&self) -> String {
        type_name::<T>().to_string()
    }
    fn highlight(&self) -> String {
        let me = <Self as Who<T>>::who(&self);
        paris::formatter::colorize_string(format!("<italic><on-bright-cyan>{}</>", me))
    }
    fn lite(&self) -> String {
        let me = <Self as Who<T>>::who(&self);
        paris::formatter::colorize_string(format!("<italic><bright-cyan>{}</>", me))
    }
}

use log::{info, warn};

/// Pretty prints error message
pub fn print_info<S: Into<String>>(msg: S, e: Option<&dyn std::error::Error>) {
    if let Some(e) = e {
        let mut msg: Vec<String> = vec![msg.into()];
        msg.push(format!("{}", e));
        let mut current = e.source();
        while let Some(cause) = current {
            msg.push(format!("{}", cause));
            current = cause.source();
        }
        warn!("{}", msg.join("\n .due to: "))
    } else {
        info!("{}", msg.into())
    }
}

/// Interface for data logging types
pub trait Entry<U: UniqueIdentifier> {
    /// Adds an entry to the logger
    fn entry(&mut self, size: usize);
}
