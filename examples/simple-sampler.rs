use dos_actors::prelude::*;
use rand_distr::{Distribution, Normal};
use std::{
    ops::{Deref, DerefMut},
    time::Instant,
};

#[derive(Default, Debug)]
struct Signal {
    pub sampling_frequency: f64,
    pub period: f64,
    pub n_step: usize,
    pub step: usize,
}
impl Client for Signal {
    type I = ();
    type O = f64;
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        if self.step < self.n_step {
            let value = (2.
                * std::f64::consts::PI
                * self.step as f64
                * (self.sampling_frequency * self.period).recip())
            .sin()
                - 0.25
                    * (2.
                        * std::f64::consts::PI
                        * ((self.step as f64
                            * (self.sampling_frequency * self.period * 0.25).recip())
                            + 0.1))
                        .sin();
            self.step += 1;
            Some(vec![value])
        } else {
            None
        }
    }
}
#[derive(Default, Debug)]
struct Logging(Vec<f64>);
impl Deref for Logging {
    type Target = Vec<f64>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Client for Logging {
    type I = f64;
    type O = ();
    fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
        self.0.extend(data.into_iter());
        self
    }
}

#[derive(Debug)]
struct Filter {
    data: f64,
    noise: Normal<f64>,
    step: usize,
}
impl Default for Filter {
    fn default() -> Self {
        Self {
            data: 0f64,
            noise: Normal::new(0.3, 0.05).unwrap(),
            step: 0,
        }
    }
}
impl Client for Filter {
    type I = f64;
    type O = f64;
    fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
        self.data = *data[0];
        self
    }
    fn update(&mut self) -> &mut Self {
        self.data += 0.05
            * (2. * std::f64::consts::PI * self.step as f64 * (1e3f64 * 2e-2).recip()).sin()
            + self.noise.sample(&mut rand::thread_rng());
        self.step += 1;
        self
    }
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        Some(vec![self.data])
    }
}

#[derive(Default, Debug)]
struct Sampler(f64);
impl Client for Sampler {
    type I = f64;
    type O = f64;
    fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
        self.0 = *data[0];
        self
    }
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        Some(vec![self.0])
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let n_sample = 2001;
    let sim_sampling_frequency = 1000f64;
    const R: usize = 50;
    let sampler_frequency = sim_sampling_frequency / R as f64;

    let mut signal = Signal {
        sampling_frequency: sim_sampling_frequency,
        period: 1f64,
        n_step: n_sample,
        step: 0,
    };

    let mut source = Initiator::<f64, 1>::build();
    let mut filter = Actor::<f64, f64, 1, R>::new();
    let mut sampler = Actor::<f64, f64, R, 1>::new();
    let mut sink = Terminator::<f64, 1>::build();
    //let mut sinkr = Terminator::<f64, R>::build();

    channel!(source => filter => sampler => sink);

    let now = Instant::now();
    spawn!(
        (source, signal,),
        (filter, Filter::default(),),
        (sampler, Sampler::default(),)
    );

    let mut logging = Logging::default();
    /*
    let logging = dos_actors::into_arcx(Logging::default());
    let logging_ref = logging.clone();
    tokio::spawn(async move {
    })
    .await?;*/
    run!(sink, logging);
    println!("Model run in {}ms", now.elapsed().as_millis());

    let _: complot::Plot = (
        logging
            .deref()
            .chunks(1)
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x.to_vec())),
        None,
    )
        .into();
    /*
        let _: complot::Scatter = (
            loggingr
                .lock()
                .await
                .deref()
                .deref()
                .iter()
                .enumerate()
                .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), vec![*x])),
            None,
        )
            .into();
    */
    Ok(())
}
