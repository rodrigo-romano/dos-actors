use domeseeing_analysis::{
    a2r0, AtmosphereOpdsBuilder, DeltaOPDSet, DeltaOPDSubset, DeltaOpdParam, OpdsBuilder,
    StructureFunctionSample, StructureFunctionSubSample, TurbulenceModel,
};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::{f64::consts::PI, fs::File, rc::Rc, time::Instant};

fn main() -> anyhow::Result<()> {
    let n_sample = 100;

    let dopds = {
        let builder = AtmosphereOpdsBuilder::default()
            .n_sample(n_sample)
            .outer_scale(500f64)
            .wind_speed(7f64);
        let opds = Rc::new(OpdsBuilder::default().atmosphere(builder).build().unwrap());
        let dopds_inner = DeltaOPDSet::builder()
            .opds(opds.clone())
            .max_baseline(0.5f64)
            .n_baseline(100_000)
            .build()
            .unwrap();
        /*         let dopds_outer = DeltaOPDSet::builder()
        .opds(opds)
        .min_baseline(0.5f64)
        .max_baseline(25f64)
        .n_baseline(100_000)
        .build()
        .unwrap(); */
        dopds_inner // + dopds_outer
    };
    println!("Computing structure functions at each time step ...");
    let now = Instant::now();
    let mut sf_t: Vec<StructureFunctionSample> = (0..n_sample)
        .into_par_iter()
        .progress_count(n_sample as u64)
        .map(|i| {
            dopds
                .iter()
                .filter(|&x| *x == DeltaOpdParam::Time(i))
                .collect::<DeltaOPDSubset>()
                .into()
        })
        .collect();
    println!(" ... in {}ms", now.elapsed().as_millis());
    println!("Fitting structure functions at each time step ...");
    let now = Instant::now();
    let (mut ac, residues): (Vec<_>, Vec<_>) = sf_t
        .par_iter()
        .map(|sf_t| {
            let mut sf: StructureFunctionSubSample = sf_t
                .iter()
                .filter(|&x| *x <= DeltaOpdParam::Baseline(0.5))
                .collect();
            sf.power_law_fit()
        })
        // .inspect(|&((a, c), _)| println!("Power law: {:5.1e} {:5.1}", a, c))
        .unzip();
    println!(" ... in {}ms", now.elapsed().as_millis());
    sf_t.iter_mut()
        .zip(ac.iter().cloned().zip(residues.iter().cloned()))
        .for_each(|(sf_t, plf)| {
            sf_t.update_fit(plf);
        });

    /*     let (mut ac, residues): (Vec<_>, Vec<_>) = (0..n_sample)
    .map(|i| {
        let mut sf: StructureFunctionSample = dopds
            .iter()
            .filter(|&x| *x == DeltaOpdParam::Time(i) && *x <= DeltaOpdParam::Baseline(0.5))
            .collect::<DeltaOPDSubset>()
            .into();
        sf.power_law_fit()
    })
    // .inspect(|&((a, c), _)| println!("Power law: {:5.1e} {:5.1}", a, c))
    .unzip(); */

    ac.sort_by(|&(a, _), &(b, _)| a.partial_cmp(&b).unwrap());
    let (sum_a, sum_c) = ac.iter().fold((0f64, 0f64), |(mut aa, mut ac), (a, c)| {
        aa += *a;
        ac += *c;
        (aa, ac)
    });
    let mean_residues = residues.iter().cloned().sum::<f64>() / residues.len() as f64;
    let mean_a = sum_a / n_sample as f64;
    let mean_c = sum_c / n_sample as f64;
    println!(
        "Mean Power law: (({:5.1e} {:5.2}, {:7.3e}))",
        mean_a, mean_c, mean_residues
    );
    let mut a_sorted = ac.iter().map(|(a, _)| a);
    println!(
        "a minmax: {:5.1e} {:5.1e}",
        a_sorted.next().unwrap(),
        a_sorted.last().unwrap()
    );
    println!("r0 = {:.3}cm", a2r0(sum_a / n_sample as f64, 0.5e-6) * 1e2);
    ac.sort_by(|&(_, a), &(_, b)| a.partial_cmp(&b).unwrap());
    let mut c_sorted = ac.iter().map(|(_, c)| c);
    println!(
        "c minmax: {:5.2} {:5.2}",
        c_sorted.next().unwrap(),
        c_sorted.last().unwrap()
    );

    let k2 = (2. * PI / 0.5e-6).powi(2);
    let tm = TurbulenceModel::new(k2 * mean_a, 0f64, mean_c);
    println!(
        "Equivalent diameter (V): {:.3?}cm",
        tm.equivalent_diameter().map(|x| x * 1e2)
    );

    // bincode::serialize_into(&mut File::create("atm-sf.bin")?, &sf_t)?;
    // serde_pickle::to_writer(&mut File::create("atm-sf.pkl")?, &sf_t, Default::default())?;

    Ok(())
}
