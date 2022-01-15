use std::ops::Deref;

use dos_actors::{into_arcx, io, Actor, Client, Initiator, Terminator};

#[derive(Default, Debug)]
struct Sinusoide {
    pub sampling_frequency: f64,
    pub period: f64,
    pub n_step: usize,
    pub step: usize,
}
impl Client<f64, f64> for Sinusoide {
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
impl Client<f64, f64> for Logging {
    fn consume(&mut self, data: Vec<&f64>) -> &mut Self {
        self.0.extend(data.into_iter());
        self
    }
}

#[derive(Default, Debug)]
struct DoNothing(f64);
impl Client<f64, f64> for DoNothing {
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
    let n_sample = 21;
    let sim_sampling_frequency = 20f64;

    let client = into_arcx(Sinusoide {
        sampling_frequency: sim_sampling_frequency,
        period: 1f64,
        n_step: n_sample,
        step: 0,
    });
    let mut source = Initiator::<Sinusoide, f64, 1>::build(client.clone());

    let mut actor1 = Actor::<DoNothing, f64, f64, 1, 1>::new(into_arcx(DoNothing::default()));
    let mut actor2 = Actor::<DoNothing, f64, f64, 1, 1>::new(into_arcx(DoNothing::default()));

    let logging = into_arcx(Logging::default());
    let mut sink = Terminator::<Logging, f64, 1>::build(logging.clone());

    {
        let (output, input) = io::channel();
        source.add_output(output);
        actor1.add_input(input);
    }
    {
        let (output, input) = io::channel();
        actor1.add_output(output);
        actor2.add_input(input);
    }
    {
        let (output, input) = io::channel();
        actor2.add_output(output);
        sink.add_input(input);
    }
    tokio::spawn(async move {
        if let Err(e) = source.task().await {
            dos_actors::error_msg("Source loop ended", &e);
        }
    });
    tokio::spawn(async move {
        if let Err(e) = actor1.task().await {
            dos_actors::error_msg("Actor #1 loop ended", &e);
        }
    });
    tokio::spawn(async move {
        if let Err(e) = actor2.task().await {
            dos_actors::error_msg("Actor #2 loop ended", &e);
        }
    });
    if let Err(e) = sink.task().await {
        dos_actors::error_msg("Sink loop ended", &e);
    }
    dbg!(&logging);

    let _: complot::Plot = (
        (*logging.lock().await)
            .deref()
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), vec![*x])),
        None,
    )
        .into();
    Ok(())
}
