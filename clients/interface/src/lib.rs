#[cfg(feature = "clients")]
mod clients;
#[cfg(all(feature = "clients", feature = "nalgebra"))]
pub use clients::Gain;
#[cfg(feature = "clients")]
pub use clients::{
    leftright, Average, Integrator, Logging, Mem, OneSignal, Pulse, Sampler, Signal, Signals,
    Smooth, Source, Tick, Timer, Weight,
};
#[cfg(feature = "interface")]
pub mod interface;
