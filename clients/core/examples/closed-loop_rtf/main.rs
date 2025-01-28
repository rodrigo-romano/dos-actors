use gmt_dos_actors::actorscript;
use gmt_dos_clients::{
    average::Average,
    integrator::Integrator,
    operator::{Left, Operator, Right},
    signals::{Signal, Signals},
};
use interface::UID;
use welch_sde::{Build, SpectralDensity};

#[derive(UID)]
#[uid(port = 5001)]
pub enum N {}
#[derive(UID)]
#[uid(port = 5003)]
pub enum R {}
#[derive(UID)]
#[uid(port = 5004)]
pub enum AR {}
#[derive(UID)]
#[uid(port = 5002)]
pub enum SR {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let n_step = 1_000_000;

    let white_noise = Signals::new(1, n_step).channels(Signal::white_noise()?);

    let operator = Operator::<f64>::new("+");

    let int = Integrator::new(1).gain(0.5);

    let avrg = Average::new(1);

    actorscript!(
        1: white_noise[Left<N>]${1} -> operator[R]${1} -> avrg
        1: avrg[AR] -> int
        1: int[Right<SR>]! -> operator
    );

    let data = &mut model_logging_1.lock().await;
    println!("{}", data);
    let input: Vec<f64> = data.iter("Left<N>")?.flatten().collect();
    let output: Vec<f64> = data.iter("R")?.flatten().collect();

    let fs = 1f64;
    let welch: SpectralDensity<f64> = SpectralDensity::<f64>::builder(&input, fs).build();
    println!("{}", welch);
    let input_sd = welch.periodogram();
    let welch: SpectralDensity<f64> = SpectralDensity::<f64>::builder(&output, fs).build();
    let output_sd = welch.periodogram();

    let rtf: Vec<_> = output_sd
        .iter()
        .zip(input_sd.iter())
        .map(|(o, i)| o / i)
        .collect();
    let _: complot::LogLog = (
        input_sd
            .frequency()
            .into_iter()
            .zip(&rtf)
            .skip(1)
            .map(|(x, &y)| (x, vec![y])),
        complot::complot!(
            "spectral_density.png",
            xlabel = "Frequency [Hz]",
            ylabel = "Spectral density [s^2/Hz]"
        ),
    )
        .into();
    Ok(())
}
