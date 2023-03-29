use crseo::{
    wavefrontsensor::PhaseSensor, Atmosphere, FromBuilder,  SegmentWiseSensorBuilder,
    WavefrontSensorBuilder,
};
use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{Tick, Timer};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_crseo::{SegmentPiston, SegmentWfeRms, WfeRms};
use ngao::LittleOpticalModel;
use std::{env, path::Path};

const HDFS: usize = 800;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    let data_repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    dbg!(&data_repo);
    env::set_var("DATA_REPO", &data_repo);
    env::set_var("GMT_MODES_PATH", &data_repo);

    let sampling_frequency = 8_000usize; // Hz
    let sim_duration = 1usize;
    let n_sample = HDFS * 10; // sim_duration * sampling_frequency;

    let n_lenslet = 92;
    let builder = PhaseSensor::builder()
        .lenslet(n_lenslet, 4)
        .wrapping(760e-9 * 0.5);
    let src_builder = builder.guide_stars(None);

    let gom = LittleOpticalModel::builder()
        .source(src_builder)
        .atmosphere(
            Atmosphere::builder().ray_tracing(
                25.5,
                builder.pupil_sampling() as i32,
                0f32,
                sim_duration as f32,
                Some(
                    data_repo
                        .join("ngao_atmophere.bin")
                        .to_str()
                        .unwrap()
                        .to_string(),
                ),
                None,
            ),
        )
        .sampling_frequency(sampling_frequency as f64)
        .build()?
        .into_arcx();

    let mut gom_act: Actor<_> = Actor::new(gom.clone()).name("GS>>(GMT+ATM)");
    let mut timer: Initiator<_> = Timer::new(n_sample).into();

    let logging = Arrow::builder(n_sample)
        .filename("open-loop.parquet")
        .build()
        .into_arcx();
    let mut logger: Terminator<_> = Actor::new(logging.clone());

    timer
        .add_output()
        .build::<Tick>()
        .into_input(&mut gom_act)?;

    gom_act
        .add_output()
        .unbounded()
        .build::<WfeRms>()
        .log(&mut logger)
        .await?;
    gom_act
        .add_output()
        .unbounded()
        .build::<SegmentWfeRms>()
        .log(&mut logger)
        .await?;
    gom_act
        .add_output()
        .unbounded()
        .build::<SegmentPiston>()
        .log(&mut logger)
        .await?;

    model!(timer, gom_act, logger)
        .name("OPEN-LOOP")
        .flowchart()
        .check()?
        .run()
        .await?;

    Ok(())
}
