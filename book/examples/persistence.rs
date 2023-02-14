use gmt_dos_actors::{model::Unknown, prelude::*};
use gmt_dos_clients::{Average, Integrator, Logging, Sampler, Signal, Signals, Tick, Timer};
use rand_distr::Normal;

mod common;
use common::{Sum, E, U, Y};

// ANCHOR: stage_iii_feedback_rate
const C: usize = 100;
// ANCHOR_END: stage_iii_feedback_rate

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();
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
            } + Signal::WhiteNoise(Normal::new(-1f64, 0.005)?),
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
    let model = |n| -> anyhow::Result<Model<Unknown>> {
        let mut timer: Initiator<_> = Timer::new(n).into();
        let mut source: Actor<_> = Actor::new(signal.clone());
        let mut sum: Actor<_> = (Sum::default(), "+").into();
        let mut feedback: Actor<_> = Actor::new(integrator.clone());
        let mut logger: Terminator<_> = Actor::new(logging.clone());

        timer.add_output().build::<Tick>().into_input(&mut source)?;
        source
            .add_output()
            .multiplex(2)
            .build::<U>()
            .into_input(&mut sum)
            .into_input(&mut logger)?;
        sum.add_output()
            .multiplex(2)
            .build::<E>()
            .into_input(&mut feedback)
            .into_input(&mut logger)?;
        feedback
            .add_output()
            .bootstrap()
            .build::<Y>()
            .into_input(&mut sum)?;

        Ok(model!(timer, source, sum, feedback, logger))
    };
    // ANCHOR_END: closure

    // STAGE I

    // ANCHOR: stage_i
    let stage_i = model(n_bootstrap)?
        .name("persistence-stage-I")
        .flowchart()
        .check()?;
    (*integrator.lock().await).set_gain(0.2);
    let stage_i = stage_i.run();
    // ANCHOR_END: stage_i

    // STAGE II

    // ANCHOR: stage_ii
    let stage_ii = model(n_fast_high_gain)?
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
    let mut avrg: Actor<_, 1, C> = Average::new(1).into();
    let mut sum: Actor<_, C, C> = (Sum::default(), "+").into();
    let mut feedback: Actor<_, C, C> = Actor::new(integrator.clone());
    // ANCHOR: sampler
    let mut upsampler: Actor<_, C, 1> = Sampler::new(vec![0f64]).into();
    // ANCHOR_END: sampler
    let mut logger: Terminator<_> = Actor::new(logging.clone());
    // ANCHOR_END: stage_iii_actors

    // ANCHOR: stage_iii_network
    source
        .add_output()
        .multiplex(2)
        .build::<U>()
        .into_input(&mut avrg)
        .into_input(&mut logger)?;
    avrg.add_output().build::<U>().into_input(&mut sum)?;
    sum.add_output()
        .multiplex(2)
        .build::<E>()
        .into_input(&mut feedback)
        .into_input(&mut upsampler)?;
    upsampler
        .add_output()
        .bootstrap()
        .build::<E>()
        .into_input(&mut logger)?;
    feedback
        .add_output()
        .bootstrap()
        .build::<Y>()
        .into_input(&mut sum)?;
    // ANCHOR_END: stage_iii_network

    // ANCHOR: stage_iii_model
    let stage_iii = model!(source, avrg, sum, feedback, upsampler, logger)
        .name("persistence-stage-III")
        .flowchart()
        .check()?;
    stage_ii.await?;
    stage_iii.run().await?;
    // ANCHOR_END: stage_iii_model

    // ANCHOR: transition_i-ii
    println!("Stage I to Stage II transition:");
    (*logging.lock().await)
        .chunks()
        .enumerate()
        .skip(n_bootstrap - 5)
        .take(10)
        .for_each(|(i, x)| println!("{:4}: {:+.3?}", i, x));
    // ANCHOR_END: transition_i-ii

    // ANCHOR: transition_ii-iii
    println!("Stage II to Stage III transition:");
    (*logging.lock().await)
        .chunks()
        .enumerate()
        .skip(n_bootstrap + n_fast_high_gain - 5)
        .take(10)
        .for_each(|(i, x)| println!("{:4}: {:+.3?}", i, x));
    // ANCHOR_END: transition_ii-iii

    // ANCHOR: stage-iii_int
    println!("Stage III (1st integration):");
    (*logging.lock().await)
        .chunks()
        .enumerate()
        .skip(C + n_bootstrap + n_fast_high_gain - 5)
        .take(10)
        .for_each(|(i, x)| println!("{:4}: {:+.3?}", i, x));
    // ANCHOR_END: stage-iii_int

    // PLOTTING

    //ANCHOR: plotting
    let _: complot::Plot = (
        (*logging.lock().await)
            .chunks()
            .enumerate()
            .map(|(i, data)| (i as f64 / sampling_frequency_hz, data.to_vec())),
        complot::complot!("persistence.png", xlabel = "Time [s]"),
    )
        .into();
    //ANCHOR_END: plotting

    Ok(())
}
