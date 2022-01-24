use dos_actors::prelude::*;
use m1_ctrl as m1;
use std::ops::Deref;

async fn m1_hardpoints(sim_sampling_frequency: f64) -> anyhow::Result<()> {
    let (mut source, mut hardpoints, mut sink) = stage!(Vec<f64>: source >> M1_HP << sink);

    channel![source => hardpoints => sink];

    let mut signals = Signals::new(vec![42], 1001).signals(Signal::Sinusoid {
        amplitude: 1e-6,
        sampling_frequency_hz: sim_sampling_frequency,
        frequency_hz: 10.,
        phase_s: 0.,
    });
    let mut m1_hp = m1::hp_dynamics::Controller::new();

    spawn!((source, signals,), (hardpoints, m1_hp,));
    let mut logging = Logging::default();
    run!(sink, logging);

    println!(
        "Logs size: {}x{}",
        logging.deref().len(),
        logging.deref().get(0).unwrap().len()
    );

    let _: complot::Plot = (
        logging
            .deref()
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x.to_owned())),
        complot::complot!(
            "examples/m1-hardpoints.png",
            xlabel = "Time [s]",
            ylabel = ""
        ),
    )
        .into();

    Ok(())
}
async fn m1_load_cells(sim_sampling_frequency: f64) -> anyhow::Result<()> {
    let (mut source, mut load_cells, mut sink) = stage!(Vec<f64>: source >> load_cells << sink);

    channel![source => load_cells; 2];
    channel![load_cells => sink];

    let mut signals = Signals::new(vec![84, 42], 1001).signals(Signal::Sinusoid {
        amplitude: 1e-6,
        sampling_frequency_hz: sim_sampling_frequency,
        frequency_hz: 10.,
        phase_s: 0.,
    });
    let mut m1_lc = m1::hp_load_cells::Controller::new();

    spawn!((source, signals,), (load_cells, m1_lc,));
    let mut logging = Logging::default();
    run!(sink, logging);

    println!(
        "Logs size: {}x{}",
        logging.deref().len(),
        logging.deref().get(0).unwrap().len()
    );

    let _: complot::Plot = (
        logging
            .deref()
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x.to_owned())),
        complot::complot!(
            "examples/m1-loadcells.png",
            xlabel = "Time [s]",
            ylabel = ""
        ),
    )
        .into();

    Ok(())
}
async fn m1_segment1(sim_sampling_frequency: f64) -> anyhow::Result<()> {
    let (mut source, mut segment1, mut sink) = stage!(Vec<f64>: source >> M1S1 << sink);

    channel![source => segment1 => sink; 2];
    channel![segment1 => sink];

    let mut signals = Signals::new(vec![6, 335], 1001).signals(Signal::Sinusoid {
        amplitude: 1e-6,
        sampling_frequency_hz: sim_sampling_frequency,
        frequency_hz: 10.,
        phase_s: 0.,
    });
    let mut m1_s1 = m1::actuators::segment1::Controller::new();

    spawn!((source, signals,), (segment1, m1_s1,));
    let mut logging = Logging::default();
    run!(sink, logging);

    println!(
        "Logs size: {}x{}",
        logging.deref().len(),
        logging.deref().get(0).unwrap().len()
    );

    let _: complot::Plot = (
        logging
            .deref()
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x.to_owned())),
        complot::complot!("examples/m1-segment1.png", xlabel = "Time [s]", ylabel = ""),
    )
        .into();

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //simple_logger::SimpleLogger::new().env().init().unwrap();
    let sim_sampling_frequency = 1000f64;
    println!("++ M1 hardpoints ++");
    m1_hardpoints(sim_sampling_frequency).await?;
    println!("++ M1 load cells ++");
    m1_load_cells(sim_sampling_frequency).await?;
    println!("++ M1 segment #1 ++");
    m1_segment1(sim_sampling_frequency).await?;
    Ok(())
}
