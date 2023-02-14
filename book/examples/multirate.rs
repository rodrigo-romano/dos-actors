use gmt_dos_actors::prelude::*;
use gmt_dos_actors_interface::{Data, Read, Update, Write, UID};
use gmt_dos_clients::{Average, Logging, Sampler, Signal, Signals};
use std::{collections::HashMap, sync::Arc};

// ANCHOR: io
#[derive(UID)]
enum U {}
#[derive(UID, Clone)]
enum Y {}
#[derive(UID)]
enum A {}
#[derive(UID)]
enum Z {}

// ANCHOR_END: io

// ANCHOR: sdiff_client
pub struct SignedDiff {
    left: Arc<Data<Y>>,
    right: Arc<Data<A>>,
    delta: Option<Vec<f64>>,
}
impl SignedDiff {
    pub fn new() -> Self {
        Self {
            left: Arc::new(Data::new(vec![])),
            right: Arc::new(Data::new(vec![])),
            delta: None,
        }
    }
}
impl Update for SignedDiff {
    fn update(&mut self) {
        self.left
            .iter()
            .zip(self.right.iter())
            .map(|(l, r)| l - r)
            .zip(self.delta.get_or_insert(vec![0f64; (**self.left).len()]))
            .for_each(|(d, delta)| *delta = -d * delta.signum());
    }
}
impl Read<A> for SignedDiff {
    fn read(&mut self, data: Arc<Data<A>>) {
        self.right = data.clone();
    }
}
impl Read<Y> for SignedDiff {
    fn read(&mut self, data: Arc<Data<Y>>) {
        self.left = data.clone();
    }
}
impl Write<Z> for SignedDiff {
    fn write(&mut self) -> Option<Arc<Data<Z>>> {
        self.delta
            .as_ref()
            .map(|delta| Arc::new(Data::new(delta.clone())))
    }
}
// ANCHOR_END: sdiff_client

// ANCHOR: rates
const UPRATE: usize = 2;
const DOWNRATE: usize = 4;
// ANCHOR_END: rates

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();
    // ANCHOR: signal
    let mut signal: Initiator<_> = Signals::new(1, 20)
        .channel(0, Signal::Ramp { a: 1f64, b: 0f64 })
        .into();
    // ANCHOR_END: signal
    // ANCHOR: logging
    let logging = Logging::<f64>::new(1).into_arcx();
    let mut logger = Terminator::<_>::new(logging.clone());
    // ANCHOR_END: logging

    // ANCHOR: downsampling
    let mut downsampler: Actor<_, 1, DOWNRATE> = (
        Sampler::default(),
        format!(
            r"1:{}
Downsampling",
            DOWNRATE
        ),
    )
        .into();
    // ANCHOR_END: downsampling
    // ANCHOR: upsampling
    /*     let mut upsampler: Actor<_, DOWNRATE, UPRATE> = (
            Sampler::default(),
            format!(
                "{}:{}
    Upsampling",
                DOWNRATE, UPRATE
            ),
        )
            .into(); */
    // ANCHOR_END: upsampling

    // ANCHOR: signed_diff
    let mut diff: Actor<SignedDiff, DOWNRATE, UPRATE> =
        (SignedDiff::new(), "-(Y - A)*sign(x[i-1])").into();
    // ANCHOR_END: signed_diff

    // ANCHOR: average
    let mut averager: Actor<_, 1, DOWNRATE> = (
        Average::new(1),
        format!(
            "1/{}
Average",
            DOWNRATE
        ),
    )
        .into();
    // ANCHOR_END: average

    // ANCHOR: downlogging
    let down_logging = Logging::<f64>::new(2).into_arcx();
    let mut down_logger = Terminator::<_, DOWNRATE>::new(down_logging.clone()).name(
        "Down
Logging",
    );
    // ANCHOR_END: downlogging
    // ANCHOR: uplogging
    let up_logging = Logging::<f64>::new(1).into_arcx();
    let mut up_logger = Terminator::<_, UPRATE>::new(up_logging.clone()).name(
        "Up
Logging",
    );
    // ANCHOR_END: uplogging

    // ANCHOR: network
    signal
        .add_output()
        .multiplex(3)
        .build::<U>()
        .into_input(&mut logger)
        .into_input(&mut downsampler)
        .into_input(&mut averager)?;
    downsampler
        .add_output()
        .multiplex(2)
        .build::<Y>()
        .into_input(&mut diff)
        .into_input(&mut down_logger)?;
    averager
        .add_output()
        .multiplex(2)
        .build::<A>()
        .into_input(&mut diff)
        .into_input(&mut down_logger)?;
    diff.add_output().build::<Z>().into_input(&mut up_logger)?;

    // ANCHOR_END: network

    // ANCHOR: model
    model!(
        signal,
        downsampler,
        diff,
        down_logger,
        logger,
        up_logger,
        averager
    )
    .name("multirate-model")
    .flowchart()
    .check()?
    .run()
    .await?;
    // ANCHOR_END: model

    // ANCHOR: log
    let mut data: HashMap<usize, Vec<f64>> = HashMap::new();

    (*logging.lock().await)
        .chunks()
        .enumerate()
        .for_each(|(i, x)| data.entry(i).or_insert(vec![f64::NAN; 4])[0] = x[0]);

    (*down_logging.lock().await)
        .chunks()
        .enumerate()
        .for_each(|(i, x)| {
            data.entry(DOWNRATE * (i + 1) - 1)
                .or_insert(vec![f64::NAN; 4])[1..3]
                .iter_mut()
                .zip(x)
                .for_each(|(v, x)| *v = *x);
        });

    (*up_logging.lock().await)
        .chunks()
        .enumerate()
        .for_each(|(i, x)| {
            data.entry(UPRATE * i + DOWNRATE - 1)
                .or_insert(vec![f64::NAN; 4])[3] = x[0]
        });

    // Printing the time table
    let mut sorted_data: Vec<_> = data.iter().collect();
    sorted_data.sort_by_key(|data| data.0);
    println!("Step: [  U ,  Y  ,  A  ,  Z  ]");
    sorted_data
        .iter()
        .for_each(|(k, v)| println!("{:4}: {:4.1?}", k, v));
    // ANCHOR_END: log

    Ok(())
}
