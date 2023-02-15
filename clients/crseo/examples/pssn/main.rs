use std::sync::{Arc, Mutex};

use crseo::{Atmosphere, FromBuilder, Fwhm};
use dos_actors::prelude::*;
use gmt_dos_clients_ceo::{
    OpticalModel, OpticalModelOptions, PSSn, PSSnOptions, Wavefront, WfeRms,
};
use skyangle::Conversion;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var(
        "DATA_REPO",
        std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR")?)
            .join("examples")
            .join("pssn"),
    );

    let progress = Arc::new(Mutex::new(linya::Progress::new()));
    let n_sample = 300_000;
    let atm_n_duration = Some(46);
    let atm_sampling_frequency = 1e3;

    let (atm_model, atm_om) = {
        let atm_duration = 20f32;
        let atm_sampling = 48 * 16 + 1;
        let atm = Atmosphere::builder().ray_tracing(
            25.5,
            atm_sampling,
            20f32.from_arcmin(),
            atm_duration,
            Some("/fsx/atmosphere/atm_15mn.bin".to_owned()),
            atm_n_duration,
        );
        let atm_om = OpticalModel::builder()
            .source(crseo::Source::builder().pupil_sampling(atm_sampling as usize))
            .options(vec![
                OpticalModelOptions::Atmosphere {
                    builder: atm,
                    time_step: (atm_sampling_frequency as f64).recip(),
                },
                OpticalModelOptions::PSSn(PSSnOptions::AtmosphereTelescope(crseo::PSSn::builder())),
            ])
            .build()?
            .into_arcx();
        let mut om: Terminator<_> = Actor::new(atm_om.clone()).name("Optical Model");

        let mut timer: Initiator<_> = Timer::new(n_sample).progress_with(progress.clone()).into();

        timer.add_output().build::<Tick>().into_input(&mut om);

        (
            Model::new(vec![Box::new(timer), Box::new(om)])
                .name("pssn")
                .check()?
                .flowchart()
                .run(),
            atm_om,
        )

        /*         let logs = &*logging.lock().await;
        println!("{}", logs);

        for data in logs.chunks().last().unwrap() {
            println!("{:?}", data);
        } */
    };
    let (full_model, full_om) = {
        let atm_duration = 20f32;
        let atm_sampling = 48 * 16 + 1;
        let atm = Atmosphere::builder()
            .ray_tracing(
                25.5,
                atm_sampling,
                20f32.from_arcmin(),
                atm_duration,
                Some("/fsx/atmosphere/free_atm_15mn.bin".to_owned()),
                atm_n_duration,
            )
            .remove_turbulence_layer(0);
        let free_atm = OpticalModelOptions::Atmosphere {
            builder: atm,
            time_step: (atm_sampling_frequency as f64).recip(),
        };
        let dome_seeing = OpticalModelOptions::DomeSeeing {
            cfd_case: "/fsx/CASES/zen30az000_OS7".to_string(),
            upsampling_rate: atm_sampling_frequency as usize / 5,
        };
        let pssn =
            OpticalModelOptions::PSSn(PSSnOptions::AtmosphereTelescope(crseo::PSSn::builder()));
        let full_om = OpticalModel::builder()
            .source(crseo::Source::builder().pupil_sampling(atm_sampling as usize))
            .options(vec![free_atm, dome_seeing, pssn])
            .build()?
            .into_arcx();
        let mut om: Terminator<_> = Actor::new(full_om.clone()).name("Optical Model");

        let mut timer: Initiator<_> = Timer::new(n_sample).progress_with(progress.clone()).into();

        timer.add_output().build::<Tick>().into_input(&mut om);

        (
            Model::new(vec![Box::new(timer), Box::new(om)])
                .name("pssn")
                .check()?
                .flowchart()
                .run(),
            full_om,
        )
    };

    atm_model.wait().await?;
    full_model.wait().await?;

    let atm_data = {
        let om = &mut *atm_om.lock().await;
        let mut data = om.src.wfe_rms_10e(-6);
        let mut fwhm = Fwhm::new();
        fwhm.build(&mut om.src);
        om.pssn
            .as_mut()
            .map(|pssn| {
                let (w, r0, ll0) = pssn.atmosphere();
                (
                    pssn.estimates(),
                    fwhm.from_complex_otf(&pssn.otf()),
                    Fwhm::atmosphere(w, r0, ll0),
                )
            })
            .map(|(mut x, y, z)| {
                println!("Atmosphere seeing: {:.4}", z.to_arcsec());
                data.append(&mut x);
                data.append(&mut y.to_arcsec());
            });
        data
        //logs.chunks().last().unwrap().to_vec()
    };
    let full_data = {
        let om = &mut *full_om.lock().await;
        let mut data = om.src.wfe_rms_10e(-6);
        let mut fwhm = Fwhm::new();
        fwhm.build(&mut om.src);
        om.pssn
            .as_mut()
            .map(|pssn| (pssn.estimates(), fwhm.from_complex_otf(&pssn.otf())))
            .map(|(mut x, y)| {
                data.append(&mut x);
                data.append(&mut y.to_arcsec());
            });
        data
    };

    println!("{:.4?}", atm_data);
    println!("{:.4?}", full_data);

    Ok(())
}
