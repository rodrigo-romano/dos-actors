use dos_actors::prelude::*;
use std::{ops::Deref, time::Instant};

mod feedback;
mod filter;
mod logging;
mod sampler;
mod signal;

#[cfg(feature = "feedback")]
use feedback::{Compensator, Integrator};
use feedback::{CompensatorToIntegrator, IntegratorToCompensator};
use filter::{Filter, FilterToCompensator, FilterToSampler, FilterToSink};
use logging::Logging;
#[cfg(feature = "sampler")]
use sampler::Sampler;
use sampler::SamplerToSink;
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

    #[cfg(not(feature = "sampler"))]
    const R: usize = 1;
    #[cfg(feature = "sampler")]
    const R: usize = 50;

    let mut source = Initiator::<_, 1>::build(into_arcx(signal)).tag("source");
    let mut filter = Actor::<_, 1, R>::new(into_arcx(Filter::default())).tag("filter");
    let mut sink = Terminator::<_, 1>::build(logging.clone()).tag("sink");

    #[cfg(feature = "sampler")]
    let mut sampler = Actor::<_, R, 1>::new(into_arcx(Sampler::default())).tag("sampler");

    #[cfg(feature = "feedback")]
    let mut compensator =
        Actor::<_, 1, 1>::new(into_arcx(Compensator::default())).tag("compensator");
    #[cfg(feature = "feedback")]
    let mut integrator = {
        use rand::Rng;
        let gain = rand::thread_rng().gen_range(0f64..1f64);
        println!("Integrator gain: {:.3}", gain);
        Actor::<_, 1, 1>::new(into_arcx(Integrator::new(gain, 1))).tag("integrator")
    };

    #[cfg(not(any(feature = "sampler", feature = "feedback")))]
    {
        source
            .add_output::<f64, SignalToFilter>(Some(2))
            .into_input(&mut filter)
            .into_input(&mut sink);

        filter
            .add_output::<f64, FilterToSink>(None)
            .into_input(&mut sink);
    }
    #[cfg(feature = "sampler")]
    {
        source
            .add_output::<f64, SignalToFilter>(None)
            .into_input(&mut filter);

        filter
            .add_output::<f64, FilterToSampler>(None)
            .into_input(&mut sampler);

        sampler
            .add_output::<f64, SamplerToSink>(None)
            .into_input(&mut sink);

        tokio::spawn(async move {
            if let Err(e) = sampler.run().await {
                dos_actors::print_error(format!("{} loop ended", sampler.tag.unwrap()), &e);
            };
        });
    }

    #[cfg(feature = "feedback")]
    {
        source
            .add_output::<f64, SignalToFilter>(Some(2))
            .into_input(&mut filter)
            .into_input(&mut sink);

        filter
            .add_output::<f64, FilterToCompensator>(None)
            .into_input(&mut compensator);
        compensator
            .add_output::<f64, CompensatorToIntegrator>(Some(2))
            .into_input(&mut integrator)
            .into_input(&mut sink);
        integrator
            .add_output::<f64, IntegratorToCompensator>(None)
            .into_input(&mut compensator);

        tokio::spawn(async move {
            if let Err(e) = compensator.run().await {
                dos_actors::print_error(format!("{} loop ended", compensator.tag.unwrap()), &e);
            };
        });
        tokio::spawn(async move {
            if let Err(e) = integrator.bootstrap().await {
                dos_actors::print_error(
                    format!("{} loop ended", integrator.tag.as_ref().unwrap()),
                    &e,
                );
            };
            if let Err(e) = integrator.run().await {
                dos_actors::print_error(format!("{} loop ended", integrator.tag.unwrap()), &e);
            };
        });
    }

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
