use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{logging::Logging, signals::Signals, timer::Timer};
use interface::{Tick, UID};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// no inputs positive rate
    NoInputsPositiveRate,
    /// no outputs positive rate
    NoOutputsPositiveRate,
    /// inputs outputs number mismatch
    InputsOutputsNumberMismatch,
    /// inputs outputs hashed mismatch
    InputsOutputsHashesMismatch,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::NoInputsPositiveRate => {
            // ANCHOR: noinputs_positiverate_clause
            let mut timer: Actor<Timer, 1> = Timer::new(3).into();
            let mut signals: Actor<Signals, 1> = Signals::new(1, 3).into();
            timer
                .add_output()
                .build::<Tick>()
                .into_input(&mut signals)?;
            model!(timer, signals).check()?;
            // ANCHOR_END: noinputs_positiverate_clause
        }
        Commands::NoOutputsPositiveRate => {
            // ANCHOR: nooutputs_positiverate_clause
            let mut timer: Initiator<Timer> = Timer::new(3).into();
            let mut signals: Actor<_> = Signals::new(1, 3).into();
            timer
                .add_output()
                .build::<Tick>()
                .into_input(&mut signals)?;
            let logging = Logging::<f64>::new(1).into_arcx();
            let mut logger = Actor::<_>::new(logging.clone());
            #[derive(UID)]
            enum Sig {}
            signals
                .add_output()
                .build::<Sig>()
                .into_input(&mut logger)?;
            model!(timer, signals, logger).check()?;
            // ANCHOR_END: nooutputs_positiverate_clause
        }
        Commands::InputsOutputsNumberMismatch => {
            // ANCHOR: inputs_outputs_number_mismatch_clause
            let mut timer: Initiator<Timer> = Timer::new(3).into();
            let mut signals: Actor<_> = Signals::new(1, 3).into();
            timer
                .add_output()
                .build::<Tick>()
                .into_input(&mut signals)?;
            let logging = Logging::<f64>::new(1).into_arcx();
            let mut logger = Terminator::<_>::new(logging.clone());
            #[derive(UID)]
            enum Sig {}
            signals
                .add_output()
                .build::<Sig>()
                .into_input(&mut logger)?;
            model!(timer, signals).check()?;
            // ANCHOR_END: inputs_outputs_number_mismatch_clause
        }
        Commands::InputsOutputsHashesMismatch => todo!(),
    }
    Ok(())
}
