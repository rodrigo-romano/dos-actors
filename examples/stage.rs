use dos_actors::{Actor, Client, Initiator, Terminator};
use std::{ops::Deref, time::Instant};

#[derive(Default, Debug)]
struct Sinusoide {
    pub sampling_frequency: f64,
    pub period: f64,
    pub n_step: usize,
    pub step: usize,
}
impl Client for Sinusoide {
    type I = ();
    type O = f64;
    fn produce(&mut self) -> Option<Vec<f64>> {
        if self.step < self.n_step {
            let value = (2.
                * std::f64::consts::PI
                * self.step as f64
                * (self.sampling_frequency * self.period).recip())
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
    fn consume(&mut self, data: Vec<&f64>) -> &mut Self {
        self.0.extend(data.into_iter());
        self
    }
}

#[derive(Default, Debug)]
struct DoNothing(f64);
impl Client for DoNothing {
    type I = f64;
    type O = f64;
    fn consume(&mut self, data: Vec<&f64>) -> &mut Self {
        self.0 = *data[0];
        self
    }
    fn produce(&mut self) -> Option<Vec<f64>> {
        Some(vec![self.0])
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let n_sample = 1001;
    let sim_sampling_frequency = 1000f64;

    let mut client = Sinusoide {
        sampling_frequency: sim_sampling_frequency,
        period: 1f64,
        n_step: n_sample,
        step: 0,
    };
    let mut logging = Logging::default();

    let mut source = Initiator::<f64, 1>::build();
    let mut actor1 = Actor::<f64, f64, 1, 1>::new();
    let mut actor2 = Actor::<f64, f64, 1, 1>::new();
    let mut sink = Terminator::<f64, 1>::build();

    dos_actors::channel(&mut source, &mut actor1);
    dos_actors::channel(&mut actor1, &mut actor2);
    dos_actors::channel(&mut actor2, &mut sink);

    tokio::spawn(async move {
        if let Err(e) = source.run(&mut client).await {
            dos_actors::print_error("Source loop ended", &e);
        }
    });
    tokio::spawn(async move {
        if let Err(e) = actor1.run(&mut DoNothing::default()).await {
            dos_actors::print_error("Actor #1 loop ended", &e);
        }
    });
    tokio::spawn(async move {
        if let Err(e) = actor2.run(&mut DoNothing::default()).await {
            dos_actors::print_error("Actor #2 loop ended", &e);
        }
    });
    let now = Instant::now();
    if let Err(e) = sink.run(&mut logging).await {
        dos_actors::print_error("Sink loop ended", &e);
    }
    print!("Model run in {}ms", now.elapsed().as_millis());

    let _: complot::Plot = (
        logging
            .deref()
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), vec![*x])),
        None,
    )
        .into();
    Ok(())
}
