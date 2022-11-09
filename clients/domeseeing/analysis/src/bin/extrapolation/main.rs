//use crseo::{Atmosphere, Builder, FromBuilder, Source};
use argmin::core::observers::{ObserverMode, SlogLogger};
use argmin::core::{Error, Executor, Jacobian, Operator};
use argmin::solver::gaussnewton::GaussNewton;
use domeseeing_analysis::Opds;
use nalgebra as na;
use parse_monitors::cfd;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use serde::Serialize;
use statrs::function::gamma::gamma;
use std::collections::HashMap;
use std::{f64::consts::PI, fs::File, path::Path, time::Instant};

use nalgebra::{DMatrix, DVector};

mod turbulence_model;
use turbulence_model::TurbulenceModel;

#[derive(Debug, Serialize)]
pub struct StructFun {
    pub r: f64,
    pub o: f64,
    pub var: f64,
}

struct Problem {
    data: Vec<(f64, f64)>,
    a: f64,
    c: f64,
}

impl Operator for Problem {
    type Param = DVector<f64>;
    type Output = DVector<f64>;

    fn apply(&self, p: &Self::Param) -> Result<Self::Output, Error> {
        Ok(DVector::from_vec(
            self.data
                .iter()
                .map(|(r, sf)| {
                    sf - TurbulenceModel::new(self.a, p[0].abs(), self.c).structure_function(*r)
                })
                .collect(),
        ))
    }
}

impl Jacobian for Problem {
    type Param = DVector<f64>;
    type Jacobian = DMatrix<f64>;

    fn jacobian(&self, p: &Self::Param) -> Result<Self::Jacobian, Error> {
        Ok(DMatrix::from_fn(self.data.len(), 1, |ri, i| {
            let mdl = TurbulenceModel::new(self.a, p[0].abs(), self.c);
            mdl.b_partial_structure_function(self.data[ri].0)
        }))
    }
}

/* #[derive(Debug)]
pub struct DeltaOpd {
    r: f64,
    o: f64,
    pub delta_opd: Vec<f64>,
}
impl DeltaOpd {
    pub fn new(r: f64, o: f64, n_sample: usize) -> Self {
        Self {
            r,
            o,
            delta_opd: Vec::with_capacity(n_sample),
        }
    }
    pub fn push(&mut self, delta_opd: f64) -> &mut Self {
        self.delta_opd.push(delta_opd);
        self
    }
    pub fn polar(&self) -> (f64, f64) {
        (self.r, self.o)
    }
    pub fn var(&self) -> f64 {
        let n = self.delta_opd.len() as f64;
        let mean = self.delta_opd.iter().cloned().sum::<f64>() / n;
        self.delta_opd
            .iter()
            .map(|x| x - mean)
            .map(|x| x * x)
            .sum::<f64>()
            / n
    }
} */
pub fn polyfit<T: na::RealField + Copy>(
    x_values: &[T],
    y_values: &[T],
    polynomial_degree: usize,
) -> Result<Vec<T>, &'static str> {
    let number_of_columns = polynomial_degree + 1;
    let number_of_rows = x_values.len();
    let mut a = na::DMatrix::zeros(number_of_rows, number_of_columns);

    for (row, &x) in x_values.iter().enumerate() {
        // First column is always 1
        a[(row, 0)] = T::one();

        for col in 1..number_of_columns {
            a[(row, col)] = x.powf(na::convert(col as f64));
        }
    }

    let b = na::DVector::from_row_slice(y_values);

    let decomp = na::SVD::new(a, true, true);

    match decomp.solve(&b, na::convert(1e-18f64)) {
        Ok(mat) => Ok(mat.data.into()),
        Err(error) => Err(error),
    }
}

