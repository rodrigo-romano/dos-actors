use gmt_dos_actors::prelude::*;

// ANCHOR: io
#[derive(UID)]
enum Sinusoides {}
#[derive(UID)]
enum UpDown {}
// ANCHOR_END: io

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();
    // ANCHOR: n_step
    let n_step = 9;
    // ANCHOR_END: n_step
    {
        // ANCHOR: signals
        let mut signals: Initiator<_> = Signals::new(2, n_step)
            .channel(
                0,
                Signal::Sinusoid {
                    amplitude: 1f64,
                    sampling_frequency_hz: (n_step - 1) as f64,
                    frequency_hz: 1f64,
                    phase_s: 0f64,
                },
            )
            .channel(
                1,
                Signal::Sinusoid {
                    amplitude: 1f64,
                    sampling_frequency_hz: (n_step - 1) as f64,
                    frequency_hz: 1f64,
                    phase_s: 0.5f64,
                },
            )
            .into();
        // ANCHOR_END: signals

        // ANCHOR: source
        let mut source: Initiator<_> = Source::new(
            (0..n_step)
                .flat_map(|x| vec![x as f64, (n_step - x - 1) as f64]) // 2 channels
                .collect(),
            2,
        )
        .into();
        // ANCHOR_END: source

        // ANCHOR: logging
        let logging = Logging::<f64>::new(2).into_arcx();
        let mut logger = Terminator::<_>::new(logging.clone());
        // ANCHOR_END: logging

        // ANCHOR: model
        signals
            .add_output()
            .unbounded()
            .build::<Sinusoides>()
            .into_input(&mut logger);
        source
            .add_output()
            .unbounded()
            .build::<UpDown>()
            .into_input(&mut logger);

        model!(signals, source, logger)
            .name("signals-logger")
            .flowchart()
            .check()?
            .run()
            .await?;
        // ANCHOR_END: model

        // ANCHOR: logs
        println!("Logs:");
        (*logging.lock().await)
            .chunks()
            .enumerate()
            .for_each(|(i, x)| println!("{}: {:+.3?}", i, x));
        // ANCHOR_END: logs
    }
    {
        // ANCHOR: signals_and_source
        let mut signals: Actor<_> = Signals::new(2, n_step)
            .channel(
                0,
                Signal::Sinusoid {
                    amplitude: 1f64,
                    sampling_frequency_hz: (n_step - 1) as f64,
                    frequency_hz: 1f64,
                    phase_s: 0f64,
                },
            )
            .channel(
                1,
                Signal::Sinusoid {
                    amplitude: 1f64,
                    sampling_frequency_hz: (n_step - 1) as f64,
                    frequency_hz: 1f64,
                    phase_s: 0.5f64,
                },
            )
            .into();

        let mut source: Actor<_> = Source::new(
            (0..n_step)
                .flat_map(|x| vec![x as f64, (n_step - x - 1) as f64])
                .collect(),
            2,
        )
        .into();
        // ANCHOR_END: signals_and_source

        // ANCHOR: timer
        let mut timer: Initiator<_> = Timer::new(n_step / 2).into();
        // ANCHOR_END: timer

        let logging = Logging::<f64>::new(2).into_arcx();
        let mut logger = Terminator::<_>::new(logging.clone());

        // ANCHOR: timer_signals_source
        timer
            .add_output()
            .multiplex(2)
            .build::<Tick>()
            .into_input(&mut signals)
            .into_input(&mut source);
        signals
            .add_output()
            .unbounded()
            .build::<Sinusoides>()
            .into_input(&mut logger);
        source
            .add_output()
            .unbounded()
            .build::<UpDown>()
            .into_input(&mut logger);
        // ANCHOR_END: timer_signals_source

        // ANCHOR: model_with_timer
        model!(timer, signals, source, logger)
            .name("signals-logger-trunc")
            .flowchart()
            .check()?
            .run()
            .await?;
        // ANCHOR_END: model_with_timer

        // ANCHOR: trunc-logs
        println!("Logs:");
        (*logging.lock().await)
            .chunks()
            .enumerate()
            .for_each(|(i, x)| println!("{}: {:+.3?}", i, x));
        // ANCHOR_END: trunc-logs
    }
    Ok(())
}
