use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{logging::Logging, signals::Signals, timer::Timer};
use interface::UID;

#[derive(UID)]
pub enum In {}

#[test]
pub fn uid_input_clause() {
    // ANCHOR: uid_input_clause
    let mut timer: Initiator<_> = Timer::new(3).into();
    let mut signals: Actor<_> = Signals::new(1, 3).into();
    timer.add_output().build::<In>();
    // ANCHOR_END: uid_input_clause
}

#[test]
pub fn uid_output_clause() {
    // ANCHOR: uid_output_clause
    let mut timer: Initiator<_> = Timer::new(3).into();
    let mut logging = Logging::<f64>::new(2).into_arcx();
    let mut logger = Terminator::<_>::new(logging.clone());
    timer.add_output().build::<Tick>().into_input(&mut logger);
    // ANCHOR_END: uid_output_clause
}

#[test]
pub fn rate_clause() {
    // ANCHOR: rate_clause
    let mut timer: Initiator<_> = Timer::new(3).into();
    let mut signals: Actor<_, 2> = Signals::new(1, 3).into();
    timer
        .add_output()
        .build::<Tick>()
        .into_input(&mut signals)
        .unwrap();
    // ANCHOR_END: rate_clause
}
