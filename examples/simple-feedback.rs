use dos_actors::prelude::*;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
use std::{ops::Deref, time::Instant};

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
            Some(vec![value, value])
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

#[derive(Debug, Default)]
struct Compensator(f64);
impl Client for Compensator {
    type I = f64;
    type O = f64;
    fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
        self.0 = data[0] - data[1];
        self
    }
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        Some(vec![self.0, self.0])
    }
}
#[derive(Debug, Default)]
pub struct Integrator {
    gain: f64,
    mem: Vec<f64>,
}
impl Integrator {
    pub fn new(gain: f64, n_data: usize) -> Self {
        Self {
            gain,
            mem: vec![0f64; n_data],
        }
    }
    pub fn last(&self) -> Option<Vec<f64>> {
        Some(self.mem.clone())
    }
}
impl Client for Integrator {
    type I = f64;
    type O = f64;
    fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
        let gain = self.gain;
        self.mem.iter_mut().zip(data).for_each(|(a, v)| {
            *a += *v * gain;
        });
        self
    }
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        self.last()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let n_sample = 2001;
    let sim_sampling_frequency = 1000f64;

    let mut signal = Signal {
        sampling_frequency: sim_sampling_frequency,
        period: 1f64,
        n_step: n_sample,
        step: 0,
    };
    let mut logging = Logging::default();

    let mut source = Initiator::<f64, 1>::build();
    let mut filter = Actor::<f64, f64, 1, 1>::new();
    let mut compensator = Actor::<f64, f64, 1, 1>::new();
    let mut integrator = Actor::<f64, f64, 1, 1>::new();
    let mut sink = Terminator::<f64, 1>::build();

    channel!(source => filter => compensator => integrator => compensator => sink);
    channel!(source => sink);

    let gain = thread_rng().gen_range(0f64..1f64);
    println!("Integrator gain: {:.3}", gain);
    spawn!(
        (source, signal,),
        (filter, Filter::default(),),
        (compensator, Compensator::default(),),
        (integrator, Integrator::new(gain, 1), vec![0f64])
    );
    let now = Instant::now();
    run!(sink, logging);
    println!("Model run in {}ms", now.elapsed().as_millis());

    let _: complot::Plot = (
        logging
            .deref()
            .chunks(2)
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x.to_vec())),
        None,
    )
        .into();

    Ok(())
}
