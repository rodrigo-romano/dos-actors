use std::sync::Arc;

use dos_actors::{
    clients::{Integrator, Signals},
    io::{Data, Read, Write},
    prelude::*,
    Update,
};
use num_complex::Complex;
use rand_distr::Normal;
use welch_sde::{Build, SpectralDensity};

pub struct Add {
    left: Vec<f64>,
    right: Vec<f64>,
}
impl Add {
    pub fn new() -> Self {
        Self {
            left: vec![0f64],
            right: vec![0f64],
        }
    }
}
impl Update for Add {}
#[derive(UID)]
pub enum W {}
impl Read<Vec<f64>, W> for Add {
    fn read(&mut self, data: Arc<Data<W>>) {
        self.left[0] = (**data)[0];
    }
}
#[derive(UID)]
pub enum I {}
impl Read<Vec<f64>, I> for Add {
    fn read(&mut self, data: Arc<Data<I>>) {
        self.right[0] = (**data)[0];
    }
}
#[derive(UID)]
pub enum D {}
impl Write<Vec<f64>, D> for Add {
    fn write(&mut self) -> Option<Arc<Data<D>>> {
        let d = self.left[0] + self.right[0];
        Some(Arc::new(Data::new(vec![d])))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let n_step = 100_000;
    let mut white_noise: Initiator<_> = (
        Signals::new(1, n_step).signals(Signal::WhiteNoise(Normal::new(0f64, 1f64)?)),
        "White Noise",
    )
        .into();
    let gain = 0.5f64;
    let mut integrator: Actor<_> = Integrator::<f64, D>::new(1).gain(gain).into();
    let y = Logging::<f64>::default().into_arcx();
    let mut sink: Terminator<_> = Actor::new(y.clone());

    let mut diff: Actor<_> = Add::new().into();

    white_noise.add_output().build::<W>().into_input(&mut diff);
    diff.add_output()
        .bootstrap()
        .multiplex(2)
        .build::<D>()
        .into_input(&mut integrator)
        .into_input(&mut sink)
        .confirm()?;
    integrator.add_output().build::<I>().into_input(&mut diff);

    Model::new(vec![
        Box::new(white_noise),
        Box::new(integrator),
        Box::new(diff),
        Box::new(sink),
    ])
    .name("integrator")
    .flowchart()
    .check()?
    .run()
    .wait()
    .await?;

    let data: &[f64] = &y.lock().await;

    let fs = 1f64;
    let welch: SpectralDensity<f64> = SpectralDensity::<f64>::builder(&data, fs)
        .dft_log2_max_size(10)
        .build();
    //let welch: PowerSpectrum<f64> = PowerSpectrum::builder(&data).dft_log2_max_size(8).build();
    println!("{welch}");
    let sd = welch.periodogram();

    println!(
        "{:?}",
        sd.frequency().into_iter().take(5).collect::<Vec<f64>>()
    );
    println!(
        "{:?}",
        sd.frequency()
            .into_iter()
            .rev()
            .take(5)
            .collect::<Vec<f64>>()
    );

    let laplace = |x: f64| Complex::i() * std::f64::consts::TAU * x;

    let open_loop_1delay: Vec<_> = sd
        .frequency()
        .iter()
        .map(|x| (-laplace(*x)).exp() * gain / (1. - (-laplace(*x)).exp()))
        .collect();
    let open_loop: Vec<_> = sd
        .frequency()
        .iter()
        .map(|x| gain / (1. - (-laplace(*x)).exp()))
        .collect();
    let closed_loop_1delay: Vec<_> = open_loop_1delay
        .iter()
        .map(|x| (1. / (1. + x)).norm_sqr())
        .collect();
    let closed_loop: Vec<_> = open_loop
        .iter()
        .map(|x| (1. / (1. + x)).norm_sqr())
        .collect();
    let _: complot::LogLog = (
        sd.frequency()
            .into_iter()
            .zip(&(*sd))
            .zip(&closed_loop_1delay)
            .zip(&closed_loop)
            .skip(1)
            .map(|(((x, &y), &cl1), &cl)| (x, vec![y, cl1, cl])),
        complot::complot!(
            "spectral_density.png",
            xlabel = "Frequency [Hz]",
            ylabel = "Spectral density [s^2/Hz]"
        ),
    )
        .into();

    Ok(())
}
