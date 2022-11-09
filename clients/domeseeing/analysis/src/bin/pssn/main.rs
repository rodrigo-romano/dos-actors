use std::{f32::consts::PI, fs::File};

use crseo::{pssn, Builder, FromBuilder};
// use domeseeing_analysis::{PhaseMap, TurbulenceModel};
use skyangle::Conversion;

fn main() -> anyhow::Result<()> {
    std::env::set_var("GMT_MODES_PATH", "/fsx/ceo");

    /*     let n_xy = 104;
    let n_sample = 1000;
    let delta = 0.25;
    let diameter = delta * (n_xy - 1) as f64;
    println!("D: {diameter}m");

    let n_xy2 = n_xy * n_xy;
    let mut xy: Vec<(f64, f64)> = Vec::with_capacity(n_xy2);
    for i in 0..n_xy {
        for j in 0..n_xy {
            let x = (i as f64) * delta;
            let y = (j as f64) * delta;
            xy.push((x, y));
        }
    }

    let mut atm = crseo::Atmosphere::builder()
        .r0_at_zenith(0.5)
        .oscale(50.)
        .single_turbulence_layer(0., None, None)
        .build()?;
    let mut src = crseo::Source::builder()
        .pupil_sampling(n_xy)
        .pupil_size(diameter)
        .build()?;

    let (x, y): (Vec<_>, Vec<_>) = xy.iter().cloned().unzip();
    let opd: Vec<Vec<f64>> = (0..n_sample)
        .map(|_| {
            atm.reset();
            atm.get_phase_values(&mut src, x.as_slice(), y.as_slice(), 0.)
        })
        .collect();
    serde_pickle::to_writer(
        &mut File::create("von-karman_dome-seeing_104.pkl")?,
        &opd,
        Default::default(),
    )?; */

    let n_px = 512;
    let mut gmt = crseo::Gmt::builder().build()?;
    let src_builder = crseo::Source::builder().pupil_sampling(n_px);
    let mut pssn = crseo::PSSn::<pssn::TelescopeError>::builder()
        .source(src_builder.clone())
        .build()?;
    let mut src = src_builder.build()?;

    /*     let rays_xyz = src.rays().coordinates();
    serde_pickle::to_writer(
        &mut File::create("rays_xyz_512.pkl")?,
        &rays_xyz,
        Default::default(),
    )?; */

    let interp_opds: Vec<Vec<f64>> = serde_pickle::from_reader(
        &mut File::open("von-karman_dome-seeing_104-512.pkl")?,
        Default::default(),
    )?;
    dbg!((interp_opds.len(), interp_opds[0].len()));

    src.through(&mut gmt).xpupil().through(&mut pssn);
    dbg!(&pssn.peek().estimates);
    pssn.reset();

    interp_opds.iter().for_each(|opd| {
        src.through(&mut gmt).xpupil().add(opd).through(&mut pssn);
    });
    dbg!(&pssn.peek().estimates);
    let mut fwhm = crseo::Fwhm::new();
    fwhm.build(&mut src);

    let e_fwhm = fwhm.from_complex_otf(&pssn.telescope_error_otf())[0].to_mas();
    dbg!(e_fwhm);

    let mut atm = crseo::Atmosphere::builder()
        .r0_at_zenith(0.5)
        .oscale(50.)
        .single_turbulence_layer(0., None, None)
        .build()?;

    pssn.reset();
    for _ in 0..200 {
        atm.reset();
        src.through(&mut gmt)
            .xpupil()
            .through(&mut atm)
            .through(&mut pssn);
    }
    dbg!(&pssn.peek().estimates);
    let mut fwhm = crseo::Fwhm::new();
    fwhm.build(&mut src);

    let e_fwhm = fwhm.from_complex_otf(&pssn.telescope_error_otf())[0].to_mas();
    dbg!(e_fwhm);
    /* src.through(&mut gmt).xpupil().through(&mut pssn);
       dbg!(&pssn.peek().estimates);
       pssn.reset();

       let mut atm = crseo::Atmosphere::builder()
           .r0_at_zenith(0.5)
           .oscale(50.)
           .single_turbulence_layer(0., None, None)
           .build()?;

       for _ in 0..200 {
           atm.reset();
           src.through(&mut gmt)
               .xpupil()
               .through(&mut atm)
               .through(&mut pssn);
       }

       dbg!(&pssn.peek().estimates);
       let mut fwhm = crseo::Fwhm::new();
       fwhm.build(&mut src);

       let e_fwhm = fwhm.from_complex_otf(&pssn.telescope_error_otf())[0].to_mas();
       dbg!(e_fwhm);

       let phase: Vec<_> = src
           .phase()
           .iter()
           .map(|x| 1e9 * 0.5 * x * 0.5e-6 / PI)
           .collect();
       let _: complot::Heatmap = ((phase.as_slice(), (n_px, n_px)), None).into();
    */
    Ok(())
}
