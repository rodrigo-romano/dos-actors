use crseo::{calibrations, Builder, Calibration, Geometric, GMT, SH48};
use dos_actors::{
    clients::{
        arrow_client::Arrow,
        ceo,
        ceo::M1modes,
        fsm::*,
        m1::*,
        mount::{Mount, MountEncoders, MountSetPoint, MountTorques},
        Integrator,
    },
    prelude::*,
};
use fem::{
    dos::{DiscreteModalSolver, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use nalgebra as na;
use rand::Rng;
use std::{fs::File, sync::Arc, time::Instant};

fn fig_2_mode(sid: u32) -> na::DMatrix<f64> {
    let fig_2_mode: Vec<f64> =
        bincode::deserialize_from(File::open(format!("m1s{sid}fig2mode.bin")).unwrap()).unwrap();
    if sid < 7 {
        na::DMatrix::from_vec(162, 602, fig_2_mode)
    } else {
        na::DMatrix::from_vec(151, 579, fig_2_mode)
    }
}

pub struct Mode2Force<const S: usize> {
    mode_2_force: na::DMatrix<f64>,
    mode: na::DVector<f64>,
    force: Option<na::DVector<f64>>,
}
impl<const S: usize> Mode2Force<S> {
    pub fn new() -> Self {
        let mode_2_force = {
            let mode_2_force: Vec<f64> =
                bincode::deserialize_from(File::open(format!("m1s{S}mode2forces.bin")).unwrap())
                    .unwrap();
            if S == 7 {
                na::DMatrix::from_vec(306, 151, mode_2_force)
            } else {
                na::DMatrix::from_vec(335, 162, mode_2_force)
            }
        };
        Self {
            mode_2_force,
            mode: na::DVector::zeros(if S == 7 { 151 } else { 162 }),
            force: None,
        }
    }
}
use dos_actors::{
    io::{Data, Read, Write},
    Update,
};
impl<const S: usize> Update for Mode2Force<S> {
    fn update(&mut self) {
        self.force = Some(&self.mode_2_force * &self.mode);
    }
}
impl<U, const S: usize> Write<Vec<f64>, U> for Mode2Force<S> {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, U>>> {
        self.force
            .as_ref()
            .map(|force| Arc::new(Data::new(force.as_slice().to_vec())))
    }
}
enum M1ModalCmd {}
impl<const S: usize> Read<Vec<f64>, M1ModalCmd> for Mode2Force<S> {
    fn read(&mut self, data: Arc<Data<Vec<f64>, M1ModalCmd>>) {
        self.mode
            .iter_mut()
            .zip(&(**data)[27 * (S - 1)..27 * S])
            .for_each(|(m, d)| *m = *d);
    }
}

#[tokio::test]
async fn setpoint_mount_m1() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000;
    let sim_duration = 10_usize;
    let n_step = sim_sampling_frequency * sim_duration;

    const M1_RATE: usize = 10;
    assert_eq!(sim_sampling_frequency / M1_RATE, 100);

    const SH48_RATE: usize = 100;
    assert_eq!(sim_sampling_frequency / SH48_RATE, 10);

    type D = Vec<f64>;

    let state_space = {
        let fem = FEM::from_env()?.static_from_env()?;
        let n_io = (fem.n_inputs(), fem.n_outputs());
        print!("{fem}");
        {}
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .ins::<OSSElDriveTorque>()
            .ins::<OSSAzDriveTorque>()
            .ins::<OSSRotDriveTorque>()
            .ins::<OSSHarpointDeltaF>()
            .ins::<M1ActuatorsSegment1>()
            .ins::<M1ActuatorsSegment2>()
            .ins::<M1ActuatorsSegment3>()
            .ins::<M1ActuatorsSegment4>()
            .ins::<M1ActuatorsSegment5>()
            .ins::<M1ActuatorsSegment6>()
            .ins::<M1ActuatorsSegment7>()
            .ins::<MCM2SmHexF>()
            .ins::<MCM2PZTF>()
            .outs::<OSSAzEncoderAngle>()
            .outs::<OSSElEncoderAngle>()
            .outs::<OSSRotEncoderAngle>()
            .outs::<OSSHardpointD>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .outs_with::<M1Segment1AxialD>(fig_2_mode(1))
            .outs_with::<M1Segment2AxialD>(fig_2_mode(2))
            .outs_with::<M1Segment3AxialD>(fig_2_mode(3))
            .outs_with::<M1Segment4AxialD>(fig_2_mode(4))
            .outs_with::<M1Segment5AxialD>(fig_2_mode(5))
            .outs_with::<M1Segment6AxialD>(fig_2_mode(6))
            .outs_with::<M1Segment7AxialD>(fig_2_mode(7))
            .outs::<MCM2SmHexD>()
            .outs::<MCM2PZTD>()
            .use_static_gain_compensation(n_io)
            .build()?
    };
    /*
        {
            for i in 0..7 {
                let mut fem = FEM::from_env()?.static_from_env()?;
                let n_io = (fem.n_inputs(), fem.n_outputs());
                let nodes = fem.outputs[i + 1]
                    .as_ref()
                    .unwrap()
                    .get_by(|x| x.properties.location.as_ref().map(|x| x.to_vec()))
                    .into_iter()
                    .flatten()
                    .collect::<Vec<f64>>();
                serde_pickle::to_writer(
                    &mut File::create(format!("M1Segment{}AxialD.pkl", i + 1))?,
                    &nodes,
                    Default::default(),
                )?;
                serde_pickle::to_writer(
                    &mut File::create(format!("m1s{}f2d.pkl", i + 1))?,
                    &fem.keep_inputs(&[i + 1])
                        .keep_outputs(&[i + 1])
                        .reduced_static_gain(n_io)
                        .unwrap()
                        .as_slice()
                        .to_vec(),
                    Default::default(),
                )?;
            }
        }
    */
    // FEM
    let mut fem: Actor<_> = state_space.into();
    // MOUNT
    let mut mount: Actor<_> = Mount::new().into();

    // HARDPOINTS
    let mut m1_hardpoints: Actor<_> = m1_ctrl::hp_dynamics::Controller::new().into();
    // LOADCELLS
    let mut m1_hp_loadcells: Actor<_, 1, M1_RATE> =
        m1_ctrl::hp_load_cells::Controller::new().into();
    // M1 SEGMENTS ACTUATORS
    let mut m1_segment1: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment1::Controller::new().into();
    let mut m1_segment2: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment2::Controller::new().into();
    let mut m1_segment3: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment3::Controller::new().into();
    let mut m1_segment4: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment4::Controller::new().into();
    let mut m1_segment5: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment5::Controller::new().into();
    let mut m1_segment6: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment6::Controller::new().into();
    let mut m1_segment7: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment7::Controller::new().into();

    //let logging = Logging::default().n_entry(2).into_arcx();
    let logging = Arrow::builder(n_step)
        .entry::<f64, OSSM1Lcl>(42)
        .entry::<f64, MCM2Lcl6D>(42)
        .entry::<f64, M1modes>(162 * 6 + 151)
        .build()
        .into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    let mut mount_set_point: Initiator<_> = (Signals::new(3, n_step), "Mount_setpoint").into();
    mount_set_point
        .add_output()
        .build::<D, MountSetPoint>()
        .into_input(&mut mount);
    mount
        .add_output()
        .build::<D, MountTorques>()
        .into_input(&mut fem);

    /*let m1s1f_set_point: Initiator<_, M1_RATE> = Signals::new(335, n_step)
        .output_signal(0, Signal::Constant(100f64))
        .into();
    let mut m1s1f_set_point: Initiator<_, M1_RATE> = (0..335)
        .step_by(5)
        .fold(Signals::new(335, n_step), |s, i| {
            s.output_signal(
                i,
                Signal::Constant(rand::thread_rng().gen_range(-100f64..100f64)),
            )
        })
    .into();*/
    // M1S1 -------------------------------------------------------------------------------
    let mut m1s1f: Actor<_, M1_RATE, M1_RATE> = (Mode2Force::<1>::new(), "M1S1_M2F").into();
    m1s1f
        .add_output()
        .build::<D, S1SAoffsetFcmd>()
        .into_input(&mut m1_segment1);
    // M1S2 -------------------------------------------------------------------------------
    let mut m1s2f: Actor<_, M1_RATE, M1_RATE> = (Mode2Force::<2>::new(), "M1S2_M2F").into();
    m1s2f
        .add_output()
        .build::<D, S2SAoffsetFcmd>()
        .into_input(&mut m1_segment2);
    // M1S3 -------------------------------------------------------------------------------
    let mut m1s3f: Actor<_, M1_RATE, M1_RATE> = (Mode2Force::<3>::new(), "M1S3_M2F").into();
    m1s3f
        .add_output()
        .build::<D, S3SAoffsetFcmd>()
        .into_input(&mut m1_segment3);
    // M1S4 -------------------------------------------------------------------------------
    let mut m1s4f: Actor<_, M1_RATE, M1_RATE> = (Mode2Force::<4>::new(), "M1S4_M2F").into();
    m1s4f
        .add_output()
        .build::<D, S4SAoffsetFcmd>()
        .into_input(&mut m1_segment4);
    // M1S5 -------------------------------------------------------------------------------
    let mut m1s5f: Actor<_, M1_RATE, M1_RATE> = (Mode2Force::<5>::new(), "M1S5_M2F").into();
    m1s5f
        .add_output()
        .build::<D, S5SAoffsetFcmd>()
        .into_input(&mut m1_segment5);
    // M1S6 -------------------------------------------------------------------------------
    let mut m1s6f: Actor<_, M1_RATE, M1_RATE> = (Mode2Force::<6>::new(), "M1S6_M2F").into();
    m1s6f
        .add_output()
        .build::<D, S6SAoffsetFcmd>()
        .into_input(&mut m1_segment6);
    // M1S7 -------------------------------------------------------------------------------
    let mut m1s7f: Actor<_, M1_RATE, M1_RATE> = (Mode2Force::<7>::new(), "M1S7_M2F").into();
    m1s7f
        .add_output()
        .build::<D, S7SAoffsetFcmd>()
        .into_input(&mut m1_segment7);

    let mut m1rbm_set_point: Initiator<_> = (Signals::new(42, n_step), "M1RBM_setpoint").into();
    m1rbm_set_point
        .add_output()
        .build::<D, M1RBMcmd>()
        .into_input(&mut m1_hardpoints);
    m1_hardpoints
        .add_output()
        .multiplex(2)
        .build::<D, OSSHarpointDeltaF>()
        .into_input(&mut fem)
        .into_input(&mut m1_hp_loadcells);

    m1_hp_loadcells
        .add_output()
        .bootstrap()
        .build::<D, S1HPLC>()
        .into_input(&mut m1_segment1);
    m1_hp_loadcells
        .add_output()
        .bootstrap()
        .build::<D, S2HPLC>()
        .into_input(&mut m1_segment2);
    m1_hp_loadcells
        .add_output()
        .bootstrap()
        .build::<D, S3HPLC>()
        .into_input(&mut m1_segment3);
    m1_hp_loadcells
        .add_output()
        .bootstrap()
        .build::<D, S4HPLC>()
        .into_input(&mut m1_segment4);
    m1_hp_loadcells
        .add_output()
        .bootstrap()
        .build::<D, S5HPLC>()
        .into_input(&mut m1_segment5);
    m1_hp_loadcells
        .add_output()
        .bootstrap()
        .build::<D, S6HPLC>()
        .into_input(&mut m1_segment6);
    m1_hp_loadcells
        .add_output()
        .bootstrap()
        .build::<D, S7HPLC>()
        .into_input(&mut m1_segment7);

    m1_segment1
        .add_output()
        .build::<D, M1ActuatorsSegment1>()
        .into_input(&mut fem);
    m1_segment2
        .add_output()
        .build::<D, M1ActuatorsSegment2>()
        .into_input(&mut fem);
    m1_segment3
        .add_output()
        .build::<D, M1ActuatorsSegment3>()
        .into_input(&mut fem);
    m1_segment4
        .add_output()
        .build::<D, M1ActuatorsSegment4>()
        .into_input(&mut fem);
    m1_segment5
        .add_output()
        .build::<D, M1ActuatorsSegment5>()
        .into_input(&mut fem);
    m1_segment6
        .add_output()
        .build::<D, M1ActuatorsSegment6>()
        .into_input(&mut fem);
    m1_segment7
        .add_output()
        .build::<D, M1ActuatorsSegment7>()
        .into_input(&mut fem);

    fem.add_output()
        .bootstrap()
        .build::<D, MountEncoders>()
        .into_input(&mut mount);
    fem.add_output()
        .build::<D, OSSHardpointD>()
        .into_input(&mut m1_hp_loadcells);
    fem.add_output()
        .build::<D, OSSM1Lcl>()
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, MCM2Lcl6D>()
        .into_input(&mut sink);

    // M2 POSITIONER COMMAND
    let mut m2_pos_cmd: Initiator<_> = (Signals::new(42, n_step), "M1_Pos_setpoint").into();
    // FSM POSITIONNER
    let mut m2_positionner: Actor<_> = fsm::positionner::Controller::new().into();
    m2_pos_cmd
        .add_output()
        .build::<D, M2poscmd>()
        .into_input(&mut m2_positionner);
    m2_positionner
        .add_output()
        .build::<D, MCM2SmHexF>()
        .into_input(&mut fem);
    // FSM PIEZOSTACK COMMAND
    let mut m2_pzt_cmd: Initiator<_> = (Signals::new(21, n_step), "M2_PZT_setpoint").into();
    // FSM PIEZOSTACK
    let mut m2_piezostack: Actor<_> = fsm::piezostack::Controller::new().into();
    m2_pzt_cmd
        .add_output()
        .build::<D, PZTcmd>()
        .into_input(&mut m2_piezostack);
    m2_piezostack
        .add_output()
        .build::<D, MCM2PZTF>()
        .into_input(&mut fem);

    fem.add_output()
        .bootstrap()
        .build::<D, MCM2SmHexD>()
        .into_input(&mut m2_positionner);
    fem.add_output()
        .bootstrap()
        .build::<D, MCM2PZTD>()
        .into_input(&mut m2_piezostack);

    // OPTICAL MODEL (Geometric)
    let mut agws_sh48: Actor<_, 1, SH48_RATE> = {
        let sensor = SH48::<Geometric>::new().n_sensor(1);
        let mut agws_sh48 = ceo::OpticalModel::builder()
            .gmt(GMT::new().m1_n_mode(162))
            .sensor_builder(sensor.clone())
            .build()?;
        use calibrations::Mirror;
        use calibrations::Segment::*;
        // GMT 2 WFS
        let mut gmt2sh48 = Calibration::new(&agws_sh48.gmt, &agws_sh48.src, sensor);
        let specs = vec![Some(vec![(Mirror::M1MODES, vec![Modes(1e-6, 0..27)])]); 7];
        let now = Instant::now();
        gmt2sh48.calibrate(
            specs,
            calibrations::ValidLensletCriteria::OtherSensor(
                &mut agws_sh48.sensor.as_mut().unwrap(),
            ),
        );
        println!(
            "GMT 2 SH48 calibration [{}x{}] in {}s",
            gmt2sh48.n_data,
            gmt2sh48.n_mode,
            now.elapsed().as_secs()
        );
        let dof_2_wfs: Vec<f64> = gmt2sh48.poke.into();
        let dof_2_wfs = na::DMatrix::<f64>::from_column_slice(
            dof_2_wfs.len() / gmt2sh48.n_mode,
            gmt2sh48.n_mode,
            &dof_2_wfs,
        );
        let wfs_2_dof = dof_2_wfs.clone().pseudo_inverse(1e-12).unwrap();
        agws_sh48.sensor_matrix_transform(wfs_2_dof);
        agws_sh48.into()
    };

    fem.add_output()
        .multiplex(2)
        .build::<D, M1modes>()
        .into_input(&mut agws_sh48)
        .into_input(&mut sink);

    let mut mode_m1s = vec![
        {
            let mut mode = vec![0f64; 162];
            mode[26] = 0e-6;
            na::DVector::from_vec(mode)
        };
        6
    ];
    //mode_m1s[0][26] = 1e-6;
    mode_m1s.push({
        let mut mode = vec![0f64; 151];
        mode[26] = 0e-6;
        na::DVector::from_vec(mode)
    });
    let zero_point: Vec<_> = mode_m1s
        .iter()
        .flat_map(|x| x.as_slice()[..27].to_vec())
        .collect();
    dbg!(&zero_point);
    let mut gain = vec![0.; 7 * 27];
    gain.iter_mut().skip(26).step_by(27).for_each(|g| *g = 0.5);
    let mut integrator: Actor<_, SH48_RATE, SH48_RATE> =
        Integrator::<f64, ceo::SensorData>::new(27 * 7)
            //.gain_vector(gain)
            .gain(0.1)
            .zero(zero_point)
            .into();

    let sh48_arrow = Arrow::builder(n_step)
        .entry::<f64, ceo::SensorData>(27 * 7)
        .entry::<f64, ceo::WfeRms>(1)
        .filename("sh48.parquet")
        .build();
    let mut sh48_log: Terminator<_, SH48_RATE> = (sh48_arrow, "SH48_Log").into();

    agws_sh48
        .add_output()
        .multiplex(2)
        .build::<D, ceo::SensorData>()
        .into_input(&mut integrator)
        .into_input(&mut sh48_log);
    agws_sh48
        .add_output()
        .build::<D, ceo::WfeRms>()
        .into_input(&mut sh48_log);

    enum M1ModalCmdRT {}
    let mut sampler: Actor<_, SH48_RATE, M1_RATE> =
        Sampler::<D, M1ModalCmdRT, M1ModalCmd>::default().into();

    integrator
        .add_output()
        .bootstrap()
        .build::<D, M1ModalCmdRT>()
        .into_input(&mut sampler);

    let sampler_arrow = Arrow::builder(n_step)
        .entry::<f64, M1ModalCmd>(27 * 7)
        .filename("sampler.parquet")
        .build();
    let mut sampler_log: Terminator<_, M1_RATE> = (sampler_arrow, "Sampler_Log").into();

    sampler
        .add_output()
        .multiplex(8)
        .build::<D, M1ModalCmd>()
        .into_input(&mut m1s1f)
        .into_input(&mut m1s2f)
        .into_input(&mut m1s3f)
        .into_input(&mut m1s4f)
        .into_input(&mut m1s5f)
        .into_input(&mut m1s6f)
        .into_input(&mut m1s7f)
        .into_input(&mut sampler_log);

    println!("{integrator}");

    Model::new(vec![
        Box::new(mount_set_point),
        Box::new(mount),
        Box::new(m1s1f),
        Box::new(m1s2f),
        Box::new(m1s3f),
        Box::new(m1s4f),
        Box::new(m1s5f),
        Box::new(m1s6f),
        Box::new(m1s7f),
        Box::new(m1rbm_set_point),
        Box::new(m1_hardpoints),
        Box::new(m1_hp_loadcells),
        Box::new(m1_segment1),
        Box::new(m1_segment2),
        Box::new(m1_segment3),
        Box::new(m1_segment4),
        Box::new(m1_segment5),
        Box::new(m1_segment6),
        Box::new(m1_segment7),
        Box::new(m2_pos_cmd),
        Box::new(m2_positionner),
        Box::new(m2_pzt_cmd),
        Box::new(m2_piezostack),
        Box::new(fem),
        Box::new(agws_sh48),
        Box::new(integrator),
        Box::new(sampler),
        Box::new(sampler_log),
        Box::new(sh48_log),
        Box::new(sink),
    ])
    .name("mount-m1-m2-sh48")
    .flowchart()
    .check()?
    .run()
    .wait()
    .await?;

    let m1_modes = (*logging.lock().await).get(format!("M1modes")).unwrap();
    m1_modes
        .last()
        .as_ref()
        .unwrap()
        .chunks(162)
        .zip(&mode_m1s)
        .enumerate()
        .for_each(|(sid, (m1sifig, mode_m1s))| {
            /*
            let fig_2_mode = {
                let fig_2_mode: Vec<f64> =
                    bincode::deserialize_from(File::open(format!("m1s{sid}fig2mode.bin")).unwrap())
                        .unwrap();
                if sid < 7 {
                    na::DMatrix::from_vec(162, 602, fig_2_mode)
                } else {
                    na::DMatrix::from_vec(151, 579, fig_2_mode)
                }
            };*/
            let mode_from_fig = na::DVector::from_column_slice(m1sifig);
            //println!("{:.3}", mode_from_fig.map(|x| x * 1e6));
            let mode_err = (mode_m1s - mode_from_fig).norm();
            println!(
                "M1S{} mode vector estimate error (x10e6): {:.3}",
                sid + 1,
                mode_err * 1e6
            );
        });

    /*
    println!("{}", *logging.lock().await);
    println!("M1 RBMS (x1e6):");
    (*logging.lock().await)
        .chunks()
        .last()
        .unwrap()
        .chunks(6)
        .take(7)
        .for_each(|x| println!("{:+.3?}", x.iter().map(|x| x * 1e6).collect::<Vec<f64>>()));

    let rbm_residuals = (*logging.lock().await)
        .chunks()
        .last()
        .unwrap()
        .chunks(6)
        .take(7)
        .enumerate()
        .map(|(i, x)| {
            x.iter()
                .enumerate()
                .map(|(j, x)| x * 1e6 - (-1f64).powi((i + j) as i32))
                .map(|x| x * x)
                .sum::<f64>()
                / 6f64
        })
        .sum::<f64>()
        / 7f64;

    println!("M1 RBM set points RSS error: {}", rbm_residuals.sqrt());

    assert!(rbm_residuals.sqrt() < 1e-2);
     */

    Ok(())
}
