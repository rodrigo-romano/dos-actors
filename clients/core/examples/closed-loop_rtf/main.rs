use num_complex::{Complex64, ComplexFloat};
use std::f64::consts::PI;

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

    let g = 0.5;
    let int = Integrator::new(1).gain(g);

    let avrg = Average::new(1);

    actorscript!(
        1: white_noise[Left<N>]${1} -> operator[R]${1} -> avrg
        2: avrg[AR] -> int
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

    let z = |nu: f64, l: f64| (l * 2f64 * Complex64::I * PI * nu).exp();
    let g_z = |nu: f64, g: f64, l: f64, m: f64| {
        g * z(nu, -l) * (m + (1. - m) * z(nu, -1.)) / (1f64 - z(nu, -1.))
    };
    let rtf0 = |nu: f64, g: f64, l: f64, m: f64| (1f64 / (1f64 + g_z(nu, g, l, m))).abs().powi(2);

    let _: complot::LogLog = (
        input_sd
            .frequency()
            .into_iter()
            .zip(&rtf)
            .skip(1)
            .map(|(x, &y)| (x, vec![y, rtf0(x, g, 1., 1.)])),
        complot::complot!(
            "spectral_density.png",
            xlabel = "Frequency [Hz]",
            ylabel = "Spectral density [s^2/Hz]"
        ),
    )
        .into();
    Ok(())
}
