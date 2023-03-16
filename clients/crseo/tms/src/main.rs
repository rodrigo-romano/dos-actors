use crseo::FromBuilder;
use gmt_dos_actors::prelude::*;
use gmt_dos_clients::interface::{Data, TimerMarker, Update, Write};
use gmt_dos_clients::{Signal, Signals, Tick, Timer};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_crseo::{OpticalModel, PointingError, SegmentTipTilt, TipTilt};
use gmt_dos_clients_io::{gmt_m1::M1RigidBodyMotions, gmt_m2::M2RigidBodyMotions};
use rand::{rngs::StdRng, SeedableRng};
use std::sync::Arc;
// use std::{fs::File, io::Write};
use polars::prelude::*;
use rand_distr::{Distribution, Normal, Uniform};
use skyangle::Conversion;

const PUPIL_SAMPLING: usize = 201;
const N_SAMPLE: usize = 10_000;
const MOUNT_1SIGMA: f64 = 0f64; //1.4108078120287496e-05; // 2.91arcsec

pub struct PointingErrorRandomGenerator {
    rng: StdRng,
    normal: Normal<f64>,
    uniform: Uniform<f64>,
}
impl PointingErrorRandomGenerator {
    pub fn new(sigma: f64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(123456),
            normal: Normal::new(0f64, sigma).expect("Failed to create a Normal distribution"),
            uniform: Uniform::new(0f64, 2. * std::f64::consts::PI),
        }
    }
}
impl Update for PointingErrorRandomGenerator {}
impl Write<PointingError> for PointingErrorRandomGenerator {
    fn write(&mut self) -> Option<Arc<Data<PointingError>>> {
        let zen = self.normal.sample(&mut self.rng);
        let az = self.uniform.sample(&mut self.rng);
        Some(Arc::new(Data::new((zen, az))))
    }
}
impl TimerMarker for PointingErrorRandomGenerator {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let gomb =
        OpticalModel::builder().source(crseo::Source::builder().pupil_sampling(PUPIL_SAMPLING));
    // let toml = toml::to_string(&gomb).unwrap();
    // let mut file = File::create("optical_model.toml")?;
    // write!(file, "{}", toml)?;
    let gom = gomb.build()?.into_arcx();

    let mut timer: Initiator<_> = Timer::new(N_SAMPLE).into();
    let mut pointing_error: Actor<_> = (
        PointingErrorRandomGenerator::new(MOUNT_1SIGMA),
        "Pointing Error
    RNG",
    )
        .into();

    let m1_tms_req: Vec<_> = vec![vec![75., 75., 4.3, 1.7, 1.7, 190.]; 7];
    let mut m1_rbm: Initiator<_> = (
        m1_tms_req
            .into_iter()
            .flatten()
            .enumerate()
            .fold(Signals::new(6 * 7, N_SAMPLE), |s, (i, v)| {
                s.channel(i, Signal::WhiteNoise(Normal::new(0f64, 0. * 1e-6).unwrap()))
            }),
        "M1 TMS Error",
    )
        .into();

    let m2_tms_req: Vec<_> = vec![vec![75., 75., 4.3, 15., 15., 1600.]; 7];
    let mut m2_rbm: Initiator<_> = (
        m2_tms_req
            .into_iter()
            .flatten()
            .enumerate()
            .fold(Signals::new(6 * 7, N_SAMPLE), |s, (i, v)| {
                s.channel(i, Signal::WhiteNoise(Normal::new(0f64, 0. * 1e-6).unwrap()))
            }),
        "M2 TMS Error",
    )
        .into();

    let data = Arrow::builder(N_SAMPLE).build().into_arcx();
    let mut data_log = Terminator::<_>::new(data.clone());
    let mut agom = Actor::<_>::new(gom.clone());

    timer
        .add_output()
        .build::<Tick>()
        .into_input(&mut pointing_error)?;
    pointing_error
        .add_output()
        .build::<PointingError>()
        .into_input(&mut agom)?;
    m1_rbm
        .add_output()
        .build::<M1RigidBodyMotions>()
        .into_input(&mut agom)?;
    m2_rbm
        .add_output()
        .build::<M2RigidBodyMotions>()
        .into_input(&mut agom)?;
    agom.add_output()
        .build::<TipTilt>()
        .log(&mut data_log)
        .await?;
    agom.add_output()
        .build::<SegmentTipTilt>()
        .log(&mut data_log)
        .await?;
    model!(timer, pointing_error, m1_rbm, m2_rbm, agom, data_log)
        .name("TMS")
        .flowchart()
        .check()?
        .run()
        .await?;

    let mut file = std::fs::File::open("data.parquet")?;

    let df = ParquetReader::new(&mut file).finish()?;

    println!("{}", df.head(None));

    /*     let fun  = |col: &Series| {
        col.into_iter()
    }; */

    /*     let stt = df.apply("SegmentTipTilt", |v| {
        v.list()
            .unwrap()
            .apply(|v| dbg!(v).f64().unwrap().apply(|v| v.to_arcsec()).into())
    });
    // let q: Vec<f64> = stt.f64().iter().map(|x| x.apply(|v| v * 2.)).collect();
    println!("{:?}", stt); */

    let out = df
        .clone()
        .lazy()
        .select([
            col("TipTilt")
                .arr()
                .get(Expr::from(0))
                .alias("X Mean")
                .mean()
                * 1f64.to_arcsec().into(),
            col("TipTilt")
                .arr()
                .get(Expr::from(1))
                .alias("Y Mean")
                .mean()
                * 1f64.to_arcsec().into(),
            col("TipTilt")
                .arr()
                .get(Expr::from(0))
                .alias("X Std")
                .std(0)
                * 1f64.to_arcsec().into(),
            col("TipTilt")
                .arr()
                .get(Expr::from(1))
                .alias("Y Std")
                .std(0)
                * 1f64.to_arcsec().into(),
        ])
        .collect()?;
    println!("{out}");

    let out = df
        .lazy()
        .select([
            col("SegmentTipTilt")
                .arr()
                .get(Expr::from(0))
                .alias("S1 X Std.")
                .std(0)
                * 1f64.to_arcsec().into(),
            col("SegmentTipTilt")
                .arr()
                .get(Expr::from(1))
                .alias("S2 X Std.")
                .std(0)
                * 1f64.to_arcsec().into(),
        ])
        .collect()?;
    println!("{out}");

    /*     df.apply("TipTilt", |v| {
        v.list()
            .unwrap()
            .apply(|v| {
                let x = v.head(Some(1));
                let y = v.tail(Some(1));
                &x * &x + &y * &y
            })
            .apply(|v| v.f64().unwrap().apply(|v| v.sqrt().to_arcsec()).into())
    })?;
    df.apply("SegmentTipTilt", |v| {
        v.list()
            .unwrap()
            .apply(|v| {
                let x = v.head(Some(7));
                let y = v.tail(Some(7));
                &x * &x + &y * &y
            })
            .apply(|v| v.f64().unwrap().apply(|v| v.sqrt().to_arcsec()).into())
    })?;
    println!("{:?}", df);

    let out = df
        .lazy()
        .select([col("TipTilt").arr().get(Expr::from(0)).mean()])
        .collect()?;
    println!("{out}"); */

    /*     let out = df
        .lazy()
        .select([mean("TipTilt").alias("Mean")])
        .collect()?;
    println!("{out}"); */

    /*     let stt = df.apply("SegmentTipTilt", |v| {
        v.list().unwrap().into_iter().take(7).collect::<Vec<f64()
    }); */
    Ok(())
}
