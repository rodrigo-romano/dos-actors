use arrow::Arrow;
use clap::Parser;
use crseo::{
    pssn::{AtmosphereTelescopeError, TelescopeError},
    Atmosphere, FromBuilder, PSSn,
};
use crseo_client::{
    OpticalModel, OpticalModelOptions, PSSnFwhm, PSSnOptions, SegmentPiston, SegmentTipTilt,
    SegmentWfeRms, TipTilt, WfeRms,
};
use dos_actors::{clients::Timer, prelude::*};
use parse_monitors::cfd;
use skyangle::Conversion;
use std::env;
use vec_box::vec_box;

#[derive(Parser)]
struct Cli {
    /// path to CFD case
    #[arg(long)]
    domeseeing: Option<String>,
    /// Atmospheric turbulence: "full" or "free"
    #[arg(long)]
    atmosphere: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("GMT_MODES_PATH", "/fsx/ceo");
    std::env::set_var("CFD_REPO", "/fsx/CASES");

    let cli = Cli::parse();
    let mut options = vec![];
    let mut suffix = String::new();
    if let Ok(job_idx) = env::var("AWS_BATCH_JOB_ARRAY_INDEX") {
        let idx = job_idx
            .parse::<usize>()
            .expect("failed to convert `AWS_BATCH_JOB_ARRAY_INDEX` into usize");
        let cfd_path = cfd::Baseline::<2021>::path();
        let cfd_case = cfd::Baseline::<2021>::default()
            .into_iter()
            .nth(idx)
            .expect(&format!("Failed to load CFD case #{}", idx));
        println!("CFD CASE: {cfd_case}");
        options.push(OpticalModelOptions::DomeSeeing {
            cfd_case: cfd_path
                .join(cfd_case.to_string())
                .to_str()
                .unwrap()
                .to_string(),
            upsampling_rate: 1,
        });
        std::env::set_var("DATA_REPO", cfd_path.join(cfd_case.to_string()));
        suffix += "_domeseeing";
    }
    if let Some(cfd_case) = cli.domeseeing {
        options.push(OpticalModelOptions::DomeSeeing {
            cfd_case,
            upsampling_rate: 1,
        });
        std::env::set_var(
            "DATA_REPO",
            std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR")?),
        );
        suffix += "_domeseeing";
    }
    if let Some(atmosphere_profile) = cli.atmosphere {
        suffix += format!("_{atmosphere_profile}-atmosphere").as_str();
        let atm_duration = 20f32;
        let atm_n_duration = Some((410f64 / atm_duration as f64).ceil() as i32);
        let atm_sampling = 48 * 16 + 1;
        let atm = match atmosphere_profile.to_lowercase().as_str() {
            "full" => Atmosphere::builder().ray_tracing(
                25.5,
                atm_sampling,
                20f32.from_arcmin(),
                atm_duration,
                Some("/fsx/atmosphere/atm_15mn.bin".to_owned()),
                atm_n_duration,
            ),
            "free" => Atmosphere::builder()
                .ray_tracing(
                    25.5,
                    atm_sampling,
                    20f32.from_arcmin(),
                    atm_duration,
                    Some("/fsx/atmosphere/free_atm_15mn.bin".to_owned()),
                    atm_n_duration,
                )
                .remove_turbulence_layer(0),
            _ => unimplemented!(),
        };
        options.push(OpticalModelOptions::Atmosphere {
            builder: atm,
            time_step: 5f64.recip(),
        });
        options.push(OpticalModelOptions::PSSn(PSSnOptions::AtmosphereTelescope(
            PSSn::<AtmosphereTelescopeError>::builder(),
        )));
    } else {
        options.push(OpticalModelOptions::PSSn(PSSnOptions::Telescope(PSSn::<
            TelescopeError,
        >::builder(
        ))));
    };

    let on_axis_gmt = OpticalModel::builder()
        .source(crseo::Source::builder().band(pho).pupil_sampling(769))
        .options(options)
        .build()?
        .into_arcx();

    let mut gmt: Actor<_> = Actor::new(on_axis_gmt.clone()).name("On-axis GMT");
    let n_step = 400 * 5;
    let mut timer: Initiator<_> = Timer::new(n_step).into();
    let mut data_logs: Terminator<_> = Arrow::builder(n_step)
        .filename(format!("stats{suffix}"))
        .build()
        .into();

    timer
        .add_output()
        .unbounded()
        .build::<Tick>()
        .into_input(&mut gmt);
    gmt.add_output()
        .unbounded()
        .build::<WfeRms>()
        .log(&mut data_logs)
        .await;
    gmt.add_output()
        .unbounded()
        .build::<TipTilt>()
        .log(&mut data_logs)
        .await;
    gmt.add_output()
        .unbounded()
        .build::<SegmentWfeRms>()
        .log(&mut data_logs)
        .await;
    gmt.add_output()
        .unbounded()
        .build::<SegmentTipTilt>()
        .log(&mut data_logs)
        .await;
    gmt.add_output()
        .unbounded()
        .build::<SegmentPiston>()
        .log(&mut data_logs)
        .await;

    Model::new(vec_box![timer, gmt, data_logs])
        .inspect()
        .flowchart()
        .check()?
        .run()
        .await?;

    let mut gmt: Actor<_> = Actor::new(on_axis_gmt.clone()).name("On-axis GMT");
    let n_step = 0;
    let mut timer: Initiator<_> = Timer::new(n_step).into();
    let mut data_logs: Terminator<_> = Arrow::builder(n_step)
        .filename(format!("pssn-fwhm{suffix}"))
        .build()
        .into();

    timer.add_output().build::<Tick>().into_input(&mut gmt);
    gmt.add_output()
        .bootstrap()
        .build::<PSSnFwhm>()
        .log(&mut data_logs)
        .await;

    Model::new(vec_box![timer, gmt, data_logs])
        .check()?
        .run()
        .await?;

    Ok(())
}
