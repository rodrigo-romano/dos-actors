use crseo::{calibrations, Builder, Calibration, Geometric, ShackHartmann, SH48, SHACKHARTMANN};
use dos_actors::clients::ceo;
//use dos_actors::prelude::*;
//use nalgebra as na;
//use skyangle::Conversion;
//use std::default::Default;
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // AGWS SH48
    let mut agws_sh48 = ceo::OpticalModel::builder()
        .sensor_builder(ceo::SensorBuilder::new(SH48::<Geometric>::new()))
        .build()?;
    let mirror = vec![calibrations::Mirror::M1];
    let segments = vec![vec![calibrations::Segment::Rxyz(1e-6, Some(0..2))]; 7];
    let mut gmt2wfs = Calibration::new(
        &agws_sh48.gmt,
        &agws_sh48.src,
        SH48::<crseo::Geometric>::new(),
    );
    let now = Instant::now();
    gmt2wfs.calibrate(
        mirror,
        segments,
        calibrations::ValidLensletCriteria::Threshold(Some(0.8)),
    );
    println!(
        "GMT 2 WFS calibration [{}x{}] in {}s",
        gmt2wfs.n_data,
        gmt2wfs.n_mode,
        now.elapsed().as_secs()
    );
    Ok(())
}
