use crseo::{Builder, FromBuilder};
use dos_actors::clients::ceo::Wavefront;
use dos_actors::clients::{arrow_client as arrow, ceo};
use dos_actors::prelude::*;
use std::fs::File;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pwd = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("static_aberration");
    std::env::set_var("DATA_REPO", &pwd);

    let n_mode = 332;
    let mut gmt = crseo::Gmt::builder()
        .m1(
            "m1_eigen-modes_raw-polishing_print-through_soak1deg",
            n_mode,
        )
        .build()?;
    let n_px = 769;
    let mut src = crseo::Source::builder().pupil_sampling(n_px).build()?;
    let a: Vec<_> = (0..7)
        .flat_map(|_| {
            let mut a = vec![0f64; n_mode];
            a[n_mode - 1] = 1f64;
            a[n_mode - 2] = 1f64;
            a[n_mode - 3] = 1f64;
            a
        })
        .collect();
    gmt.m1_modes(a.as_slice());
    src.through(&mut gmt).xpupil();
    println!("WFE RMS: {:.0}nm", src.wfe_rms_10e(-9)[0]);

    let phase = src.phase().to_vec();
    let mut file =
        File::create(pwd.join(format!("raw-polishing_print-through_soak1deg_{n_px}.bin")))?;
    bincode::serialize_into(&mut file, &phase)?;

    let phase: Vec<f32> = bincode::deserialize_from(File::open(
        pwd.join("raw-polishing_print-through_soak1deg_512.bin"),
    )?)?;

    let mut optical_model: Actor<_> = ceo::OpticalModel::builder()
        .options(vec![ceo::OpticalModelOptions::StaticAberration(
            phase.into(),
        )])
        .build()?
        .into();
    let mut logs: Terminator<_> = arrow::Arrow::builder(1).build().into();
    let mut clock: Initiator<_> = Timer::new(1).into();

    clock
        .add_output()
        .build::<Tick>()
        .into_input(&mut optical_model);
    optical_model
        .add_output()
        .build::<Wavefront>()
        .log(&mut logs)
        .await;

    Model::new(vec![
        Box::new(optical_model),
        Box::new(logs),
        Box::new(clock),
    ])
    .check()?
    .run()
    .wait()
    .await?;

    Ok(())
}
