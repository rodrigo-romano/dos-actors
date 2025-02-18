use gmt_dos_actors::actorscript;

mod feedback;
mod filter;
mod logging;
mod sampler;
mod signal;

use feedback::DifferentiatorToIntegrator;
use filter::{Filter, FilterToDifferentiator, FilterToSampler, FilterToSink};
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
    let logging = Logging::default();

    #[cfg(not(feature = "sampler"))]
    const R: usize = 1;
    #[cfg(feature = "sampler")]
    const R: usize = 50;

    let source = signal;
    let filter = Filter::default();

    #[cfg(not(any(feature = "sampler", feature = "feedback")))]
    actorscript! {
        #[model(state = ready, flowchart = "simple")]
        1: source[SignalToFilter] -> logging
        1: source[SignalToFilter] -> filter[FilterToSink] -> logging
    }

    #[cfg(feature = "sampler")]
    actorscript! {
        #[model(state = ready, flowchart = "simple")]
        1: source[SignalToFilter] -> filter
        1: &logging
        50: filter[FilterToSampler] -> &logging
    }

    #[cfg(feature = "feedback")]
    let (model, logging) = {
        use feedback::{Differentiator, Integrator, IntegratorToDifferentiator};

        let mut compensator = Differentiator::default();
        let mut integrator = {
            use rand::Rng;
            let gain = rand::thread_rng().gen_range(0f64..1f64);
            println!("Integrator gain: {:.3}", gain);
            Integrator::new(gain, 1)
        };

        actorscript! {
            #[model(state = ready, flowchart = "simple")]
            1: source[SignalToFilter] -> &logging
            1: source[SignalToFilter]
                -> filter[FilterToDifferentiator]
                    -> compensator[DifferentiatorToIntegrator]
                        -> integrator[IntegratorToDifferentiator]!
                            -> compensator[DifferentiatorToIntegrator]
                                -> &logging
        };
        (model, logging)
    };

    model.run().wait().await?;

    let _: complot::Plot = (
        logging
            .lock()
            .await
            .chunks(2)
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x.to_vec())),
        None,
    )
        .into();

    Ok(())
}
