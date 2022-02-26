use dos_actors::io::{Consuming, Data, Producing};
use dos_actors::prelude::*;
use dos_actors::Updating;
use rand_distr::{Distribution, Normal};
use std::{marker::PhantomData, ops::Deref, sync::Arc, time::Instant};

struct Signal {
    pub sampling_frequency: f64,
    pub period: f64,
    pub n_step: usize,
    pub step: usize,
    pub value: Option<f64>,
}
impl Updating for Signal {
    fn update(&mut self) {
        self.value = {
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
                Some(value)
            } else {
                None
            }
        };
    }
}
#[derive(Debug)]
enum SignalToFilter {}
impl Producing<f64, SignalToFilter> for Signal {
    fn produce(&self) -> Option<Arc<Data<f64, SignalToFilter>>> {
        self.value.map(|x| Arc::new(Data(x, PhantomData)))
    }
}

#[derive(Default)]
struct Logging(Vec<f64>);
impl Deref for Logging {
    type Target = Vec<f64>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Updating for Logging {}
impl Consuming<f64, SignalToFilter> for Logging {
    fn consume(&mut self, data: Arc<Data<f64, SignalToFilter>>) {
        self.0.push(**data);
    }
}
impl Consuming<f64, FilterToSink> for Logging {
    fn consume(&mut self, data: Arc<Data<f64, FilterToSink>>) {
        self.0.push(**data);
    }
}

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
impl Updating for Filter {
    fn update(&mut self) {
        self.data += 0.05
            * (2. * std::f64::consts::PI * self.step as f64 * (1e3f64 * 2e-2).recip()).sin()
            + self.noise.sample(&mut rand::thread_rng());
        self.step += 1;
    }
}
impl Consuming<f64, SignalToFilter> for Filter {
    fn consume(&mut self, data: Arc<Data<f64, SignalToFilter>>) {
        self.data = **data;
    }
}
#[derive(Debug)]
enum FilterToSink {}
impl Producing<f64, FilterToSink> for Filter {
    fn produce(&self) -> Option<Arc<Data<f64, FilterToSink>>> {
        Some(Arc::new(Data(self.data, PhantomData)))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let n_sample = 2001;
    let sim_sampling_frequency = 1000f64;

    let signal = Signal {
        sampling_frequency: sim_sampling_frequency,
        period: 1f64,
        n_step: n_sample,
        step: 0,
        value: None,
    };
    let logging = into_arcx(Logging::default());

    /*
    model!{
       - data type: f64
       - actors: source + filter >> sink
       - channels:
          - source => filter => sink
          - source => sink
       - clients:
          - spawn:
            - source, signal
            - filter, Filter::default()
          - run:
            - sink, logging
     }
     */

    let mut source = Initiator::<_, 1>::build(into_arcx(signal)).tag("source");
    let mut filter = Actor::<_, 1, 1>::new(into_arcx(Filter::default())).tag("filter");
    let mut sink = Terminator::<_, 1>::build(logging.clone()).tag("sink");

    source
        .add_output::<f64, SignalToFilter>(Some(2))
        .into_input(&mut filter)
        .into_input(&mut sink);

    filter
        .add_output::<f64, FilterToSink>(None)
        .into_input(&mut sink);

    //        spawn!((source, signal,), (filter, Filter::default(),));
    tokio::spawn(async move {
        if let Err(e) = source.run().await {
            dos_actors::print_error(format!("{} loop ended", source.tag.unwrap()), &e);
        };
    });
    tokio::spawn(async move {
        if let Err(e) = filter.run().await {
            dos_actors::print_error(format!("{} loop ended", filter.tag.unwrap()), &e);
        };
    });

    let now = Instant::now();
    run!(sink, logging);
    println!("Model run in {}ms", now.elapsed().as_millis());

    let _: complot::Plot = (
        logging
            .lock()
            .await
            .deref()
            .chunks(2)
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x.to_vec())),
        None,
    )
        .into();

    Ok(())
}
