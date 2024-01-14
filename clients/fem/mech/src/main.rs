use gmt_dos_actors::actorscript;
use gmt_dos_clients::multiplex::Multiplex;
use gmt_dos_clients::{
    operator::{Left, Operator, Right},
    Gain, Integrator, Signal, Signals,
};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_fem::outputs::MCM2RB6D;
use gmt_dos_clients_io::gmt_m2::asm::M2ASMReferenceBodyNodes;
use gmt_dos_clients_io::{
    gmt_fem::{
        inputs::MCM2SmHexF,
        outputs::{M2EdgeSensors, MCM2Lcl6D, MCM2SmHexD, OSSM1EdgeSensors, OSSM1Lcl},
    },
    gmt_m1::{assembly, M1RigidBodyMotions},
    gmt_m2::{
        asm::{
            segment::VoiceCoilsMotion, M2ASMAsmCommand, M2ASMFluidDampingForces,
            M2ASMVoiceCoilsForces, M2ASMVoiceCoilsMotion,
        },
        M2PositionerForces, M2PositionerNodes, M2RigidBodyMotions,
    },
    mount::{MountEncoders, MountSetPoint, MountTorques},
    optics::{M2modes, Wavefront, WfeRms},
};
use gmt_dos_clients_lom::{LinearOpticalModel, OpticalSensitivities};
use gmt_dos_clients_m1_ctrl::{Calibration, M1};
use gmt_dos_clients_m2_ctrl::{positioner::AsmsPositioners, ASMS};
use gmt_dos_clients_mount::Mount;
use gmt_fem::FEM;
use interface::{Write, UID};
use matio_rs::MatFile;
use nalgebra as na;
use std::{env, path::Path};

const ACTUATOR_RATE: usize = 80;

#[derive(UID)]
pub enum RBMCmd {}

#[derive(UID)]
pub enum ActuatorCmd {}

#[derive(UID)]
pub enum AsmCmd {}

#[derive(UID)]
#[alias(name = WfeRms<-9>, port = 55991, client = LinearOpticalModel, traits = Write)]
pub enum M1RbmWfeRms {}

#[derive(UID)]
#[alias(name = WfeRms<-9>, port = 55992, client = LinearOpticalModel, traits = Write)]
pub enum AsmShellWfeRms {}

#[derive(UID)]
#[alias(name = WfeRms<-9>, port = 55993, client = LinearOpticalModel, traits = Write)]
pub enum AsmRefBodyWfeRms {}

