use domeseeing_analysis::{
    DeltaOPDSet, DeltaOPDSubset, DeltaOpdParam, StructureFunctionSample, TurbulenceModel,
};
use indicatif::ParallelProgressIterator;
use parse_monitors::cfd::{self, CfdCase};
use rayon::prelude::*;
use serde::Serialize;
use skyangle::Conversion;
use std::{f64::consts::PI, fs::File};

#[derive(Serialize)]
pub struct Results {
    pub cfd_case: String,
    pub mean_a: f64,
    pub mean_c: f64,
    pub equiv_diam: Option<Vec<f64>>,
    pub ac: Option<Vec<(f64, f64)>>,
}

fn main() -> anyhow::Result<()> {
    let n_sample = 1000;
    let results: anyhow::Result<Vec<Results>> = cfd::Baseline::<2021>::default()
        .into_iter()
        .collect::<Vec<CfdCase<2021>>>()
        .chunks(20)
        .flat_map(|x| {
            x.into_par_iter()
                .enumerate()
                .map(|(id, cfd_case)| {
                    let dopds = DeltaOPDSet::builder()
                        .cfd_case_id(id)?
                        .max_baseline(0.5f64)
                        .n_baseline(100_000)
                        .build()?;
                    let (mut ac, residues): (Vec<_>, Vec<_>) = (0..n_sample)
                        .into_par_iter()
                        // .progress_count(n_sample as u64)
                        .map(|i| {
                            let mut sf: StructureFunctionSample = dopds
                                .iter()
                                .filter(|&x| *x == DeltaOpdParam::Time(i))
                                .collect::<DeltaOPDSubset>()
                                .into();
                            sf.power_law_fit()
                        })
                        // .inspect(|&((a, c), _)| println!("Power law: {:5.1e} {:5.1}", a, c))
                        .unzip();
                    ac.sort_by(|&(a, _), &(b, _)| a.partial_cmp(&b).unwrap());

                    let (sum_a, sum_c) =
                        ac.iter().fold((0f64, 0f64), |(mut aa, mut ac), (a, c)| {
                            aa += *a;
                            ac += *c;
                            (aa, ac)
                        });
                    let mean_residues =
                        residues.iter().cloned().sum::<f64>() / residues.len() as f64;
                    let mean_a = sum_a / n_sample as f64;
                    let mean_c = sum_c / n_sample as f64;
                    /*                     println!(
                        "Mean Power law: (({:5.1e} {:5.2}, {:7.3e}))",
                        mean_a, mean_c, mean_residues
                    );
                    let mut a_sorted = ac.iter().map(|(a, _)| a);
                    println!(
                        "a minmax: {:5.1e} {:5.1e}",
                        a_sorted.next().unwrap(),
                        a_sorted.last().unwrap()
                    );
                    let mut a_sorted = ac.iter().map(|(a, _)| a);
                    println!(
                        "a minmax: {:5.1e} {:5.1e}",
                        a_sorted.next().unwrap(),
                        a_sorted.last().unwrap()
                    );
                    ac.sort_by(|&(_, a), &(_, b)| a.partial_cmp(&b).unwrap());
                    let mut c_sorted = ac.iter().map(|(_, c)| c);
                    println!(
                        "c minmax: {:5.2} {:5.2}",
                        c_sorted.next().unwrap(),
                        c_sorted.last().unwrap()
                    );

                    let lambda = 0.5e-6;
                    let k2 = (2. * PI / lambda).powi(2);
                    let tm = TurbulenceModel::new(k2 * mean_a, 0f64, mean_c);
                    let equiv_diam = tm.equivalent_diameter();
                    println!(
                        "Equivalent diameter (V): {:.3?}cm ({:.3?}arcsec)",
                        equiv_diam.map(|x| x * 1e2),
                        equiv_diam.map(|x| (lambda / x).to_arcsec())
                    ); */

                    let lambda = 0.5e-6;
                    let k2 = (2. * PI / lambda).powi(2);
                    let equiv_diam: Option<Vec<f64>> = ac
                        .iter()
                        .map(|(a, c)| {
                            let tm = TurbulenceModel::new(k2 * a, 0f64, *c);
                            tm.equivalent_diameter()
                        })
                        .collect();

                    Ok(Results {
                        cfd_case: cfd_case.to_string(),
                        mean_a,
                        mean_c,
                        equiv_diam,
                        ac: Some(ac),
                    })
                })
                .collect::<Vec<anyhow::Result<Results>>>()
        })
        .collect();
    serde_pickle::to_writer(
        &mut File::create("domeseeing-fit_series.pkl")?,
        &results.as_ref().unwrap(),
        Default::default(),
    )?;

    Ok(())
}
