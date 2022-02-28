use dos_actors::prelude::*;
use std::{ops::Deref, time::Instant};

mod feedback;
mod filter;
mod logging;
mod sampler;
mod signal;

use feedback::CompensatorToIntegrator;
use filter::{Filter, FilterToCompensator, FilterToSampler, FilterToSink};
use logging::Logging;
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
    let logging = Logging::default().into_arcx();

    #[cfg(not(feature = "sampler"))]
    const R: usize = 1;
    #[cfg(feature = "sampler")]
    const R: usize = 50;

    let mut source: Actor<_, 0, 1> = signal.into();
    let mut filter: Actor<_, 1, R> = Filter::default().into();
    let mut sink = Actor::<_, 1, 0>::new(logging.clone());

    #[cfg(not(any(feature = "sampler", feature = "feedback")))]
    {
        source
            .add_output::<f64, SignalToFilter>(Some(&[1, 1]))
            .into_input(&mut filter)
            .into_input(&mut sink);

        filter
            .add_output::<f64, FilterToSink>(None)
            .into_input(&mut sink);
    }

    #[cfg(feature = "sampler")]
    {
        use sampler::Sampler;

        let mut sampler: Actor<_, R, 1> = Sampler::default().into();

        source
            .add_output::<f64, SignalToFilter>(None)
            .into_input(&mut filter);

        filter
            .add_output::<f64, FilterToSampler>(None)
            .into_input(&mut sampler);

        sampler
            .add_output::<f64, SamplerToSink>(None)
            .into_input(&mut sink);

        spawn!(sampler);
    }

    #[cfg(feature = "feedback")]
    {
        use feedback::{Compensator, Integrator, IntegratorToCompensator};

        let mut compensator: Actor<_, 1, 1> = Compensator::default().into();
        let mut integrator: Actor<_, 1, 1> = {
            use rand::Rng;
            let gain = rand::thread_rng().gen_range(0f64..1f64);
            println!("Integrator gain: {:.3}", gain);
            Integrator::new(gain, 1).into()
        };

        source
            .add_output::<f64, SignalToFilter>(Some(&[1, 1]))
            .into_input(&mut filter)
            .into_input(&mut sink);

        filter
            .add_output::<f64, FilterToCompensator>(None)
            .into_input(&mut compensator);
        compensator
            .add_output::<f64, CompensatorToIntegrator>(Some(&[1, n_sample]))
            .into_input(&mut integrator)
            .into_input(&mut sink);
        integrator
            .add_output::<f64, IntegratorToCompensator>(None)
            .into_input(&mut compensator);

        spawn!(compensator);
        spawn_bootstrap!(integrator);
    }

    spawn!(source, filter);

    let now = Instant::now();
    run!(sink);
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
