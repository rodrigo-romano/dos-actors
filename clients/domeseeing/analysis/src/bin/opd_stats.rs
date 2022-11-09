use std::{fs::File, path::PathBuf, time::Instant};

use domeseeing::DomeSeeing;
use parse_monitors::cfd;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Opds {
    pub values: Vec<f64>,
    pub mask: Vec<bool>,
}
fn main() -> anyhow::Result<()> {
    /*     cfd::Baseline::<2021>::default()
    .into_iter()
    .map(|cfd_case| cfd::Baseline::<2021>::path().join(cfd_case.to_string()))
    .collect::<Vec<PathBuf>>()
    .into_par_iter()
    .for_each(|path| {
        let ds = DomeSeeing::new(path.to_str().unwrap(), 1, None)
            .unwrap()
            .masked();

        let opd_mask = ds.get(0).map(|opd| opd.mask).unwrap();

        let now = Instant::now();
        let opd_values: Vec<_> = (0..)
            .map_while(|i| ds.get(i).ok().map(|opd| opd.values))
            .collect();
        println!("Elapsed time: {}ms", now.elapsed().as_millis());

        let opds = Opds {
            values: opd_values.into_iter().flatten().collect(),
            mask: opd_mask,
        };
        bincode::serialize_into(&mut File::create(path.join("opds.bin")).unwrap(), &opds)
            .unwrap();
    }); */

    let stats: Vec<_> = cfd::Baseline::<2021>::default()
        .into_iter()
        .map(|cfd_case| cfd::Baseline::<2021>::path().join(cfd_case.to_string()))
        .collect::<Vec<PathBuf>>()
        .into_iter()
        .skip(25)
        .take(1)
        .map(|path| {
            let opds: Opds =
                bincode::deserialize_from(File::open(path.join("opds.bin")).unwrap()).unwrap();
            serde_pickle::to_writer(
                &mut File::create("opds.pkl").unwrap(),
                &opds,
                Default::default(),
            )
            .unwrap();
            let n = opds.mask.iter().filter(|&&m| m).count();
            let n_sample = opds.values.len() / n;

            let mut opd_std: Vec<_> = (0..n)
                .map(|i| {
                    opds.values.iter().skip(i).step_by(n).fold(
                        (0f64, 0f64),
                        |(mut sum, mut squared_sum), &opd| {
                            sum += opd;
                            squared_sum += opd * opd;
                            (sum, squared_sum)
                        },
                    )
                })
                .map(|(sum, squared_sum)| {
                    squared_sum / n_sample as f64 - (sum / n_sample as f64).powi(2)
                })
                .map(|x| (x.sqrt() * 1e9))
                .collect();
            opd_std.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mean = opd_std.iter().cloned().sum::<f64>() / n as f64;
            let median = if n % 2 == 0 {
                0.5 * (opd_std[n / 2 - 1] + opd_std[n / 2])
            } else {
                opd_std[n / 2]
            };
            let min = opd_std[0];
            let max = opd_std.last().unwrap().to_owned();
            let opd_stats = (mean, median, min, max);

            let mut opd_mask_std: Vec<_> = opds
                .values
                .chunks(n)
                .map(|opd| {
                    opd.iter()
                        .fold((0f64, 0f64), |(mut sum, mut squared_sum), &opd| {
                            sum += opd;
                            squared_sum += opd * opd;
                            (sum, squared_sum)
                        })
                })
                .map(|(sum, squared_sum)| squared_sum / n as f64 - (sum / n as f64).powi(2))
                .map(|x| (x.sqrt() * 1e9))
                .collect();
            opd_mask_std.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mean = opd_mask_std.iter().cloned().sum::<f64>() / n_sample as f64;
            let median = if n % 2 == 0 {
                0.5 * (opd_mask_std[n_sample / 2 - 1] + opd_mask_std[n_sample / 2])
            } else {
                opd_mask_std[n_sample / 2]
            };
            let min = opd_mask_std[0];
            let max = opd_mask_std.last().unwrap().to_owned();
            let opd_mask_stats = (mean, median, min, max);

            (opd_stats, opd_mask_stats)
        })
        .collect();

    cfd::Baseline::<2021>::default()
        .into_iter()
        .zip(&stats)
        .skip(25)
        .take(1)
        .for_each(
            |(
                cfd_case,
                ((mean, median, min, max), (mean_mask, median_mask, min_mask, max_mask)),
            )| {
                println!(
                    "{:<16} | {:>5.0} {:>5.0} {:>5.0} {:>5.0} | {:>5.0} {:>5.0} {:>5.0} {:>5.0}",
                    cfd_case.to_string(),
                    mean,
                    median,
                    min,
                    max,
                    mean_mask,
                    median_mask,
                    min_mask,
                    max_mask
                )
            },
        );

    Ok(())
}
