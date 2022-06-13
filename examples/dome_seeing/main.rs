use crseo::FromBuilder;
use dos_actors::clients::arrow_client::Arrow;
use dos_actors::clients::ceo::{OpticalModel, OpticalModelOptions, Wavefront, WfeRms};
use dos_actors::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut dome_seeing: Actor<_> = (
        OpticalModel::builder()
            .source(crseo::Source::builder().pupil_sampling(769))
            .options(vec![OpticalModelOptions::DomeSeeing {
                cfd_case: "/fsx/CASES/zen30az000_OS7/".to_string(),
                upsampling_rate: 10,
            }])
            .build()?,
        "Dome Seeing",
    )
        .into();
    let mut sink: Terminator<_> = Arrow::builder(11).build().into();
    let mut timer: Initiator<_> = Timer::new(11).into();

    timer
        .add_output()
        .build::<Tick>()
        .into_input(&mut dome_seeing);
    dome_seeing
        .add_output()
        .build::<WfeRms>()
        .log(&mut sink)
        .await;
    dome_seeing
        .add_output()
        .build::<Wavefront>()
        .log(&mut sink)
        .await;

    Model::new(vec![Box::new(timer), Box::new(dome_seeing), Box::new(sink)])
        .name("dome_seeing")
        .check()?
        .flowchart()
        .run()
        .wait()
        .await?;

    Ok(())
}
