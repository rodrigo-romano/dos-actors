use crseo::{calibrations, Builder, Calibration, Geometric, ShackHartmann, SH24};
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut onaxis_gmt: ceo::OpticalModel = ceo::OpticalModel::builder().build()?;
    // AGWS SH24
    let mut agws_sh48 = ceo::OpticalModel::builder()
        .sensor_builder(ceo::SensorBuilder::new(SH24::<Geometric>::new()))
        .build()?;
    use calibrations::Mirror;
    use calibrations::Segment::*;
    let mirror = vec![Mirror::M1, Mirror::M2];
    let mut segments = vec![vec![Txyz(1e-6, None), Rxyz(1e-6, None)]; 6];
    segments.append(&mut vec![vec![Txyz(1e-6, None), Rxyz(1e-6, Some(0..2))]]);
    let mut gmt2wfs = Calibration::new(
        &agws_sh48.gmt,
        &agws_sh48.src,
        SH24::<crseo::Geometric>::new(),
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
    let poke_sum = gmt2wfs.poke.from_dev().iter().sum::<f32>();
    println!("Poke sum: {}", poke_sum);
    let rxy_2_wfs: Vec<f64> = gmt2wfs.poke.clone().into();
    let rxy_2_wfs = na::DMatrix::<f64>::from_column_slice(
        rxy_2_wfs.len() / gmt2wfs.n_mode,
        gmt2wfs.n_mode,
        &rxy_2_wfs,
    );
    let wfs_2_rxy = rxy_2_wfs.clone().pseudo_inverse(1e-12).unwrap();

    let rxy_2_wfs_svd = rxy_2_wfs.clone().svd(false, true);
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

    let m1_txyz = vec![vec![0f64; 3]; 7];
    let mut m1_rxyz = vec![vec![0f64; 3]; 7];
    m1_rxyz[1] = vec![1e-6, 0.0, 0.];
    m1_rxyz[4] = vec![0., 1e-6, 0.];
    m1_rxyz[6] = vec![1e-6, 1e-6, 0.];
    let m2_txyz = vec![vec![0f64; 3]; 7];
    let m2_rxyz = vec![vec![0f64; 3]; 7];
    /*let m1_rbm: Vec<_> = m1_txyz
            .iter()
            .zip(m1_rxyz.iter())
            .flat_map(|(t, r)| {
                let mut tr = t.to_vec();
                tr.extend_from_slice(r);
                tr
            })
            .collect();
    */
    let mut m12_rbm = vec![];
    for k in 0..7 {
        m12_rbm.extend_from_slice(&m1_txyz[k]);
        m12_rbm.extend_from_slice(&m1_rxyz[k]);
        m12_rbm.extend_from_slice(&m2_txyz[k]);
        m12_rbm.extend_from_slice(&m2_rxyz[k]);
    }
    let m1_rbm: Vec<f64> = m12_rbm
        .chunks(6)
        .step_by(2)
        .flat_map(|x| x.to_owned())
        .collect();
    let m2_rbm: Vec<f64> = m12_rbm
        .chunks(6)
        .skip(1)
        .step_by(2)
        .flat_map(|x| x.to_owned())
        .collect();

    let mut gmt_state = crseo::gmt::SegmentsDof::new(None, None);
    gmt_state
        .segment(
            2,
            crseo::gmt::SegmentDof::M1((
                Some(crseo::gmt::MirrorDof::RigidBodyMotions((
                    None,
                    Some(crseo::gmt::RBM::Rxyz(vec![1e-6, 0.0, 0.])),
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
                None,
            )),
        )?
        .segment(
            7,
            crseo::gmt::SegmentDof::M1((
                Some(crseo::gmt::MirrorDof::RigidBodyMotions((
                    None,
                    Some(crseo::gmt::RBM::Rxyz(vec![1e-6, 1e-6, 0.])),
                ))),
                None,
            )),
        )?;

    let data = Arc::new(Data::<Vec<f64>, ceo::M1rbm>::new(m1_rbm.clone()));
    agws_sh48.read(data);
    agws_sh48.update();
    let data: Option<Arc<Data<Vec<f64>, ceo::SensorData>>> =
        <ceo::OpticalModel<ShackHartmann<Geometric>, SH24<Geometric>> as Write<
            Vec<f64>,
            ceo::SensorData,
        >>::write(&mut agws_sh48);
    let wfs_data: Vec<f64> = (&**data.as_ref().unwrap()).into();

    let now = Instant::now();
    let a = &wfs_2_rxy * na::DVector::from_vec(wfs_data.clone());
    println!("LSQ solution ({:}mus):", now.elapsed().as_micros());
    /*a.as_slice()
    .chunks(2)
    .enumerate()
    .for_each(|(k, x)| println!("#{}: [{:+0.3},{:+0.3}]", 1 + k, x[0] * 1e6, x[1] * 1e6));*/

    let data = Arc::new(Data::<Vec<f64>, ceo::M1rbm>::new(m1_rbm.clone()));
    onaxis_gmt.read(data);
    onaxis_gmt.update();
    let data: Option<Arc<Data<Vec<f64>, ceo::WfeRms>>> = onaxis_gmt.write();
    println!("{data:?}");

    let mut prob = {
        let settings = Settings::default().verbose(false);
        let p = {
            let d2 = &rxy_2_wfs.transpose() * &rxy_2_wfs + w2;
            CscMatrix::from_column_iter_dense(
                d2.nrows(),
                d2.ncols(),
                d2.as_slice().to_vec().into_iter(),
            )
            .into_upper_tri()
        };
        let q = na::DMatrix::from_row_slice(1, wfs_data.len(), &wfs_data) * &rxy_2_wfs;
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
    /*let u0: Vec<_> = rxyz
        .into_iter()
        .flat_map(|rxyz| rxyz[..2].to_vec())
    .collect();*/
    let mut u0 = m12_rbm.clone();
    u0.remove(78);
    u0.pop();
    let mut u = vec![0f64; gmt2wfs.n_mode];
    let gain = 0.5;
    let n_step = 10;

    for k in 0..n_step {
        println!("Step #{k}");
        let now = Instant::now();
        let result = prob.solve();
        let x = result.x().unwrap();
        println!("QP solution ({:}mus):", now.elapsed().as_micros());
        /*x.chunks(2)
        .enumerate()
        .for_each(|(k, x)| println!("#{}: [{:+0.3},{:+0.3}]", 1 + k, x[0] * 1e6, x[1] * 1e6));*/
        u.iter_mut().zip(x).for_each(|(u, x)| *u += gain * x);
        let r: Vec<_> = u.iter().zip(&u0).map(|(u, u0)| u + u0).collect();
        /*r.chunks(2)
        .enumerate()
        .for_each(|(k, x)| println!("#{}: [{:+0.3},{:+0.3}]", 1 + k, x[0] * 1e6, x[1] * 1e6));*/

        /*
           let rxyz: Vec<_> = r
               .chunks(2)
               .map(|r| {
                   let mut v = r.to_vec();
                   v.push(0f64);
                   v
               })
               .collect();

           let m1_rbm: Vec<_> = txyz
               .iter()
               .zip(rxyz.iter())
               .flat_map(|(t, r)| {
                   let mut tr = t.to_vec();
                   tr.extend_from_slice(r);
                   tr
               })
               .collect();
        */
        let mut m12_rbm = r.clone();
        m12_rbm.insert(78, 0f64);
        m12_rbm.push(0f64);
        let m1_rbm: Vec<f64> = m12_rbm
            .chunks(6)
            .step_by(2)
            .flat_map(|x| x.to_owned())
            .collect();
        let m2_rbm: Vec<f64> = m12_rbm
            .chunks(6)
            .skip(1)
            .step_by(2)
            .flat_map(|x| x.to_owned())
            .collect();
        let data = Arc::new(Data::<Vec<f64>, ceo::M1rbm>::new(m1_rbm.clone()));
        agws_sh48.read(data);
        let data = Arc::new(Data::<Vec<f64>, ceo::M2rbm>::new(m2_rbm.clone()));
        agws_sh48.read(data);
        agws_sh48.update();
        let data: Option<Arc<Data<Vec<f64>, ceo::SensorData>>> =
            <ceo::OpticalModel<ShackHartmann<Geometric>, SH24<Geometric>> as Write<
                Vec<f64>,
                ceo::SensorData,
            >>::write(&mut agws_sh48);
        let wfs_data: Vec<f64> = (&**data.as_ref().unwrap()).into();
        dbg!(wfs_data.iter().cloned().sum::<f64>());

        let data = Arc::new(Data::<Vec<f64>, ceo::M1rbm>::new(m1_rbm));
        onaxis_gmt.read(data);
        let data = Arc::new(Data::<Vec<f64>, ceo::M2rbm>::new(m2_rbm));
        onaxis_gmt.read(data);
        onaxis_gmt.update();
        let data: Option<Arc<Data<Vec<f64>, ceo::WfeRms>>> = onaxis_gmt.write();
        println!("{data:?}");
        let data: Option<Arc<Data<Vec<f64>, ceo::SegmentWfeRms>>> = onaxis_gmt.write();
        println!("{data:?}");
        let data: Option<Arc<Data<Vec<f64>, ceo::SegmentPiston>>> = onaxis_gmt.write();
        println!("{data:?}");

        let q = na::DMatrix::from_row_slice(1, wfs_data.len(), &wfs_data) * &rxy_2_wfs;
        prob.update_lin_cost(q.as_slice());
    }

    Ok(())
}
