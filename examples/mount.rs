use dos_actors::prelude::*;
use mount_ctrl as mount;
use std::ops::Deref;

async fn controller(sim_sampling_frequency: f64) -> anyhow::Result<()> {
    let (mut source, mut mount_controller, mut sink) =
        stage!(Vec<f64>: source >> mount_controller << sink);

    channel![source => mount_controller => sink; 3];

    let mut signals = Signals::new(vec![4, 6, 4], 1001).signals(Signal::Sinusoid {
        amplitude: 1e-6,
        sampling_frequency_hz: sim_sampling_frequency,
        frequency_hz: 10.,
        phase_s: 0.,
    });
    let mut mnt_ctrl = mount::controller::Controller::new();

    spawn!((source, signals,), (mount_controller, mnt_ctrl,));
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
            "examples/mount-controller.png",
            xlabel = "Time [s]",
            ylabel = ""
        ),
    )
        .into();

    Ok(())
}

async fn driver(sim_sampling_frequency: f64) -> anyhow::Result<()> {
    let (mut source, mut mount_driver, mut sink) = stage!(Vec<f64>: source >> mount_driver << sink);

    channel!(source => mount_driver);
    channel![source => mount_driver => sink; 3];

    let mut signals = Signals::new(vec![3, 4, 6, 4], 1001).signals(Signal::Sinusoid {
        amplitude: 1e-6,
        sampling_frequency_hz: sim_sampling_frequency,
        frequency_hz: 10.,
        phase_s: 0.,
    });
    let mut mnt_driver = mount::drives::Controller::new();

    spawn!((source, signals,), (mount_driver, mnt_driver,));
    let mut logging = Logging::default();
    run!(sink, logging);

    let _: complot::Plot = (
        logging
            .deref()
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x.to_owned())),
        complot::complot!(
            "examples/mount-driver.png",
            xlabel = "Time [s]",
            ylabel = ""
        ),
    )
        .into();

    Ok(())
}

async fn both(sim_sampling_frequency: f64) -> anyhow::Result<()> {
    let (mut source, mut mount_controller, mut mount_driver, mut sink) =
        stage!(Vec<f64>: source >> mount_controller, mount_driver << sink);

    channel![source => mount_controller; 3];
    channel![mount_controller => mount_driver];
    channel![source => mount_driver => sink; 3];

    let mut signals = Signals::new(vec![4, 6, 4], 1001).signals(Signal::Sinusoid {
        amplitude: 1e-6,
        sampling_frequency_hz: sim_sampling_frequency,
        frequency_hz: 10.,
        phase_s: 0.,
    });
    let mut mnt_ctrl = mount::controller::Controller::new();
    let mut mnt_driver = mount::drives::Controller::new();

    spawn!(
        (source, signals,),
        (mount_controller, mnt_ctrl,),
        (mount_driver, mnt_driver,)
    );
    let mut logging = Logging::default();
    run!(sink, logging);

    let _: complot::Plot = (
        logging
            .deref()
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x.to_owned())),
        complot::complot!("examples/mount.png", xlabel = "Time [s]", ylabel = ""),
    )
        .into();

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //simple_logger::SimpleLogger::new().env().init().unwrap();
    let sim_sampling_frequency = 1000f64;
    println!("++ Mount controller ++");
    controller(sim_sampling_frequency).await?;
    println!("++ Mount driver ++");
    driver(sim_sampling_frequency).await?;
    println!("++ Mount controller & driver ++");
    both(sim_sampling_frequency).await?;
    Ok(())
}