// export FEM_REPO=/home/rconan/mnt/20230131_1605_zen_30_M1_202110_ASM_202208_Mount_202111/

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder().format_timestamp(None).init();

    let data_repo = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("data");
    env::set_var("DATA_REPO", &data_repo);

    let sim_sampling_frequency = 8000;
    let m1_freq = 100; // Hz
    assert!(m1_freq == sim_sampling_frequency / ACTUATOR_RATE);
    let sim_duration = 3_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    // GMT FEM
    let mut fem = FEM::from_env()?;
    // println!("{fem}");
    // M1 CONTROLLERS
    let m1_calibration = Calibration::new(&mut fem);
    let m1 = M1::<ACTUATOR_RATE>::new(&m1_calibration)?;
    // ASMS POSITIONER CONTROLLERS
    let positioners = AsmsPositioners::from_fem(&mut fem)?;
    // ASMS FACESHEETS CONTROLLERS
    let n_mode = 6;
    let asms = ASMS::<1>::from_fem(&mut fem, Some(vec![n_mode; 7]))?;

    // KARHUNEN-LOEVE MODES
    let fem_var = env::var("FEM_REPO").expect("`FEM_REPO` is not set");
    let fem_path = Path::new(&fem_var);
    let path = fem_path.join("KLmodesGS36p90.mat");
    let mut kl_modes: Vec<na::DMatrix<f64>> = vec![];
    for i in 1..=7 {
        let mat: na::DMatrix<f64> = MatFile::load(&path)?.var(format!("KL_{i}"))?;
        let (_nact, nkl) = mat.shape();
        assert!(n_mode <= nkl);
        kl_modes.push(mat.remove_columns(n_mode, nkl - n_mode));
    }

    // EDGE SENSORS
    //  * M1 EDGE SENSORS NODES
    let es_nodes_2_data: na::DMatrix<f64> =
        MatFile::load(fem_path.join("M1_edge_sensor_conversion.mat"))?.var("A1")?;
    //  * EDGE SENSORS TO RIGID-BODY MOTIONS TRANSFORM (M1 & M2)
    let (es_2_m1_rbm, es_2_m2_rbm) = {
        let mat = MatFile::load(fem_path.join("m12_e_rs").join("m12_r_es.mat"))?;
        let m1_es_recon: na::DMatrix<f64> = mat.var("m1_r_es")?;
        let m2_es_recon: na::DMatrix<f64> = mat.var("m2_r_es")?;
        (
            m1_es_recon.insert_rows(36, 6, 0f64) * es_nodes_2_data,
            m2_es_recon.insert_rows(36, 6, 0f64),
        )
    };
    //  * M1 RIGID-BODY MOTIONS TO ASMS KARHUNEN-LOEVE MODES TRANSFORM
    let m1_rbm_2_mode: na::DMatrix<f64> =
        MatFile::load(&fem_path.join("m1_rbm_2_asm_kl.mat"))?.var("r2kl")?;
    dbg!(m1_rbm_2_mode.shape());
    //  * M2 RIGID-BODY MOTIONS TO ASMS KARHUNEN-LOEVE MODES TRANSFORM
    let mut m2_rbm_2_mode: na::DMatrix<f64> =
        MatFile::load(&fem_path.join("m2_rbm_2_asm_kl.mat"))?.var("r2kl")?;
    // for i in 0..6 {
    //     m2_rbm_2_mode.swap_columns(i + 3, i + 4);
    // }
    dbg!(m2_rbm_2_mode.shape());
    // let es_2_m2_mode = &m2_rbm_2_mode * &es_2_m2_rbm;
    // dbg!(es_2_m2_mode.shape());

    // let mat = MatFile::load(&fem_path.join("rbm_2_asm_vcf.mat"))?;
    // let rbm_2_voice_coil_forces: na::DMatrix<f64> = mat.var("r2vcf")?;

    // let vcd_2_kl: Vec<_> = kl_modes_t
    //     .iter()
    //     .map(|mat| mat.view((0, 0), (6, 675)))
    //     .collect();

    let kl_modes_t = kl_modes.iter().map(|x| x.transpose()).collect::<Vec<_>>();
    let sids = vec![1, 2, 3, 4, 5, 6, 7];
    let fem_dss = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        // .truncate_hankel_singular_values(1e-7)
        // .hankel_frequency_lower_bound(50.)
        .including_mount()
        .including_m1(Some(sids.clone()))?
        .including_asms(
            Some(sids.clone()),
            Some(kl_modes.iter().map(|x| x.as_view()).collect()),
            Some(kl_modes_t.iter().map(|x| x.as_view()).collect()),
        )?
        .ins::<MCM2SmHexF>()
        .outs::<MCM2SmHexD>()
        .outs::<OSSM1Lcl>()
        .outs::<MCM2Lcl6D>()
        .outs::<MCM2RB6D>()
        .outs_with::<OSSM1EdgeSensors>(es_2_m1_rbm.as_view())
        .outs_with::<M2EdgeSensors>(es_2_m2_rbm.as_view())
        .use_static_gain_compensation()
        .build()?;
    println!("{fem_dss}");

    let plant = fem_dss;
    // .image("../icons/fem.png");

    let rbm_fun =
        |i: usize, sid: u8| (-1f64).powi(i as i32) * (1 + (i % 3)) as f64 + sid as f64 / 10_f64;
    /*     let rbm = (1..=6).fold(Signals::new(6 * 7, 2 * n_step), |signals_sid, sid| {
        (0..6).fold(signals_sid, |signals, i| {
            signals.channel(
                i + 6 * (sid - 1) as usize,
                Signal::Sigmoid {
                    amplitude: rbm_fun(i, sid) * 1e-6,
                    sampling_frequency_hz: sim_sampling_frequency as f64,
                },
            )
        })
    }); */
    let rbm = Signals::new(6 * 7, n_step * 5).channel(
        2,
        Signal::Sigmoid {
            amplitude: 1e-6,
            sampling_frequency_hz: sim_sampling_frequency as f64,
        },
    );
    /*     let mut rng = WyRand::new();
    let rbm = (1..=6).fold(Signals::new(6 * 7, 2 * n_step), |signals_sid, sid| {
        [2, 3, 4].into_iter().fold(signals_sid, |signals, i| {
            signals.channel(
                i + 6 * (sid - 1) as usize,
                Signal::Sigmoid {
                    amplitude: 1e-6 * (2. * rng.generate::<f64>() - 1.),
                    sampling_frequency_hz: sim_sampling_frequency as f64,
                },
            )
        })
    }); */

    let actuators = Signals::new(6 * 335 + 306, 10 * n_step);
    let actuators_mx = Multiplex::new(vec![335, 335, 335, 335, 335, 335, 306]);

    let rbm_mx = Multiplex::new(vec![6; 7]);

    /*     let mut m1 = SubSystem::new(M1::<ACTUATOR_RATE>::new(calibration)?)
    .name("M1 Control")
    .build()?
    .flowchart(); */
    // let mut m1 = Sys::new(M1::<ACTUATOR_RATE>::new(calibration)?).build()?;

    // let mut m1_clone = m1.clone();

    // MOUNT CONTROL
    let mount_setpoint = Signals::new(3, n_step);
    let mount = Mount::new();

    let lom = LinearOpticalModel::new()?;
    // let m2_lom = OpticalSensitivities::<42>::new(
    //     data_repo.join("M2_OrthoNormGS36p_KarhunenLoeveModes#6-optical_sensitivities.rs.bin"),
    // )?;

    let asms_mx = Multiplex::new(vec![6; 7]);
    let m2_rbm = Signals::new(6 * 7, n_step);
    let asm_cmd = Signals::new(n_mode * 7, n_step);

    actorscript! {
        #[model(name = warmup, state = completed)]
        #[labels(plant = "GMT FEM",  mount = "Mount Control")]

        1: mount_setpoint[MountSetPoint] -> mount[MountTorques] -> plant[MountEncoders]! -> mount

        1: rbm[Right<RBMCmd>] -> rbm_mx[assembly::M1RigidBodyMotions]
            -> {m1}[assembly::M1HardpointsForces]
                -> plant[assembly::M1HardpointsMotion]! -> {m1}
        1: actuators[ActuatorCmd]
            -> actuators_mx[assembly::M1ActuatorCommandForces]
                -> {m1}[assembly::M1ActuatorAppliedForces] -> plant

        1: m2_rbm[M2RigidBodyMotions] -> positioners[M2PositionerForces] -> plant[M2PositionerNodes]! -> positioners

        1: asm_cmd[AsmCmd] -> asms_mx[M2ASMAsmCommand]
                 -> {asms}[M2ASMVoiceCoilsForces]-> plant
        1: {asms}[M2ASMFluidDampingForces] -> plant[M2ASMVoiceCoilsMotion]! -> {asms}

        4000: plant[M1RigidBodyMotions] -> lom[Wavefront]${262144}

    }

    {
        let mut plant_lock = plant.lock().await;

        println!("RIGID BODY MOTIONS:");
        let rbms = <DiscreteModalSolver<ExponentialMatrix> as Write<M1RigidBodyMotions>>::write(
            &mut plant_lock,
        )
        .unwrap();

        let rbm_err = rbms
            .chunks(6)
            .map(|x| x.iter().map(|x| x * 1e6).collect::<Vec<_>>())
            .enumerate()
            .inspect(|(i, x)| println!("{:2}: {:+.1?}", i, x))
            .map(|(i, x)| {
                x.iter()
                    .enumerate()
                    .map(|(j, x)| x - rbm_fun(j, i as u8 + 1))
                    .map(|x| x * x)
                    .sum::<f64>()
                    / 6f64
            })
            .map(|x| x.sqrt())
            .sum::<f64>()
            / 7f64;
        // assert!(dbg!(rbm_err) < 5e-2);

        println!("EDGE SENSORS:");
        let es = <DiscreteModalSolver<ExponentialMatrix> as Write<OSSM1EdgeSensors>>::write(
            &mut plant_lock,
        )
        .unwrap();
        es.chunks(6)
            .map(|x| x.iter().map(|x| x * 1e6).collect::<Vec<_>>())
            .enumerate()
            .for_each(|(i, x)| println!("{:2}: {:+.1?}", i, x));
    }

    let mount_setpoint = Signals::new(3, dbg!(n_step));

    let m1_es_int = Integrator::new(42).gain(0.2);
    let m2_es_int = Integrator::new(42).gain(0.2);
    let m1_add = Operator::new("+");
    let m2_add = Operator::new("+");
    // dbg!(rbm_2_voice_coil_forces.shape());
    let m1_rbm_2_kls = Gain::new(
        m1_rbm_2_mode
            .insert_columns(36, 6, 0f64)
            .insert_rows(36, 6, 0f64),
    );
    let m2_rbm_2_kls = Gain::new(
        m2_rbm_2_mode
            .insert_columns(36, 6, 0f64)
            .insert_rows(36, 6, 0f64),
    );
    // let gain = Gain::new(rbm_2_voice_coil_forces.insert_columns(36, 6, 0f64));
    // let m1s1_vcd_2_kl = Gain::new(kl_modes_t[0].clone());

    // let print = Print::default();
    let m2_rbm = Signals::new(6 * 7, n_step).channel(2, Signal::Constant(1e-6));

    // let m2_es_int = Integrator::new(n_mode * 7).gain(0.2);
    let add_asm_cmd = Operator::new("+");

    // LOM
    let lom = LinearOpticalModel::new()?;
    let m1_lom = LinearOpticalModel::new()?;
    let asm_shell_lom = LinearOpticalModel::new()?;
    let asm_rb_lom = LinearOpticalModel::new()?;

    actorscript! {
        #[model(name = model, state = completed)]
        #[labels(plant = "GMT FEM", mount = "Mount Control",
            m1_add = "Add M1\nRBM", m2_add = "Add M2\nRBM",
            m1_rbm_2_kls = "M1 RBM\nto\nASMS KLS",
            m2_rbm_2_kls = "M2 RBM\nto\nASMS KLS",
            add_asm_cmd = "Add M1 & M2\nedge sensors (as KLs)")]
        #[images(plant = "gmt-fem.png")]

        // mount feed
        1: mount_setpoint[MountSetPoint] -> mount[MountTorques] -> plant[MountEncoders]! -> mount

        1: rbm[Right<RBMCmd>] -> m1_add[RBMCmd]
            -> rbm_mx[assembly::M1RigidBodyMotions]
                -> {m1}[assembly::M1HardpointsForces]
                    -> plant[assembly::M1HardpointsMotion]! -> {m1}
        1: actuators[ActuatorCmd]
            -> actuators_mx[assembly::M1ActuatorCommandForces]
                -> {m1}[assembly::M1ActuatorAppliedForces] -> plant

        1: plant[OSSM1EdgeSensors]!
            -> m1_rbm_2_kls
        8: m1_rbm_2_kls[Right<M2modes>] -> add_asm_cmd[M2modes] -> asms_mx

        1000: plant[OSSM1EdgeSensors]! -> m1_es_int
        1: m1_es_int[Left<OSSM1EdgeSensors>]! -> m1_add

        1: m2_rbm[Right<M2RigidBodyMotions>] ->  m2_add[M2RigidBodyMotions]
                -> positioners[M2PositionerForces] -> plant[M2PositionerNodes]! -> positioners

        1: asms_mx[M2ASMAsmCommand]
                 -> {asms}[M2ASMVoiceCoilsForces]-> plant
        1: {asms}[M2ASMFluidDampingForces] -> plant[M2ASMVoiceCoilsMotion]! -> {asms}

        500: plant[M2EdgeSensors]! -> m2_es_int
        1: m2_es_int[Left<M2RigidBodyMotions>]!->  m2_add
        1: plant[M2EdgeSensors]! -> m2_rbm_2_kls
        8: m2_rbm_2_kls[Left<M2EdgeSensors>] ->  add_asm_cmd

        // 8: plant[VoiceCoilsMotion<1>]~//${6}

        32: lom[WfeRms<-9>]~
        250: lom[Wavefront]${262144}
        1: plant[M1RigidBodyMotions] -> lom
        1: plant[M2RigidBodyMotions] -> lom

        32: m1_lom[M1RbmWfeRms]~
        1: plant[M1RigidBodyMotions] -> m1_lom

        32: asm_shell_lom[AsmShellWfeRms]~
        1: plant[M2RigidBodyMotions] -> asm_shell_lom

        32: asm_rb_lom[AsmRefBodyWfeRms]~
        1: plant[M2ASMReferenceBodyNodes] -> asm_rb_lom

    }

    let mut plant_lock = plant.lock().await;

    println!("RIGID BODY MOTIONS:");
    let rbms = <DiscreteModalSolver<ExponentialMatrix> as Write<M1RigidBodyMotions>>::write(
        &mut plant_lock,
    )
    .unwrap();

    let _rbm_err = rbms
        .chunks(6)
        .map(|x| x.iter().map(|x| x * 1e6).collect::<Vec<_>>())
        .enumerate()
        .inspect(|(i, x)| println!("{:2}: {:+.1?}", i, x))
        .map(|(i, x)| {
            x.iter()
                .enumerate()
                .map(|(j, x)| x - rbm_fun(j, i as u8 + 1))
                .map(|x| x * x)
                .sum::<f64>()
                / 6f64
        })
        .map(|x| x.sqrt())
        .sum::<f64>()
        / 7f64;
    // assert!(dbg!(rbm_err) < 5e-2);

    println!("EDGE SENSORS:");
    let es =
        <DiscreteModalSolver<ExponentialMatrix> as Write<OSSM1EdgeSensors>>::write(&mut plant_lock)
            .unwrap();
    es.chunks(6)
        .map(|x| x.iter().map(|x| x * 1e6).collect::<Vec<_>>())
        .enumerate()
        .for_each(|(i, x)| println!("{:2}: {:+.1?}", i, x));

    Ok(())
}
