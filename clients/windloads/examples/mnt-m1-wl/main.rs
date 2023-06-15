/*
Example aggregating the structural dynamics, mount controller, M1 control system, and wind force disturbances.
R. Romano & R. Conan
*/
use std::{env, path::Path};

use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{OneSignal, Signal, Signals, Smooth, Weight};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_fem::{
    fem_io::{
        actors_inputs::{MCM2Lcl6F, OSSM1Lcl6F, CFD2021106F},
        actors_outputs::{MCM2Lcl6D, OSSM1Lcl},
    },
    DiscreteModalSolver, ExponentialMatrix,
};
use gmt_dos_clients_io::{
    cfd_wind_loads::{CFDM1WindLoads, CFDM2WindLoads, CFDMountWindLoads},
    gmt_m1::M1RigidBodyMotions,
    gmt_m2::M2RigidBodyMotions,
    //mount::MountEncoders,
};
use gmt_dos_clients_mount::Mount;
use gmt_dos_clients_windloads::CfdLoads;
use gmt_fem::FEM;
use parse_monitors::cfd;
// M1 Controller crate
use gmt_dos_clients_m1_ctrl::{Calibration as M1Calibration, Segment as M1Segment};

// - - - SW Constants - - -
const SIM_RATE: usize = 1000;    // Simulation "master" sampling rate
const M1_ACT_RATE: usize = 100;   // M1 controller sampling rate

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let sim_sampling_frequency = SIM_RATE;
    let sim_duration = 1_usize;//40_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    // GMT FEM - placeholder variable
    let mut fem = Option::<FEM>::None; //FEM::from_env()?;
                                       
    // M1 calibration data folder
    let fem_repo_path = env::var("FEM_REPO")?;
    let calib_dt_path = Path::new(&fem_repo_path);
    // M1 Calibration
    let m1_calibration =
        if let Ok(m1_calibration) = M1Calibration::try_from(calib_dt_path.join("m1_calibration.bin")) {
            m1_calibration
        } else {
            let m1_calibration = M1Calibration::new(fem.get_or_insert(FEM::from_env()?));
            println!("Saving M1 calibration data at\n{}", calib_dt_path.display());
            m1_calibration
                .save(calib_dt_path.join("m1_calibration.bin"))
                .expect("failed to save M1 calibration");
            m1_calibration
        };

    // CFD WIND LOADS
    let cfd_repo = env::var("CFD_REPO").expect("CFD_REPO env var missing");
    let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    let path = Path::new(&cfd_repo).join(cfd_case.to_string());
    let cfd_loads_client = CfdLoads::foh(path.to_str().unwrap(), sim_sampling_frequency)
        .duration(sim_duration as f64)
        .mount(fem.get_or_insert(FEM::from_env()?), 0, None)
        .m1_segments()
        .m2_segments()
        .build()?.into_arcx();
    // Segment IDs
    let sids = vec![1, 2, 3, 4, 5, 6, 7];

    // FEM STATE SPACE
    let state_space = {
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem.unwrap_or(FEM::from_env()?))
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            //.max_eigen_frequency(75f64)
            .including_mount()
            .including_m1(Some(sids.clone()))?
            .ins::<CFD2021106F>()
            .ins::<OSSM1Lcl6F>()
            .ins::<MCM2Lcl6F>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .use_static_gain_compensation()
            .build()?
    };
    println!("{state_space}");

    // Structural dynamics model
    let mut plant: Actor<_> = state_space.into();    
    // Initializing model with setpoints for several actors
    let mut setpoints: Model<model::Unknown> = Default::default();

    // MOUNT CONTROL
    let mut mount_setpoint: Initiator<_> = (
        Signals::new(3, n_step), "Mount Setpoint",).into();
    // Creates mount actor and connects it to the mount setpoint
    let mount: Actor<_> = Mount::builder(&mut mount_setpoint).build(&mut plant)?;
    setpoints += mount_setpoint;
    
    // M1 Control
    let mut m1: Model<model::Unknown> = Default::default();
    for &sid in &sids {
        match sid {
            i if i == 1 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), format!("M1S{i}-RBM-SP")).into();
                let mut actuators_setpoint: Initiator<_, M1_ACT_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    format!("M1S{i}-Fact-SP"),
                ).into();
                m1 += M1Segment::<1, M1_ACT_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                ).build(&mut plant)?.name(format!("M1S{i}")).flowchart();
                setpoints += rbm_setpoint + actuators_setpoint;
            }
            i if i == 2 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), format!("M1S{i}-RBM-SP")).into();
                let mut actuators_setpoint: Initiator<_, M1_ACT_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    format!("M1S{i}-Fact-SP"),
                ).into();
                m1 += M1Segment::<2, M1_ACT_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                ).build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;
            }
            i if i == 3 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), format!("M1S{i}-RBM-SP")).into();
                let mut actuators_setpoint: Initiator<_, M1_ACT_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    format!("M1S{i}-Fact-SP"),
                ).into();
                m1 += M1Segment::<3, M1_ACT_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                ).build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;
            }
            i if i == 4 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), format!("M1S{i}-RBM-SP")).into();
                let mut actuators_setpoint: Initiator<_, M1_ACT_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    format!("M1S{i}-Fact-SP"),
                ).into();
                m1 += M1Segment::<4, M1_ACT_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                ).build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;
            }
            i if i == 5 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), format!("M1S{i}-RBM-SP")).into();
                let mut actuators_setpoint: Initiator<_, M1_ACT_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    format!("M1S{i}-Fact-SP"),
                ).into();
                m1 += M1Segment::<5, M1_ACT_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                ).build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;
            }
            i if i == 6 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), format!("M1S{i}-RBM-SP")).into();
                let mut actuators_setpoint: Initiator<_, M1_ACT_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    format!("M1S{i}-Fact-SP"),
                ).into();
                m1 += M1Segment::<6, M1_ACT_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                ).build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;
            }
            i if i == 7 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), format!("M1S{i}-RBM-SP")).into();
                let mut actuators_setpoint: Initiator<_, M1_ACT_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    format!("M1S{i}-Fact-SP"),
                ).into();
                m1 += M1Segment::<7, M1_ACT_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                ).build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;
            }
            _ => unimplemented!("Segments ID must be in the range [1,7]"),
        }
    }

    // Logger
    let logging = Arrow::builder(n_step).build().into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    // CFD wind loads and smoother actors
    let mut cfd_loads: Initiator<_> = Actor::new(cfd_loads_client.clone()).name("CFD Wind loads");
    let mut signals = Signals::new(1, n_step).channel(
        0,
        Signal::Sigmoid {
            amplitude: 1f64,
            sampling_frequency_hz: sim_sampling_frequency as f64,
        },
    );
    signals.progress();
    let signal = OneSignal::try_from(signals)?.into_arcx();
    let m1_smoother = Smooth::new().into_arcx();
    let m2_smoother = Smooth::new().into_arcx();
    let mount_smoother = Smooth::new().into_arcx();

    let mut sigmoid: Initiator<_> = Actor::new(signal.clone()).name("Sigmoid");
    let mut smooth_m1_loads: Actor<_> = Actor::new(m1_smoother.clone());
    let mut smooth_m2_loads: Actor<_> = Actor::new(m2_smoother.clone());
    let mut smooth_mount_loads: Actor<_> = Actor::new(mount_smoother.clone());

    sigmoid
        .add_output()
        .multiplex(3)
        .build::<Weight>()
        .into_input(&mut smooth_m1_loads)
        .into_input(&mut smooth_m2_loads)
        .into_input(&mut smooth_mount_loads)?;
    cfd_loads
        .add_output()
        .build::<CFDM1WindLoads>()
        .into_input(&mut smooth_m1_loads)?;
    smooth_m1_loads
        .add_output()
        .build::<CFDM1WindLoads>()
        .into_input(&mut plant)?;
    cfd_loads
        .add_output()
        .build::<CFDM2WindLoads>()
        .into_input(&mut smooth_m2_loads)?;
    smooth_m2_loads
        .add_output()
        .build::<CFDM2WindLoads>()
        .into_input(&mut plant)?;
    cfd_loads
        .add_output()
        .build::<CFDMountWindLoads>()
        .into_input(&mut smooth_mount_loads)?;
    smooth_mount_loads
        .add_output()
        .build::<CFDMountWindLoads>()
        .into_input(&mut plant)?;
    // M1 & M2 ridig-body motions
    plant.add_output()
        .unbounded()
        .build::<M1RigidBodyMotions>()
        .log(&mut sink)
        .await?;
    plant.add_output()
        .unbounded()
        .build::<M2RigidBodyMotions>()
        .log(&mut sink)
        .await?;

    let im_model =
    (model!(cfd_loads, sigmoid, smooth_m1_loads, smooth_m2_loads,
        smooth_mount_loads, plant, sink) + m1 + mount + setpoints)
        .name("mnt-m1-wl")
        .flowchart()
        .check()?
        .run()
        .wait();
        
    im_model.await?;
    Ok(())
}
