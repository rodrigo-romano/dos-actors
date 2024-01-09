use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{Integrator, Logging, Signal, Signals};
use interface::{Data, Read, Update, Write, UID};

// ANCHOR: io
#[derive(UID)]
enum U {}
#[derive(UID)]
enum Y {}
#[derive(UID)]
enum E {}
// ANCHOR_END: io

// ANCHOR: sum_client
pub struct Sum {
    left: Data<U>,
    right: Data<Y>,
}
impl Default for Sum {
    fn default() -> Self {
        Self {
            left: Data::new(vec![]),
            right: Data::new(vec![]),
        }
    }
}
impl Update for Sum {}
impl Read<U> for Sum {
    fn read(&mut self, data: Data<U>) {
        self.left = data.clone();
    }
}
impl Read<Y> for Sum {
    fn read(&mut self, data: Data<Y>) {
        self.right = data.clone();
    }
}
impl Write<E> for Sum {
    fn write(&mut self) -> Option<Data<E>> {
        Some(Data::new(
            self.left
                .iter()
                .zip(self.right.iter())
                .map(|(l, r)| l + r)
                .collect(),
        ))
    }
}
// ANCHOR_END: sum_client

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();
    let n_step = 1_000;
    // ANCHOR: signal
    let mut signal: Initiator<_> = Signals::new(1, n_step)
        .channel(0, Signal::Constant(1f64))
        .into();
    // ANCHOR_END: signal
    // ANCHOR: sum
    let mut sum: Actor<_> = (Sum::default(), "+").into();
    // ANCHOR_END: sum
    // ANCHOR: integrator
    let mut integrator: Actor<_> = Integrator::new(1).gain(0.5).into();
    // ANCHOR_END: integrator
    // ANCHOR: logging
    let logging = Logging::<f64>::new(3).into_arcx();
    let mut logger = Terminator::<_>::new(logging.clone());
    // ANCHOR_END: logging

    // ANCHOR: feedthrough
    signal
        .add_output()
        .multiplex(2)
        .build::<U>()
        .into_input(&mut sum)
        .into_input(&mut logger)?;
    sum.add_output()
        .multiplex(2)
        .build::<E>()
        .into_input(&mut integrator)
        .into_input(&mut logger)?;
    // ANCHOR_END: feedthrough
    // ANCHOR: feedback
    integrator
        .add_output()
        .multiplex(2)
        .bootstrap()
        .build::<Y>()
        .into_input(&mut sum)
        .into_input(&mut logger)?;
    // ANCHOR_END: feedback

    // ANCHOR: model
    model!(signal, sum, integrator, logger)
        .name("feedback-model")
        .flowchart()
        .check()?
        .run()
        .await?;
    // ANCHOR_END: model

    // ANCHOR: log
    println!("Logs:");
    println!("    :     U       E       Y");
    (*logging.lock().await)
        .chunks()
        .enumerate()
        .take(20)
        .for_each(|(i, x)| println!("{:4}: {:+.3?}", i, x));
    // ANCHOR_END: log

    Ok(())
}
