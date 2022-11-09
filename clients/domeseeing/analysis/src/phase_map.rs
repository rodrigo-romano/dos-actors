use crate::TurbulenceModel;
use nanorand::{Rng, WyRand};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

pub struct PhaseMapBuilder<'a> {
    small_scale: f64,
    large_scale: f64,
    delta: f64,
    turbulence: &'a TurbulenceModel,
}
impl<'a> PhaseMapBuilder<'a> {
    pub fn new(turbulence: &'a TurbulenceModel) -> Self {
        Self {
            small_scale: 1e-3,
            large_scale: f64::max(1e3, 3. * turbulence.b.recip()),
            delta: 5. / 100.,
            turbulence,
        }
    }
    pub fn build(self) -> PhaseMap<'a> {
        let f = self.large_scale / self.small_scale;
        let n_k = f.ln() / ((2. + self.delta) / (2. - self.delta)).ln();
        let f_1_n_k = f.powf(n_k.recip());
        let n_a = (0.25 * PI * ((f_1_n_k + 1.) / (f_1_n_k - 1.))).ceil();
        let n_k = n_k.ceil();
        let f = ((4. * n_a + PI) / (4. * n_a - PI)).powf(n_k);
        let delta = 0.5 * PI / n_a;

        let n = (n_k * n_a) as usize;
        let mut rng = WyRand::new();
        let variates: Vec<_> = (0..n)
            .map(|_| {
                (
                    (-(rng.generate::<f64>().ln())).sqrt(),
                    2. * PI * rng.generate::<f64>(),
                    (-(rng.generate::<f64>().ln())).sqrt(),
                    2. * PI * rng.generate::<f64>(),
                )
            })
            .collect();
        PhaseMap {
            turbulence: self.turbulence,
            variates,
            kmin: 2. * PI * self.large_scale.recip(),
            f,
            delta,
            n_k: n_k as usize,
            n_a: n_a as usize,
            rng,
        }
    }
}

