use std::{f64::consts::FRAC_PI_2, iter};

use crate::ku;
use statrs::function::gamma::gamma;

const PI: f64 = std::f64::consts::PI;

pub fn a2r0(a: f64, lambda: f64) -> f64 {
    ((a * (2. * PI / lambda).powi(2))
        * ((gamma(11. / 6.).powi(2) / (2. * PI.powf(11. / 3.)))
            * (24. * gamma(6. / 5.) / 5.).powf(5. / 6.))
        .recip())
    .powf(-3. / 5.)
}
#[derive(Debug, Clone)]
pub struct TurbulenceModel {
    /// Power spectrum scaling factor
    pub(crate) a: f64,
    /// Power spectrum zero frequency
    pub(crate) b: f64,
    /// Power spectrum power
    pub(crate) c: f64,
}
impl<'a> From<&'a TurbulenceModel> for (f64, f64, f64) {
    fn from(turbulence: &'a TurbulenceModel) -> Self {
        let &TurbulenceModel { a, b, c } = turbulence;
        (a, b, c)
    }
}
impl TurbulenceModel {
    pub fn new(a: f64, b: f64, c: f64) -> Self {
        Self { a, b, c }
    }
    pub fn params(&self) -> (f64, f64, f64) {
        (self.a, self.b, self.c)
    }
    pub fn kolmogorov(fried_parameter: f64) -> Self {
        let a = (gamma(11. / 6.).powi(2) / (2. * PI.powf(11. / 3.)))
            * (24. * gamma(6. / 5.) / 5.).powf(5. / 6.)
            * fried_parameter.powf(-5. / 3.);
        Self::new(a, 0., 11. / 6.)
    }
    pub fn von_karman(fried_parameter: f64, outer_scale: f64) -> Self {
        let Self { a, b: _, c } = Self::kolmogorov(fried_parameter);
        let b = outer_scale.recip();
        Self::new(a, b, c)
    }
    pub fn first_order_structure_function(&self, r: f64) -> f64 {
        let &Self { a, b: _, c } = self;
        let denom = gamma(c).powf(2f64) * (PI * c).sin();
        let num = -2f64 * PI.powi(2) * a;
        (num / denom) * (PI * r).powf(2. * (c - 1.))
    }
    pub fn variance(&self) -> f64 {
        let &Self { a, b, c } = self;
        PI * a * b.powf(-2. * (c - 1.)) / (c - 1.)
    }
    pub fn covariance(&self, r: f64) -> f64 {
        if r == 0. {
            self.variance()
        } else {
            let &Self { a, b, c } = self;
            let red = 2. * PI * b * r.abs();
            (2. * PI * a / gamma(c))
                * (b.powf(-2. * (c - 1.)) / 2f64.powf(c - 1.))
                * red.powf(c - 1.)
                * ku(c - 1., red)
        }
    }
    pub fn structure_function(&self, r: f64) -> f64 {
        2. * (self.variance() - self.covariance(r))
    }
    pub fn a_partial_structure_function(&self, r: f64) -> f64 {
        let &Self { a: _, b, c } = self;
        Self { a: 1f64, b, c }.structure_function(r)
    }
    pub fn b_partial_structure_function(&self, r: f64) -> f64 {
        let &Self { a, b, c } = self;
        Self {
            a: -2f64 * a * b * c,
            b,
            c: c + 1f64,
        }
        .structure_function(r)
    }
    pub fn fwhm_polynomials(&self, o: usize) -> Vec<f64> {
        let r = 2f64 / (2f64 * (self.c - 1f64));
        let mut p: Vec<f64> = iter::once(0.5 * gamma(6. / 5.))
            .chain((1..=o).map(|n| {
                (-1f64).powi(n as i32) * gamma(r * (n + 1) as f64) / gamma((n + 1) as f64).powi(2)
            }))
            .rev()
            .collect();
        let ipn = p[0].recip();
        p.iter_mut().for_each(|p| *p *= ipn);
        p
    }
    pub fn equivalent_diameter(&self) -> Option<f64> {
        let &Self { a, b: _, c } = self;
        let q = 2. * (c - 1.);
        let f = FRAC_PI_2
            * (-(2. * PI.powf(2. * c) * a) / (2. * gamma(c).powi(2) * (PI * c).sin()))
                .powf(-q.recip());

        let pn = self.fwhm_polynomials(10);
        let roots: Vec<_> = roots::find_roots_sturm(pn.as_slice(), &mut 1e-6);
        let root = roots.into_iter().filter_map(|x| x.ok()).find(|x| *x > 0f64);
        root.map(|r| f / r.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roots;

    #[test]
    fn kolmogorov() {
        let tm = TurbulenceModel::kolmogorov(15e-2);
        let sf = tm.first_order_structure_function(15e-2);
        // dbg!(sf);
        assert!((sf - 6.8838771822938).abs() < 1e-8);
    }

    #[test]
    fn von_karman_variance() {
        let tm = TurbulenceModel::von_karman(15e-2, 25.);
        let var = tm.variance();
        let vk_var = (gamma(11. / 6.) * gamma(5. / 6.) / (2. * PI.powf(8. / 3.)))
            * (24. * gamma(6. / 5.) / 5.).powf(5. / 6.)
            * (25f64 / 15e-2).powf(5. / 3.);
        // dbg!(var);
        // dbg!(vk_var);
        assert!((var - vk_var).abs() < 1e-8);
    }

    #[test]
    fn von_karman_structure_function() {
        let tm = TurbulenceModel::von_karman(15e-2, 25.);
        let sf = tm.structure_function(100f64);
        let sf_asymptote = 2. * tm.variance();
        // dbg!(sf);
        // dbg!(sf_asymptote);
        assert!((sf - sf_asymptote).abs() < 1e-6);
    }

    #[test]
    fn delta_structure_function() {
        let tm = TurbulenceModel::von_karman(10e-2, 50.);
        let r = 1.;
        let d_sf = tm.structure_function(r) - tm.first_order_structure_function(r);
        dbg!(d_sf);
    }

    #[test]
    fn poly() {
        let tm = TurbulenceModel::kolmogorov(15e-2);
        for o in 1..=10 {
            println!("{:02}: {:7.5?}", o, tm.fwhm_polynomials(o));
        }
    }

    #[test]
    fn polyroots_sturm() {
        let tm = TurbulenceModel::kolmogorov(15e-2);
        let max_o = 12;
        for o in 1..=max_o {
            let pn = tm.fwhm_polynomials(o);
            let roots: Vec<_> = roots::find_roots_sturm(pn.as_slice(), &mut 1e-6);
            println!("{:02}: {:7.5?}", o, roots);
        }
        let pn = tm.fwhm_polynomials(max_o);
        let roots: Vec<_> = roots::find_roots_sturm(pn.as_slice(), &mut 1e-6);
        let r = roots
            .into_iter()
            .filter_map(|x| x.ok())
            .find(|x| *x > Default::default());
        dbg!(r);
    }

    #[test]
    fn equiv_diam() {
        let r0 = 15e-2;
        let tm = TurbulenceModel::kolmogorov(r0);
        let equiv_diam = tm.equivalent_diameter();
        dbg!(equiv_diam);
        assert!((equiv_diam.unwrap() - r0 / 0.9759).abs() < 1e-5);
    }

    #[test]
    fn equiv_diam1() {
        let lambda = 0.5e-6;
        let k2 = (2. * PI / lambda).powi(2);
        let tm = TurbulenceModel::new(k2 * 1.4e-16, 0f64, 1.95);
        let equiv_diam = tm.equivalent_diameter();
        dbg!(equiv_diam);
    }
}
