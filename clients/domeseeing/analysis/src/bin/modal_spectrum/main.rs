use crseo::{Atmosphere, Builder, FromBuilder, Source};
use statrs::function::gamma::gamma;

fn main() -> anyhow::Result<()> {
    let n_xy = 104;
    let n_radial_order = 5;
    let n_sample = 1000;
    let diameter = 1.;
    let n_mode = dbg!((n_radial_order * (n_radial_order + 1)) / 2) as usize;

    let mut src = Source::builder()
        .pupil_sampling(n_xy)
        .pupil_size(diameter)
        .build()?;
    let mut atm = Atmosphere::builder().build()?;

    let mut zern = zernike::mode_set(n_radial_order, n_xy);
    let n_xy2 = n_xy * n_xy;
    let zern_norm: Vec<_> = zern
        .chunks(n_xy2)
        .map(|x| x.iter().map(|x| x * x).sum::<f64>())
        .collect();
    zern.chunks_mut(n_xy2)
        .zip(&zern_norm)
        .for_each(|(x, n)| x.iter_mut().for_each(|x| *x /= n));

    for i in 0..n_mode {
        let a_zern: Vec<f64> = zernike::mode_set(n_radial_order, n_xy)
            .chunks(n_xy2)
            .skip(i)
            .take(1)
            .flat_map(|x| x.to_owned())
            .collect();
        zern.chunks(n_xy2).for_each(|x| {
            let a = x.iter().zip(&a_zern).map(|(x, y)| x * y).sum::<f64>();
            print!(" {:+.3} ", a);
        });
        println!("");
    }

    let zern_coefs: Vec<_> = (0..n_sample)
        .flat_map(|_| {
            src.through(&mut atm);
            let phase: Vec<f64> = src.phase().iter().map(|&x| x as f64).collect();
            atm.reset();
            zern.chunks(n_xy2)
                .map(|x| x.iter().zip(&phase).map(|(x, y)| x * y).sum::<f64>())
                .collect::<Vec<f64>>()
        })
        .collect();

    /*         let zern_coefs: Vec<_> = (0..n_sample)
    .flat_map(|_| {
        src.through(&mut atm);
        let phase: Vec<f64> = src.phase().iter().map(|&x| x as f64).collect();
        atm.reset();
        zernike::projection(&phase, n_radial_order, n_xy)
    })
    .collect(); */

    dbg!(zern_coefs.len());
    let n = n_sample as f64;
    let zern_var: Vec<f64> = (0..n_mode)
        .map(|i| {
            let b_mean = zern_coefs.iter().skip(i).step_by(n_mode).sum::<f64>() / n;
            zern_coefs
                .iter()
                .skip(i)
                .step_by(n_mode)
                .map(|b| b - b_mean)
                .map(|x| x * x)
                .sum::<f64>()
                / n
        })
        .collect();

    let (j, n, m) = zernike::jnm(n_radial_order);
    let secz = 1f64 / atm.zenith_angle.cos();
    let r0 = (atm.r0_at_zenith.powf(-5.0 / 3.0) * secz).powf(-3.0 / 5.0);
    dbg!(r0);
    let mut c =
        gamma(14. / 3.) * gamma(11. / 6.).powf(2.) * (24. * gamma(6. / 5.) / 5.).powf(5. / 6.)
            / (2f64.powf(8. / 3.) * std::f64::consts::PI);
    dbg!(c);
    c /= gamma(17. / 6.).powf(2.);
    c *= (diameter / r0).powf(5. / 3.);

    let k2 = (2. * std::f64::consts::PI / 0.5e-6).powf(2.);
    j.into_iter()
        .zip(n.into_iter())
        .zip(m.into_iter())
        .zip(&zern_var)
        .skip(1)
        .for_each(|(((j, n), m), v)| {
            let nf = n as f64;
            let u = c * (nf + 1.) * gamma(nf - 5. / 6.) / gamma(nf + 23. / 6.);
            println!("{:>2} {} {} {:.3e} {:.3e}", j, n, m, k2 * v, u);
        });

    Ok(())
}
