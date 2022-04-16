use crseo::{
    calibrations, Builder, Calibration, Geometric, PistonSensorBuilder, ShackHartmann, GMT, SH24,
};
use dos_actors::clients::ceo;
use dos_actors::io::{Data, Read, Write};
//use dos_actors::prelude::*;
use dos_actors::Update;
use nalgebra as na;
//use skyangle::Conversion;
//use std::default::Default;
use osqp::{CscMatrix, Problem, Settings};
use std::sync::Arc;
use std::time::Instant;

const M1_N_MODE: usize = 5;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let gmt_builder = GMT::new().m1_n_mode(M1_N_MODE);
    let mut onaxis_gmt: ceo::OpticalModel = ceo::OpticalModel::builder()
        .gmt(gmt_builder.clone())
        .build()?;
    // AGWS SH24
    let mut agws_sh48 = ceo::OpticalModel::builder()
        .gmt(gmt_builder.clone())
        .sensor_builder(SH24::<Geometric>::new())
        .build()?;
    use calibrations::Mirror;
    use calibrations::Segment::*;
    // GMT 2 WFS
    let mut gmt2wfs = Calibration::new(
        &agws_sh48.gmt,
        &agws_sh48.src,
        SH24::<crseo::Geometric>::new(),
    );
    let mut specs = vec![
        Some(vec![
            (Mirror::M1, vec![Txyz(1e-6, None), Rxyz(1e-6, None)]),
            (Mirror::M1MODES, vec![Modes(1e-6, 0..M1_N_MODE)]),
            (Mirror::M2, vec![Txyz(1e-6, None), Rxyz(1e-6, None)])
        ]);
        6
    ];
    specs.append(&mut vec![Some(vec![
        (Mirror::M1, vec![Txyz(1e-6, None), Rxyz(1e-6, Some(0..2))]),
        (Mirror::M1MODES, vec![Modes(1e-6, 0..M1_N_MODE)]),
        (Mirror::M2, vec![Txyz(1e-6, None), Rxyz(1e-6, Some(0..2))]),
    ])]);
    let mut specs = vec![Some(vec![(Mirror::M1, vec![Rxyz(1e-6, Some(0..2_))]),]); 7];
    let now = Instant::now();
    gmt2wfs.calibrate(
        specs,
        calibrations::ValidLensletCriteria::OtherSensor(&mut agws_sh48.sensor.as_mut().unwrap()),
    );
    println!(
        "GMT 2 WFS calibration [{}x{}] in {}s",
        gmt2wfs.n_data,
        gmt2wfs.n_mode,
        now.elapsed().as_secs()
    );
    let poke_sum = gmt2wfs.poke.from_dev().iter().sum::<f64>();
    println!("Poke sum: {}", poke_sum);
    let dof_2_wfs: Vec<f64> = gmt2wfs.poke.into();
    let dof_2_wfs = na::DMatrix::<f64>::from_column_slice(
        dof_2_wfs.len() / gmt2wfs.n_mode,
        gmt2wfs.n_mode,
        &dof_2_wfs,
    );
    let wfs_2_rxy = dof_2_wfs.clone().pseudo_inverse(1e-12).unwrap();
    //let mut file = std::fs::File::create("poke.pkl").unwrap();
    //serde_pickle::to_writer(&mut file, &gmt2wfs.poke.from_dev(), Default::default()).unwrap();

    let rxy_2_wfs_svd = dof_2_wfs.clone().svd(true, true);
    dbg!(rxy_2_wfs_svd.singular_values);
    println!(
        "V^T shape: {:?}",
        rxy_2_wfs_svd.v_t.as_ref().unwrap().shape()
    );
    let w2 = {
        let p_t = rxy_2_wfs_svd
            .v_t
            .as_ref()
            .unwrap()
            .rows(gmt2wfs.n_mode - 12, 12);
        p_t.transpose() * p_t
    };
    println!("W2 shape: {:?}", w2.shape());

    let mut gmt_state = crseo::gmt::SegmentsDof::default(); //.m1_n_mode(M1_N_MODE);
    let mut modes = vec![0f64; M1_N_MODE];
    //modes[1] = 0e-6;
    gmt_state.segment(
        1,
        crseo::gmt::SegmentDof::M1((
            Some(crseo::gmt::MirrorDof::RigidBodyMotions((
                None,
                Some(crseo::gmt::RBM::Rxyz(vec![1e-6, 0.0, 0.])),
            ))),
            None,
        )),
    )?;
    /*
            .segment(
                2,
                crseo::gmt::SegmentDof::M2((
                    Some(crseo::gmt::MirrorDof::RigidBodyMotions((
                        Some(crseo::gmt::RBM::Txyz(vec![1e-6, 0.0, 0.])),
                        None,
                    ))),
                    None,
                )),
            )?
            .segment(
                5,
                crseo::gmt::SegmentDof::M1((
                    Some(crseo::gmt::MirrorDof::RigidBodyMotions((
                        None,
                        Some(crseo::gmt::RBM::Rxyz(vec![0., 1e-6, 0.])),
                    ))),
                    Some(crseo::gmt::MirrorDof::Modes(vec![0f64; M1_N_MODE])),
                )),
            )?
            .segment(
                7,
                crseo::gmt::SegmentDof::M1((
                    Some(crseo::gmt::MirrorDof::RigidBodyMotions((
                        None,
                        Some(crseo::gmt::RBM::Rxyz(vec![1e-6, 1e-6, 0.])),
                    ))),
                    Some(crseo::gmt::MirrorDof::Modes(vec![0f64; M1_N_MODE])),
                )),
            )?;
    */
    let data = Arc::new(Data::<crseo::gmt::SegmentsDof, ceo::GmtState>::new(
        gmt_state.clone(),
    ));
    agws_sh48.read(data);
    agws_sh48.update();
    let data: Option<Arc<Data<Vec<f64>, ceo::SensorData>>> =
        <ceo::OpticalModel<ShackHartmann<Geometric>, SH24<Geometric>> as Write<
            Vec<f64>,
            ceo::SensorData,
        >>::write(&mut agws_sh48);
    let wfs_data: Vec<f64> = (&**data.as_ref().unwrap()).into();

    dbg!(wfs_2_rxy.shape());
    dbg!(wfs_data.len());
    let now = Instant::now();
    let _a = &wfs_2_rxy * na::DVector::from_vec(wfs_data.clone());
    println!("LSQ solution ({:}mus):", now.elapsed().as_micros());
    /*a.as_slice()
    .chunks(2)
    .enumerate()
    .for_each(|(k, x)| println!("#{}: [{:+0.3},{:+0.3}]", 1 + k, x[0] * 1e6, x[1] * 1e6));*/

    let data = Arc::new(Data::<crseo::gmt::SegmentsDof, ceo::GmtState>::new(
        gmt_state.clone(),
    ));
    onaxis_gmt.read(data);
    onaxis_gmt.update();
    let data: Option<Arc<Data<Vec<f64>, ceo::WfeRms>>> = onaxis_gmt.write();
    println!("{data:?}");

    let mut prob = {
        let settings = Settings::default().verbose(false);
        let p = {
            let d2 = &dof_2_wfs.transpose() * &dof_2_wfs; // + w2;
            CscMatrix::from_column_iter_dense(
                d2.nrows(),
                d2.ncols(),
                d2.as_slice().to_vec().into_iter(),
            )
            .into_upper_tri()
        };
        let q = na::DMatrix::from_row_slice(1, wfs_data.len(), &wfs_data) * &dof_2_wfs;
        let a: Vec<_> = (0..gmt2wfs.n_mode)
            .map(|i| {
                let mut v = vec![0f64; gmt2wfs.n_mode];
                v[i] = 1f64;
                v
            })
            .collect();
        let umin = vec![f64::NEG_INFINITY; gmt2wfs.n_mode];
        let umax = vec![f64::INFINITY; gmt2wfs.n_mode];
        Problem::new(p, q.as_slice(), &a, &umin, &umax, &settings)?
    };
    //dbg!(&gmt_state);
    let u0: Vec<f64> = gmt_state.into(); //m12_rbm.clone();
                                         //u0.remove(78);
                                         //u0.pop();
    dbg!(u0.len());
    let mut u = vec![0f64; gmt2wfs.n_mode];
    let gain = 0.5;
    let n_step = 50;
    let mut steps = 0..n_step;

    let gmt_state = loop {
        let now = Instant::now();
        let result = prob.solve();
        let x = result.x().unwrap();
        /*x.chunks(2)
        .enumerate()
        .for_each(|(k, x)| println!("#{}: [{:+0.3},{:+0.3}]", 1 + k, x[0] * 1e6, x[1] * 1e6));*/
        u.iter_mut().zip(x).for_each(|(u, x)| *u += gain * x);
        let r: Vec<_> = u.iter().zip(&u0).map(|(u, u0)| u + u0).collect();
        let m12_rbm = r.clone();
        //m12_rbm.insert(78, 0f64);
        //m12_rbm.push(0f64);
        let gmt_state = crseo::gmt::SegmentsDof::new()
            .m1_n_mode(M1_N_MODE)
            .from_vec(m12_rbm);
        if let Some(k) = steps.next() {
            println!("Step #{k}");
            println!("QP solution ({:}mus):", now.elapsed().as_micros());
        } else {
            break gmt_state;
        }
        let data = Arc::new(Data::<crseo::gmt::SegmentsDof, ceo::GmtState>::new(
            gmt_state.clone(),
        ));
        agws_sh48.read(data);
        agws_sh48.update();
        let data: Option<Arc<Data<Vec<f64>, ceo::SensorData>>> =
            <ceo::OpticalModel<ShackHartmann<Geometric>, SH24<Geometric>> as Write<
                Vec<f64>,
                ceo::SensorData,
            >>::write(&mut agws_sh48);
        let wfs_data: Vec<f64> = (&**data.as_ref().unwrap()).into();
        dbg!(wfs_data.iter().cloned().sum::<f64>());

        let data = Arc::new(Data::<crseo::gmt::SegmentsDof, ceo::GmtState>::new(
            gmt_state.clone(),
        ));
        onaxis_gmt.read(data);
        onaxis_gmt.update();
        let data: Option<Arc<Data<Vec<f64>, ceo::WfeRms>>> = onaxis_gmt.write();
        println!("{data:?}");
        let data: Option<Arc<Data<Vec<f64>, ceo::SegmentWfeRms>>> = onaxis_gmt.write();
        println!("{data:?}");
        let data: Option<Arc<Data<Vec<f64>, ceo::SegmentPiston>>> = onaxis_gmt.write();
        println!("{data:?}");

        let q = na::DMatrix::from_row_slice(1, wfs_data.len(), &wfs_data) * &dof_2_wfs;
        prob.update_lin_cost(q.as_slice());
    };
    println!("{gmt_state}");

    Ok(())
}
