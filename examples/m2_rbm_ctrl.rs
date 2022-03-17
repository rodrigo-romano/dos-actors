use crseo::{calibrations, Builder, Calibration, Geometric, ShackHartmann, SH24, SHACKHARTMANN};
use dos_actors::clients::{
    arrow_client::Arrow,
    ceo,
    fsm::*,
    m1::*,
    mount::{Mount, MountEncoders, MountTorques},
};
use dos_actors::prelude::*;
use fem::{
    dos::{DiscreteModalSolver, Exponential, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use futures::future::join_all;
use gmt_lom::{LoaderTrait, OpticalSensitivities};
use nalgebra as na;
use std::default::Default;
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //simple_logger::SimpleLogger::new().env().init().unwrap();

    let sim_sampling_frequency = 1000;
    let sim_duration = 4_usize;

    // FEM
    let state_space = {
        let fem = FEM::from_env()?.static_from_env()?;
        let n_io = (fem.n_inputs(), fem.n_outputs());
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .ins::<MCM2SmHexF>()
            .ins::<MCM2PZTF>()
            .outs::<MCM2SmHexD>()
            .outs::<MCM2PZTD>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .use_static_gain_compensation(n_io)
            .build()?
    };
    println!("{}", state_space);

    const M1_RATE: usize = 10;
    assert_eq!(sim_sampling_frequency / M1_RATE, 100);
    const FSM_RATE: usize = 1;
    assert_eq!(sim_sampling_frequency / FSM_RATE, 1000);

    let n_segment = 7;
    let n_step = sim_sampling_frequency * sim_duration;
    /*
        let signals = (0..n_segment).fold(Signals::new(vec![42], n_step), |s, i| {
            (0..6).fold(s, |ss, j| {
                ss.output_signal(
                    0,
                    i * 6 + j,
                    Signal::Constant((-1f64).powi(j as i32) * (1 + i) as f64 * 1e-6),
                )
            })
        });
    */
    //let mut source: Initiator<_> = signals.clone().into();
    let mut source: Initiator<_> = Signals::new(vec![42], n_step).clone().into();
    // M2 POSITIONER COMMAND
    let mut m2_pos_cmd: Initiator<_> = Signals::new(vec![42], n_step).into();
    // M2 TT COMMAND
    /*let tt_signals = (0..n_segment).fold(Signals::new(vec![14], n_step), |s, i| {
        (0..2).fold(s, |ss, j| {
            ss.output_signal(
                0,
                i * 2 + j,
                Signal::Constant((-1f64).powi(j as i32) * (1 + i) as f64 * 1e-6),
            )
        })
    });*/
    let tt_signals = Signals::new(vec![14], n_step).output_signal(0, 0, Signal::Constant(1e-6));
    let mut m2_tt_cmd: Initiator<_, FSM_RATE> = tt_signals.into();
    //let mut m2_tt_cmd: Initiator<_> = Signals::new(vec![14], n_step).into();
    // FEM
    let mut fem: Actor<_> = state_space.into();
    // FSM POSITIONNER
    let mut m2_positionner: Actor<_> = fsm::positionner::Controller::new().into();
    // FSM PIEZOSTACK
    let mut m2_piezostack: Actor<_> = fsm::piezostack::Controller::new().into();
    //let mut m2_piezostack_sampler: Actor<_,FSM_RATE,1> = Sampler::<D, PZTcmd>::default().into();
    // FSM TIP-TILT CONTROL
    let mut m2_tiptilt: Actor<_, FSM_RATE, 1> = fsm::tiptilt::Controller::new().into();
    // ON-AXIS GMT RAY TRACING
    let mut gmt_on_axis: Actor<_> =
        ceo::OpticalModel::<ShackHartmann<Geometric>, SHACKHARTMANN<Geometric>>::builder()
            .build()?
            .into();
    // AGWS SH24
    let mut agws_sh24 = ceo::OpticalModel::builder()
        .sensor_builder(ceo::SensorBuilder::new(SH24::<Geometric>::new()))
        .build()?;
    let mirror = vec![calibrations::Mirror::M2];
    let segments = vec![vec![calibrations::Segment::Rxyz(1e-6, Some(0..2))]; 7];
    let mut gmt2wfs = Calibration::new(
        &agws_sh24.gmt,
        &agws_sh24.src,
        SH24::<crseo::Geometric>::new(),
    );
    let now = Instant::now();
    gmt2wfs.calibrate(
        mirror,
        segments,
        calibrations::ValidLensletCriteria::Threshold(Some(0.8)),
    );
    println!(
        "GTM 2 WFS calibration [{}x{}] in {}s",
        gmt2wfs.n_data,
        gmt2wfs.n_mode,
        now.elapsed().as_secs()
    );
    let rxy_2_wfs: Vec<f64> = gmt2wfs.poke.clone().into();
    let rxy_2_wfs = na::DMatrix::<f64>::from_column_slice(rxy_2_wfs.len() / 14, 14, &rxy_2_wfs);
    let wfs_2_rxy = rxy_2_wfs.pseudo_inverse(1e-12).unwrap();
    let optical_sensitivities = gmt_lom::Loader::<OpticalSensitivities>::default()
        .path("/home/ubuntu/projects/gmt-lom")
        .load()?;
    //let optical_sensitivities = gmt_lom::Loader::<Vec<gmt_lom::OpticalSensitivities>>::default();
    let rxy_2_stt = {
        let rxy_2_stt = (*optical_sensitivities)[3].m2_rxy()?;
        let v: Vec<_> = (0..7)
            .flat_map(|i| vec![rxy_2_stt.row(i), rxy_2_stt.row(i + 7)])
            .collect();
        na::DMatrix::<f64>::from_rows(&v)
    };
    println!("{:.2}", rxy_2_stt);
    let wfs_2_stt = rxy_2_stt * wfs_2_rxy;

    let wfs = agws_sh24.sensor.as_mut().unwrap();
    agws_sh24
        .gmt
        .m2_segment_state(2, &[0., 0.0, 0.], &[1e-6, 0.0, 0.]);
    agws_sh24
        .gmt
        .m2_segment_state(5, &[0., 0.0, 0.], &[0., 1e-6, 0.]);
    agws_sh24
        .gmt
        .m2_segment_state(7, &[0., 0.0, 0.], &[1e-6, 1e-6, 0.]);
    wfs.reset();
    agws_sh24
        .src
        .through(&mut agws_sh24.gmt)
        .xpupil()
        .through(wfs);
    wfs.process();
    let wfs_data: Vec<f64> = wfs.get_data().into();
    let a = &wfs_2_stt * na::DVector::from_vec(wfs_data);
    println!("{:}", a);
    a.as_slice()
        .chunks(2)
        .enumerate()
        .for_each(|(k, x)| println!("#{}: [{:+0.3},{:+0.3}]", 1 + k, x[0] * 1e6, x[1] * 1e6));

    agws_sh24.gmt.reset();
    agws_sh24.src.reset();
    wfs.reset();

    agws_sh24.sensor_matrix_transform(wfs_2_stt);
    let mut gmt_agws_sh24: Actor<_, 1, FSM_RATE> = agws_sh24.into();

    type D = Vec<f64>;

    m2_pos_cmd
        .add_output::<D, M2poscmd>(None)
        .into_input(&mut m2_positionner);
    m2_positionner
        .add_output::<D, MCM2SmHexF>(None)
        .into_input(&mut fem);

    m2_tt_cmd
        .add_output::<D, TTSP>(None)
        .into_input(&mut m2_tiptilt);
    m2_tiptilt
        .add_output::<D, PZTcmd>(None)
        .into_input(&mut m2_piezostack);
    m2_piezostack
        .add_output::<D, MCM2PZTF>(None)
        .into_input(&mut fem);

    let logging = Logging::<f64>::default().into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    let m2_rbm = Logging::<f64>::default().into_arcx();
    let mut m2_rbm_logs = Terminator::<_>::new(m2_rbm.clone());

    fem.add_output::<D, MCM2SmHexD>(None)
        .into_input(&mut m2_positionner);
    fem.add_output::<D, MCM2PZTD>(None)
        .into_input(&mut m2_piezostack);
    fem.add_output::<D, OSSM1Lcl>(Some(vec![usize::MAX; 3]))
        .into_input(&mut sink)
        .into_input(&mut gmt_on_axis)
        .into_input(&mut gmt_agws_sh24);
    fem.add_output::<D, MCM2Lcl6D>(Some(vec![usize::MAX, 3]))
        .into_input(&mut m2_rbm_logs)
        .into_input(&mut gmt_on_axis)
        .into_input(&mut gmt_agws_sh24);

    let wfe_rms = Logging::<f64>::default().into_arcx();
    let mut wfe_rms_logs = Terminator::<_>::new(wfe_rms.clone());

    let sh24_tt_fb = Arrow::builder(n_step)
        .entry::<f64, TTFB>(14)
        .filename("SH24-TTFB.parquet".to_string())
        .build()
        .into_arcx();
    let mut sh24_tt_fb_logs = Terminator::<_, FSM_RATE>::new(sh24_tt_fb.clone());

    gmt_on_axis
        .add_output::<D, ceo::WfeRms>(None)
        .into_input(&mut wfe_rms_logs);
    gmt_agws_sh24
        .add_output::<D, TTFB>(Some(vec![1; 2]))
        .into_input(&mut sh24_tt_fb_logs)
        .into_input(&mut m2_tiptilt);

    println!("{source}{m2_positionner}{m2_piezostack}{m2_tiptilt}{fem}{gmt_on_axis}{wfe_rms_logs}");

    let mut handles = vec![
        m2_pos_cmd.spawn(),
        m2_positionner.spawn(),
        m2_piezostack.spawn(),
        m2_tt_cmd.spawn(),
        m2_rbm_logs.spawn(),
        gmt_on_axis.spawn(),
        gmt_agws_sh24.spawn(),
        wfe_rms_logs.spawn(),
        sh24_tt_fb_logs.spawn(),
    ];

    fem.bootstrap::<D, MCM2SmHexD>()
        .await
        .bootstrap::<D, MCM2PZTD>()
        .await
        .bootstrap::<D, OSSM1Lcl>()
        .await
        .bootstrap::<D, MCM2Lcl6D>()
        .await;
    handles.push(fem.spawn());

    m2_tiptilt.bootstrap::<D, PZTcmd>().await;
    handles.push(m2_tiptilt.spawn());

    println!("Starting the model");
    let now = Instant::now();

    sink.run().await;

    /*for h in handles.into_iter() {
        h.await?;
    }*/
    join_all(handles).await;
    let elapsed = now.elapsed().as_millis();

    println!("Model run {}s in {}ms ()", sim_duration, elapsed);

    let tau = (sim_sampling_frequency as f64).recip();
    let labels = vec!["Tx", "Ty", "Tz", "Rx", "Ry", "Rz"];

    {
        let logging_lock = logging.lock().await;
        (0..6)
            .map(|k| {
                (**logging_lock)
                    .iter()
                    .skip(k)
                    .step_by(6)
                    .cloned()
                    .collect::<Vec<f64>>()
            })
            .enumerate()
            .for_each(|(k, rbm)| {
                let _: complot::Plot = (
                    rbm.chunks(7).enumerate().map(|(i, x)| {
                        (
                            i as f64 * tau,
                            x.iter().map(|x| x * 1e6).collect::<Vec<f64>>(),
                        )
                    }),
                    complot::complot!(
                        format!("examples/figures/m1_rbm_ctrl-{}.png", k + 1),
                        xlabel = "Time [s]",
                        ylabel = labels[k]
                    ),
                )
                    .into();
            });
    }

    {
        let logging_lock = m2_rbm.lock().await;
        (0..6)
            .map(|k| {
                (**logging_lock)
                    .iter()
                    .skip(k)
                    .step_by(6)
                    .cloned()
                    .collect::<Vec<f64>>()
            })
            .enumerate()
            .for_each(|(k, rbm)| {
                let _: complot::Plot = (
                    rbm.chunks(7).enumerate().map(|(i, x)| {
                        (
                            i as f64 * tau,
                            x.iter().map(|x| x * 1e6).collect::<Vec<f64>>(),
                        )
                    }),
                    complot::complot!(
                        format!("examples/figures/m2_rbm_ctrl-{}.png", k + 1),
                        xlabel = "Time [s]",
                        ylabel = labels[k]
                    ),
                )
                    .into();
            });
    }

    {
        let logging_lock = wfe_rms.lock().await;
        let _: complot::Plot = (
            (**logging_lock)
                .iter()
                .enumerate()
                .map(|(i, x)| (i as f64 * tau, vec![*x * 1e9])),
            complot::complot!(
                "examples/figures/wfe_rms.png",
                xlabel = "Time [s]",
                ylabel = "WFE RMS[nm]"
            ),
        )
            .into();
    }
    Ok(())
}
