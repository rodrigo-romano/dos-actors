use dos_actors::prelude::*;
use std::{ops::Deref, time::Instant};

mod filter;
mod logging;
mod signal;

use filter::{Filter, FilterToSink};
use logging::Logging;
use signal::{Signal, SignalToFilter};

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
