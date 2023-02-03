use gmt_dos_actors::{clients::Average, prelude::*};
use rand_distr::Normal;

mod common;
use common::{Sum, E, U, Y};

// ANCHOR: stage_iii_feedback_rate
const D: usize = 10;
// ANCHOR_END: stage_iii_feedback_rate

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ANCHOR: params
    let sim_sampling_frequency = 1000; //Hz
    let sampling_frequency_hz = sim_sampling_frequency as f64;
    let bootstrap_duration = 1; // s
    let fast_high_gain_duration = 3; // s
    let slow_high_gain_duration = 4; // s
                                     // ANCHOR_END: params
                                     // ANCHOR: stage_n_step
    let n_bootstrap = bootstrap_duration * sim_sampling_frequency;
    let n_fast_high_gain = fast_high_gain_duration * sim_sampling_frequency;
    let n_slow_high_gain = slow_high_gain_duration * sim_sampling_frequency;
    let n_step = n_bootstrap + n_fast_high_gain + n_slow_high_gain;
    // ANCHOR_END: stage_n_step
    // ANCHOR: signal
    let signal = Signals::new(1, n_step)
        .channel(
            0,
            Signal::Sinusoid {
                amplitude: 0.5f64,
                sampling_frequency_hz,
                frequency_hz: 1_f64,
                phase_s: 0f64,
            } + Signal::Sinusoid {
                amplitude: 0.1f64,
                sampling_frequency_hz,
                frequency_hz: 10_f64,
                phase_s: 0.1f64,
            } + Signal::WhiteNoise(Normal::new(-1f64, 0.01)?),
        )
        .into_arcx();
    // ANCHOR_END: signal
    // ANCHOR: integrator
    let integrator = Integrator::new(1).into_arcx();
    // ANCHOR_END: integrator
    // ANCHOR: logging
    let logging = Logging::<f64>::new(2).into_arcx();
    // ANCHOR_END: logging
    // ANCHOR: closure
    let model = |n| {
        let mut timer: Initiator<_> = Timer::new(n).into();
        let mut source: Actor<_> = Actor::new(signal.clone());
        let mut sum: Actor<_> = (Sum::default(), "+").into();
        let mut feedback: Actor<_> = Actor::new(integrator.clone());
        let mut logger: Terminator<_> = Actor::new(logging.clone());

        timer.add_output().build::<Tick>().into_input(&mut source);
        source
            .add_output()
            .multiplex(2)
            .build::<U>()
            .into_input(&mut sum)
            .into_input(&mut logger);
        sum.add_output()
            .multiplex(2)
            .build::<E>()
            .into_input(&mut feedback)
            .into_input(&mut logger);
        feedback
            .add_output()
            .bootstrap()
            .build::<Y>()
            .into_input(&mut sum);

        model!(timer, source, sum, feedback, logger)
    };
    // ANCHOR_END: closure

    // STAGE I

    // ANCHOR: stage_i
    let stage_i = model(n_bootstrap)
        .name("persistence-stage-I")
        .flowchart()
        .check()?;
    (*integrator.lock().await).set_gain(0.2);
    let stage_i = stage_i.run();
    // ANCHOR_END: stage_i

    // STAGE II

    // ANCHOR: stage_ii
    let stage_ii = model(n_fast_high_gain)
        .name("persistence-stage-II")
        .flowchart()
        .check()?;
    stage_i.await?;
    (*integrator.lock().await).set_gain(0.5);
    let stage_ii = stage_ii.run();
    // ANCHOR_END: stage_ii

    // STAGE III

    // ANCHOR: stage_iii_actors
    let mut source: Initiator<_> = Actor::new(signal.clone());
    let mut avrg: Actor<_, 1, D> = Average::new(1).into();
    let mut sum: Actor<_, D, D> = (Sum::default(), "+").into();
    let mut feedback: Actor<_, D, D> = Actor::new(integrator.clone());
    let mut upsampler: Actor<_, D, 1> = Sampler::default().into();
    let mut logger: Terminator<_> = Actor::new(logging.clone());
    // ANCHOR_END: stage_iii_actors

    // ANCHOR: stage_iii_network
    source
        .add_output()
        .multiplex(2)
        .unbounded()
        .build::<U>()
        .into_input(&mut avrg)
        .into_input(&mut logger);
    avrg.add_output().build::<U>().into_input(&mut sum);
    sum.add_output()
        .multiplex(2)
        .build::<E>()
        .into_input(&mut feedback)
        .into_input(&mut upsampler);
    upsampler.add_output().build::<E>().into_input(&mut logger);
    feedback
        .add_output()
        .bootstrap()
        .build::<Y>()
        .into_input(&mut sum);
    // ANCHOR_END: stage_iii_network

    // ANCHOR: stage_iii_model
    let stage_iii = model!(source, avrg, sum, feedback, upsampler, logger)
        .name("persistence-stage-III")
        .flowchart()
        .check()?;
    stage_ii.await?;
    stage_iii.run().await?;
    // ANCHOR_END: stage_iii_model

    // PLOTTING

    let _: complot::Plot = (
        (*logging.lock().await)
            .chunks()
            .enumerate()
            .map(|(i, data)| (i as f64 / sampling_frequency_hz, data.to_vec())),
        complot::complot!("persistence.png", xlabel = "Time [s]"),
    )
        .into();
    Ok(())
}