pub struct PhaseMap<'a> {
    turbulence: &'a TurbulenceModel,
    variates: Vec<(f64, f64, f64, f64)>,
    kmin: f64,
    f: f64,
    delta: f64,
    n_k: usize,
    n_a: usize,
    rng: WyRand,
}
impl<'a> PhaseMap<'a> {
    pub fn builder(turbulence: &'a TurbulenceModel) -> PhaseMapBuilder {
        PhaseMapBuilder::new(turbulence)
    }
    #[inline]
    fn get(
        &self,
        x: &f64,
        y: &f64,
        f_red0: f64,
        freq_mag0: f64,
        delta_freq_mag0: f64,
        f02: f64,
        a: f64,
        c: f64,
    ) -> f64 {
        let mut f_red = 1f64;
        let mut iter_variates = self.variates.iter();
        (0..self.n_k).fold(0f64, |mut sum, _| {
            f_red *= f_red0;
            let freq_mag = freq_mag0 * f_red;
            let delta_freq_mag = delta_freq_mag0 * f_red;
            let sqrt_spectrum_kernel = (freq_mag.powi(2) + f02).powf(-0.5 * c)
                * (freq_mag * delta_freq_mag * self.delta).sqrt();
            (0..self.n_a).for_each(|j| {
                let freq_ang = (j as f64 + 0.5) * self.delta;
                let (sin_freq_ang, cos_freq_ang) = freq_ang.sin_cos();
                let &(zeta1, eta1, zeta2, eta2) = iter_variates.next().unwrap();
                sum += ((eta1 + freq_mag * (x * cos_freq_ang + y * sin_freq_ang)).cos() * zeta1
                    + (eta2 - freq_mag * (x * sin_freq_ang - y * cos_freq_ang)).cos() * zeta2)
                    * sqrt_spectrum_kernel;
            });
            sum
        }) * a
    }
    #[inline]
    fn init(&self) -> (f64, f64, f64, f64, f64, f64) {
        let &TurbulenceModel { a, b, c } = self.turbulence;
        let f_red0 = self.f.powf((self.n_k as f64).recip());
        let freq_mag0 = 0.5 * self.kmin * (f_red0 + 1.) / f_red0;
        let delta_freq_mag0 = self.kmin * (f_red0 - 1.) / f_red0;
        let f02 = (2. * PI * b).powi(2);
        let a = a.sqrt() * (2. * PI).powf(c - 1.) * 2.;
        (f_red0, freq_mag0, delta_freq_mag0, f02, a, c)
    }
    pub fn map(&self, x: &[f64], y: &[f64]) -> Vec<f64> {
        assert_eq!(x.len(), y.len());
        let (f_red0, freq_mag0, delta_freq_mag0, f02, a, c) = self.init();
        let mut data: Vec<f64> = Vec::with_capacity(x.len());
        x.par_iter()
            .zip(y)
            .map(|(x, y)| self.get(x, y, f_red0, freq_mag0, delta_freq_mag0, f02, a, c))
            .collect_into_vec(&mut data);
        data
    }
    pub fn square_map(&self, delta: f64, n: usize) -> Vec<f64> {
        let (f_red0, freq_mag0, delta_freq_mag0, f02, a, c) = self.init();
        let mut data: Vec<f64> = Vec::with_capacity(n * n);
        (0..n * n)
            .into_par_iter()
            .map(|k| {
                let i = k / n;
                let j = k % n;
                let x = i as f64 * delta;
                let y = j as f64 * delta;
                self.get(&x, &y, f_red0, freq_mag0, delta_freq_mag0, f02, a, c)
            })
            .collect_into_vec(&mut data);
        data
    }
    pub fn square_map_n<T>(&mut self, delta: f64, n: usize, n_map: usize) -> T
    where
        T: From<Map>,
    {
        let n2 = n * n;
        let (f_red0, freq_mag0, delta_freq_mag0, f02, a, c) = self.init();
        let mut data: Vec<f64> = Vec::with_capacity(n2 * n_map);
        let mut data_i: Vec<f64> = Vec::with_capacity(n2);
        data.extend((0..n_map).into_iter().flat_map(|_| {
            self.reset();
            (0..n2)
                .into_par_iter()
                .map(|k| {
                    let i = k / n;
                    let j = k % n;
                    let x = i as f64 * delta;
                    let y = j as f64 * delta;
                    self.get(&x, &y, f_red0, freq_mag0, delta_freq_mag0, f02, a, c)
                })
                .collect_into_vec(&mut data_i);
            std::mem::take(&mut data_i)
        }));
        let map = Map {
            abc: self.turbulence.into(),
            data,
            n,
            n_map,
        };
        map.into()
    }
    pub fn reset(&mut self) -> &mut Self {
        let n = self.n_k * self.n_a;
        self.variates = (0..n)
            .map(|_| {
                (
                    (-(self.rng.generate::<f64>().ln())).sqrt(),
                    2. * PI * self.rng.generate::<f64>(),
                    (-(self.rng.generate::<f64>().ln())).sqrt(),
                    2. * PI * self.rng.generate::<f64>(),
                )
            })
            .collect();
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
    abc: (f64, f64, f64),
    data: Vec<f64>,
    n: usize,
    n_map: usize,
}
impl From<Map> for Vec<f64> {
    fn from(map: Map) -> Self {
        map.data
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use nanorand::{Rng, WyRand};

    #[test]
    fn rand() {
        let mut rng = WyRand::new();
        for _ in 0..100 {
            let var = rng.generate::<f64>();
            println!("{var}")
        }
    }
    #[test]
    fn variance() {
        let turbulence = TurbulenceModel::von_karman(10e-2, 30.);
        let mut opd = PhaseMap::builder(&turbulence).build();
        let n = 1000;
        let var = Some(
            (0..n)
                .map(|_| opd.reset().map(&[0.], &[0.]).get(0).unwrap().to_owned())
                .fold((0f64, 0f64), |(mut a, mut sa), x| {
                    a += x;
                    sa += x * x;
                    (a, sa)
                }),
        )
        .map(|(a, sa)| sa / n as f64 - (a / n as f64).powi(2))
        .unwrap();
        dbg!(var);
        dbg!(turbulence.variance());
    }
    #[test]
    fn structure_function() {
        let turbulence = TurbulenceModel::von_karman(10e-2, 30.);
        let mut opd = PhaseMap::builder(&turbulence).build();
        let n = 200;
        for r in 1..5 {
            let (a, sa) = (0..n)
                .map(|_| {
                    opd.reset().map(&[0.], &[0.]).get(0).unwrap()
                        - opd.map(&[0.], &[r as f64]).get(0).unwrap()
                })
                .fold((0f64, 0f64), |(mut a, mut sa), x| {
                    a += x;
                    sa += x * x;
                    (a, sa)
                });
            let sf = sa / n as f64 - (a / n as f64).powi(2);
            dbg!(sf);
            dbg!(turbulence.structure_function(r as f64));
        }
    }

    #[test]
    fn square_map() {
        let von_karman = TurbulenceModel::von_karman(75e-2, 50.);
        let (a, b, _) = von_karman.params();
        let turbulence = TurbulenceModel::new(a, b, 1.97);
        let opd = PhaseMap::builder(&turbulence).build();
        let map = opd.square_map(5e-2, 501);
        serde_pickle::to_writer(
            &mut std::fs::File::create("square_map.pkl").unwrap(),
            &map,
            Default::default(),
        )
        .unwrap();
        bincode::serialize_into(&mut std::fs::File::create("square_map.bin").unwrap(), &map)
            .unwrap();
    }

    #[test]
    fn square_map_n() {
        // let von_karman = TurbulenceModel::von_karman(50e-2, 50.);
        // let (a, b, _) = von_karman.params();
        let turbulence = TurbulenceModel::new(5.72e-17, 1. / 50., 1.97);
        let mut opd = PhaseMap::builder(&turbulence).build();
        let map: Map = opd.square_map_n(25e-2 / 8., 103 * 8 + 1, 1000);

        bincode::serialize_into(
            &mut std::fs::File::create("square_map_n.bin").unwrap(),
            &map,
        )
        .unwrap();

        /*         serde_pickle::to_writer(
            &mut std::fs::File::create("square_map_n.pkl").unwrap(),
            &map,
            Default::default(),
        )
        .unwrap(); */
    }
}
