use std::{collections::HashMap, sync::Arc};

use gmt_dos_actors::{
    clients::Average,
    io::{Data, Read, Write},
    prelude::*,
    Update,
};

#[derive(UID)]
enum U {}
#[derive(UID)]
enum Y {}
#[derive(UID)]
enum DY {}
#[derive(UID)]
enum Z {}
#[derive(UID)]
enum A {}

pub struct Gain {
    gain: f64,
    value: Arc<Data<Y>>,
}
impl Gain {
    pub fn new(gain: f64) -> Self {
        Self {
            gain,
            value: Arc::new(Data::new(vec![])),
        }
    }
}
impl Update for Gain {}
impl Read<Y> for Gain {
    fn read(&mut self, data: Arc<Data<Y>>) {
        self.value = data.clone()
    }
}
impl Write<DY> for Gain {
    fn write(&mut self) -> Option<Arc<Data<DY>>> {
        Some(Arc::new(Data::new(
            self.value.iter().map(|x| x * self.gain).collect(),
        )))
    }
}

pub struct Diff {
    left: Arc<Data<Y>>,
    right: Arc<Data<A>>,
    delta: Option<Vec<f64>>,
}
impl Diff {
    pub fn new() -> Self {
        Self {
            left: Arc::new(Data::new(vec![])),
            right: Arc::new(Data::new(vec![])),
            delta: None,
        }
    }
}
impl Update for Diff {
    fn update(&mut self) {
        self.left
            .iter()
            .zip(self.right.iter())
            .map(|(l, r)| l - r)
            .zip(self.delta.get_or_insert(vec![0f64; (**self.left).len()]))
            .for_each(|(d, delta)| *delta = -d * delta.signum());
    }
}
impl Read<A> for Diff {
    fn read(&mut self, data: Arc<Data<A>>) {
        self.right = data.clone();
    }
}
impl Read<Y> for Diff {
    fn read(&mut self, data: Arc<Data<Y>>) {
        self.left = data.clone();
    }
}
impl Write<Z> for Diff {
    fn write(&mut self) -> Option<Arc<Data<Z>>> {
        self.delta
            .as_ref()
            .map(|delta| Arc::new(Data::new(delta.clone())))
    }
}

const UPRATE: usize = 2;
const DOWNRATE: usize = 4;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ANCHOR: signal`
    let mut signal: Initiator<_> = Signals::new(1, 20)
        .channel(0, Signal::Ramp { a: 1f64, b: 0f64 })
        .into();
    // ANCHOR_END: signal
    // ANCHOR: logging
    let logging = Logging::<f64>::new(1).into_arcx();
    let mut logger = Terminator::<_>::new(logging.clone());
    // ANCHOR_END: logging

    let mut downsampler: Actor<Sampler<Vec<f64>, U, Y>, 1, DOWNRATE> = (
        Sampler::default(),
        format!(
            r"1:{}
Downsampling",
            DOWNRATE
        ),
    )
        .into();
    let mut upsampler: Actor<_, DOWNRATE, UPRATE> = (
        Sampler::default(),
        format!(
            "{}:{}
Upsampling",
            DOWNRATE, UPRATE
        ),
    )
        .into();

    // let mut gain: Actor<Gain, DOWNRATE, DOWNRATE> = (Gain::new(2.), "x2 Gain").into();
    let mut diff: Actor<Diff, DOWNRATE, DOWNRATE> = (Diff::new(), "-(Y - A)*sign(x[i-1])").into();

    let mut averager: Actor<Average<f64, U, A>, 1, DOWNRATE> = (
        Average::new(1),
        format!(
            "1/{}
Average",
            DOWNRATE
        ),
    )
        .into();

    let down_logging = Logging::<f64>::new(2).into_arcx();
    let mut down_logger = Terminator::<Logging<f64>, DOWNRATE>::new(down_logging.clone()).name(
        "Down
Logging",
    );

    let up_logging = Logging::<f64>::new(1).into_arcx();
    let mut up_logger = Terminator::<_, UPRATE>::new(up_logging.clone()).name(
        "Up
Logging",
    );

    signal
        .add_output()
        .multiplex(3)
        .build::<U>()
        .into_input(&mut logger)
        .into_input(&mut downsampler)
        .into_input(&mut averager);
    downsampler
        .add_output()
        .multiplex(2)
        .build::<Y>()
        .into_input(&mut diff)
        .into_input(&mut down_logger);
    diff.add_output().build::<Z>().into_input(&mut upsampler);
    upsampler
        .add_output()
        .build::<Z>()
        .into_input(&mut up_logger);
    averager
        .add_output()
        .multiplex(2)
        .build::<A>()
        .into_input(&mut diff)
        .into_input(&mut down_logger);

    // ANCHOR: model
    Model::new(vec_box![
        signal,
        downsampler,
        upsampler,
        diff,
        down_logger,
        logger,
        up_logger,
        averager
    ])
    .name("multirate-model")
    .flowchart()
    .check()?
    .run()
    .await?;
    // ANCHOR_END: model

    let mut data: HashMap<usize, Vec<f64>> = HashMap::new();

    // ANCHOR: log
    (*logging.lock().await)
        .chunks()
        .enumerate()
        .for_each(|(i, x)| data.entry(i).or_insert(vec![f64::NAN; 4])[0] = x[0]);
    // ANCHOR_END: log

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

    let mut sorted_data: Vec<_> = data.iter().collect();
    sorted_data.sort_by_key(|data| data.0);
    println!("Step: [  U ,  Y  ,  A  ,  Z  ]");
    sorted_data
        .iter()
        .for_each(|(k, v)| println!("{:4}: {:4.1?}", k, v));

    Ok(())
}
