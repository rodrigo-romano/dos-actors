#[cfg(feature = "clients")]
pub mod clients;
#[cfg(all(feature = "clients", feature = "nalgebra"))]
pub use clients::Gain;
#[cfg(feature = "clients")]
pub use clients::{
    integrator, leftright, once, Average, Integrator, Logging, OneSignal, Pulse, Sampler, Signal,
    Signals, Smooth, Source, Tick, Timer, Weight,
};
#[cfg(feature = "interface")]
pub mod interface;
