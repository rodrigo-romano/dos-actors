use domeseeing::DomeSeeing;
use rayon::prelude::*;
use std::fs::File;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let (kl, n_kl): (Vec<f64>, usize) =
        serde_pickle::from_reader(File::open("data/KL255036.pkl")?, Default::default())?;
    println!("Orthonormalization");
    let now = Instant::now();
    let nkl = zernike::gram_schmidt(&kl, n_kl);
    println!(" done in {}ms", now.elapsed().as_millis());
    let dome_seeing = DomeSeeing::new("/fsx/CASES/zen30az000_OS7", 1, None)?.masked();
    let n_sample = dbg!(dome_seeing.len());
    let n_mode = 1000;
    let n = nkl.len() / n_kl;
    println!("Projection");
    let now = Instant::now();
    let b: Vec<f64> = dome_seeing
        .take(n_sample - 1)
        .flat_map(|opd| {
            nkl.par_chunks(n)
                .take(n_mode)
                .map(|kl| kl.iter().zip(&opd).map(|(kl, opd)| *kl * *opd).sum::<f64>())
                .collect::<Vec<f64>>()
        })
        .collect();
    println!(" done in {}ms", now.elapsed().as_millis());
    dbg!(b.len());

    //println!("{b:?}");

    let n = (b.len() / n_mode) as f64;
    let b_var: Vec<f64> = (0..n_mode)
        .map(|i| {
            let b_mean = b.iter().skip(i).step_by(n_mode).sum::<f64>() / n;
            b.iter()
                .skip(i)
                .step_by(n_mode)
                .map(|b| b - b_mean)
                .map(|x| x * x)
                .sum::<f64>()
                / n
        })
        .collect();
    dbg!(b_var.len());

    serde_pickle::to_writer(&mut File::create("kl_var.pkl")?, &b_var, Default::default())?;

    Ok(())
}