#[test]
fn gaussnewton() {
    let mdl = TurbulenceModel::new(1., 1. / 100., 11. / 6.);
    let structure_fun: Vec<_> = (1..=10)
        .map(|i| {
            let r = i as f64;
            (r, mdl.structure_function(r))
        })
        .collect();
    let cost = Problem {
        data: structure_fun,
        a: 1.,
        c: 11. / 6.,
    };
    let init_param: DVector<f64> = DVector::from_vec(vec![1. / 50.]);
    let solver: GaussNewton<f64> = GaussNewton::new();
    let res = Executor::new(cost, solver)
        .configure(|state| state.param(init_param).max_iters(10))
        // .add_observer(SlogLogger::term(), ObserverMode::Always)
        .run()
        .unwrap();
    println!("{}", res);
}

#[derive(PartialEq)]
enum Turbulence {
    Atmosphere,
    DomeSeeing,
}

fn main() -> anyhow::Result<()> {
    let n_xy = 104;
    let n_sample = 1000;
    let delta = 0.25;
    let diameter = delta * (n_xy - 1) as f64;
    println!("D: {diameter}m");

    let turbulence = Turbulence::DomeSeeing;

    let mut results: HashMap<String, (Vec<(f64, f64)>, Vec<f64>, (f64, f64, f64))> = HashMap::new();

    for cfd_case in cfd::Baseline::<2021>::default().into_iter() {
        println!("CFD CASE: {cfd_case}");

        println!("Collecting OPD samples ...");
        let now = Instant::now();
        /*     let mut src = Source::builder()
            .pupil_sampling(n_xy)
            .pupil_size(diameter)
            .build()?;
        let mut atm = Atmosphere::builder()
            .single_turbulence_layer(0., None, None)
            .build()?;

        /*     let opd: Vec<f64> = (0..n_sample)
            .flat_map(|_| {
                atm.reset();
                atm.get_phase_values(&mut src, &vec![0.], &vec![0.], 0.)
            })
            .collect();
        let n = n_sample as f64;
        let mean = opd.iter().cloned().sum::<f64>() / n;
        let var = opd.iter().map(|x| x - mean).map(|x| x * x).sum::<f64>() / n;
        dbg!(var); */

        let (x, y): (Vec<_>, Vec<_>) = xy.iter().cloned().unzip();
        let opd: Vec<Vec<f64>> = (0..n_sample)
            .map(|_| {
                atm.reset();
                atm.get_phase_values(&mut src, x.as_slice(), y.as_slice(), 0.)
            })
            .collect();
        bincode::serialize_into(&mut File::create("atmosphere_opd.bin")?, &opd)?; */
        let path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
        let mut opds: Opds =
            bincode::deserialize_from(File::open(path.join("opds.bin")).unwrap()).unwrap();

        if turbulence == Turbulence::Atmosphere {
            println!("Replacing dome seeing with atmospheric turbulence");
            let opd: Vec<Vec<f64>> = bincode::deserialize_from(File::open("atmosphere_opd.bin")?)?;
            opds.values = opd
                .into_iter()
                .flat_map(|opd| {
                    opd.into_iter()
                        .zip(opds.mask.iter())
                        .filter(|(_, &m)| m)
                        .map(|(opd, m)| opd)
                        .collect::<Vec<f64>>()
                })
                .collect();
        }
        println!(" ... in {}ms", now.elapsed().as_millis());

        let n = opds.mask.iter().filter(|&&m| m).count();
        let n_sample = opds.values.len() / n;
        dbg!((n, n_sample));

        let n_xy2 = n_xy * n_xy;
        let mut xy: Vec<(f64, f64)> = Vec::with_capacity(n_xy2);
        let mut mask_iter = opds.mask.iter();
        for i in 0..n_xy {
            for j in 0..n_xy {
                if *mask_iter.next().unwrap() {
                    let x = (i as f64) * delta;
                    let y = (j as f64) * delta;
                    xy.push((x, y));
                }
            }
        }

        let var = (0..n)
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
            // .reduce(f64::max)
            // .unwrap();
            .sum::<f64>()
            / n as f64;
        dbg!(var);

        println!("Computing structure function ...");
        let now = Instant::now();
        let mut sf: Vec<StructFun> = vec![];
        dbg!(n_xy.pow(2));
        let n_baseline = 100_000;
        let mut rng = thread_rng();
        while sf.len() < n_baseline {
            let i = rng.gen_range(0..xy.len());
            let j = rng.gen_range(0..xy.len());
            let (x1, y1) = xy[i];
            let (x2, y2) = xy[j];
            let x = x2 - x1;
            let y = y2 - y1;
            let r = x.hypot(y);
            if r == 0f64 || r > 0.5 * diameter {
                continue;
            }
            let o = y.atan2(x);
            let (sum, squared_sum) =
                opds.values
                    .chunks(n)
                    .fold((0f64, 0f64), |(mut sum, mut squared_sum), opd| {
                        let opd1 = opd[i];
                        let opd2 = opd[j];
                        let d_opd = opd2 - opd1;
                        sum += d_opd;
                        squared_sum += d_opd * d_opd;
                        (sum, squared_sum)
                    });
            let sf_var = squared_sum / n_sample as f64 - (sum / n_sample as f64).powi(2);
            sf.push(StructFun { r, o, var: sf_var });
        }
        sf.sort_by(|sfa, sfb| sfa.r.partial_cmp(&sfb.r).unwrap());
        println!(" ... in {}ms", now.elapsed().as_millis());
        println!("Structure function samples #: {}", sf.len());
        // serde_pickle::to_writer(&mut File::create("sf.pkl")?, &sf, Default::default())?;

        println!("Computing the azimuthal average of the structure functions");
        let mut n = 0;
        let mut structure_fun: Vec<(f64, f64)> = Vec::with_capacity(10_000);
        while let Some(sf0) = sf.get(n).as_ref() {
            let (mut sf_mean, n_sf) = sf
                .iter()
                .skip(n)
                .take_while(|sf| sf.r - sf0.r == 0f64)
                .fold((0f64, 0usize), |(mut a, mut n), sf| {
                    a += sf.var;
                    n += 1;
                    (a, n)
                });
            sf_mean /= n_sf as f64;
            structure_fun.push((sf0.r, sf_mean));
            n += n_sf;
        }

        let (x, y): (Vec<_>, Vec<_>) = structure_fun
            .iter()
            .filter(|(r, _)| *r < 2.)
            .map(|(r, sf)| ((std::f64::consts::PI * r).ln(), sf.ln()))
            .unzip();
        let fit = polyfit(&x, &y, 1).unwrap();
        dbg!(&fit);
        let c = fit[1] / 2. + 1.;
        println!("PSD exponent: {c}");
        let a = -1. * fit[0].exp() * gamma(c).powi(2) * (PI * c).sin() / (2. * PI.powi(2));
        println!("a={a:e}");

        let residue: f64 = structure_fun
            .iter()
            // .filter(|(r, _)| *r < 4.)
            .map(|(r, sf)| sf - TurbulenceModel::new(a, 0., c).first_order_structure_function(*r))
            .map(|x| x * x)
            .sum();
        dbg!(residue);

        let cost = Problem {
            data: structure_fun.iter().map(|(r, sf)| (*r, *sf / a)).collect(),
            a: 1.,
            c,
        };
        let init_param: DVector<f64> = DVector::from_vec(vec![1. / 50.]);
        let solver: GaussNewton<f64> = GaussNewton::new();
        let res = Executor::new(cost, solver)
            .configure(|state| state.param(init_param).max_iters(50))
            // .add_observer(SlogLogger::term(), ObserverMode::Always)
            .run()?;
        println!("{}", res);
        dbg!(res.state().best_cost);
        dbg!(&res.state().best_param);
        let b = res.state.best_param.unwrap()[0].abs();
        dbg!(b.recip());

        let residue: f64 = structure_fun
            .iter()
            .map(|(r, sf)| sf - TurbulenceModel::new(a, b, c).structure_function(*r))
            .map(|x| x * x)
            .sum();
        dbg!(residue);
        results.insert(cfd_case.to_string(), (structure_fun, fit, (a, b, c)));
    }

    serde_pickle::to_writer(
        &mut File::create("sf_mean.pkl")?,
        &results,
        Default::default(),
    )?;

    Ok(())
}
