use gmt_dos_actors::actorscript;
use gmt_dos_clients::{
    fill::Fill,
    low_pass_filter::LowPassFilter,
    operator::{Left, Operator, Right},
    select::Select,
    Gain, Integrator, OneSignal, Signal, Signals, Smooth, Weight,
};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::{
    cfd_wind_loads::{CFDM1WindLoads, CFDM2WindLoads, CFDMountWindLoads},
    gmt_fem::{
        inputs::{MCM2Lcl6F, MCM2SmHexF, OSSM1Lcl6F, CFD2021106F},
        outputs::{M2EdgeSensors, MCM2Lcl6D, MCM2SmHexD, OSSM1EdgeSensors, OSSM1Lcl, MCM2RB6D},
    },
    gmt_m1::{assembly, M1RigidBodyMotions},
    gmt_m2::{
        asm::{
            M2ASMAsmCommand, M2ASMFluidDampingForces, M2ASMReferenceBodyNodes,
            M2ASMVoiceCoilsForces, M2ASMVoiceCoilsMotion,
        },
        M2PositionerForces, M2PositionerNodes, M2RigidBodyMotions,
    },
    mount::{AverageMountEncoders, MountEncoders, MountSetPoint, MountTorques},
    optics::{M2modes, SegmentD21PistonRSS, SegmentTipTilt, Wavefront, WfeRms},
};
use gmt_dos_clients_lom::LinearOpticalModel;
use gmt_dos_clients_m1_ctrl::{Calibration, M1};
use gmt_dos_clients_m2_ctrl::{positioner::AsmsPositioners, ASMS};
use gmt_dos_clients_mount::Mount;
use gmt_dos_clients_scope::server::{Monitor, Scope};
use gmt_dos_clients_windloads::CfdLoads;
use gmt_fem::FEM;
use interface::{units::Mas, UID};
use matio_rs::MatFile;
use mechanics::{
    AsmRefBodyWfeRms, AsmShellSegmentD21PistonRSS, AsmShellWfeRms, M1RbmSegmentD21PistonRSS,
    M1RbmWfeRms,
};
use nalgebra as na;
use std::{env, path::Path};

const ACTUATOR_RATE: usize = 80;

#[derive(UID)]
pub enum RBMCmd {}

#[derive(UID)]
pub enum ActuatorCmd {}

#[derive(UID)]
pub enum AsmCmd {}

const SCOPE_SERVER_IP: &'static str = "127.0.0.1";

// export FLOWCHART=sfdp
// export FEM_REPO=/home/rconan/mnt/20230131_1605_zen_30_M1_202110_ASM_202208_Mount_202111/

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder().format_timestamp(None).init();

    let data_repo = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("data");
    env::set_var("DATA_REPO", &data_repo);
    let fem_var = env::var("FEM_REPO").expect("`FEM_REPO` is not set");
    let fem_path = Path::new(&fem_var);

    let sim_sampling_frequency = 8000;
    let m1_freq = 100; // Hz
    assert!(m1_freq == sim_sampling_frequency / ACTUATOR_RATE);
    let sim_duration = 4_usize; // second
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
    // CFD WIND LOADS
    let cfd_loads = CfdLoads::foh(fem_path.to_str().unwrap(), sim_sampling_frequency)
        .duration(120_f64)
        .mount(&mut fem, 0, None)
        .m1_segments()
        .m2_segments()
        .build()?;

    // KARHUNEN-LOEVE MODES
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
    let m2_rbm_2_mode: na::DMatrix<f64> =
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

    // FEM STATE SPACE
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
        .ins::<CFD2021106F>()
        .ins::<OSSM1Lcl6F>()
        .ins::<MCM2Lcl6F>()
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

    let rbm = Signals::new(6 * 7, n_step);
    /* .channel(
        2,
        Signal::Sigmoid {
            amplitude: 1e-6,
            sampling_frequency_hz: sim_sampling_frequency as f64,
        },
    ); */
    let actuators = Signals::new(6 * 335 + 306, n_step);

    // MOUNT CONTROL
    let mount_setpoint = Signals::new(3, n_step);
    let mount = Mount::new();

    let m2_rbm = Signals::new(6 * 7, n_step);
    let asm_cmd = Signals::new(n_mode * 7, n_step);

    let sigmoid = OneSignal::try_from(Signals::new(1, n_step).channel(
        0,
        Signal::Sigmoid {
            amplitude: 1f64,
            sampling_frequency_hz: sim_sampling_frequency as f64,
        },
    ))?;
    let m1_smoother = Smooth::new();
    let m2_smoother = Smooth::new();
    let mount_smoother = Smooth::new();

    // LINEAR OPTICAL MODELS
    let lom = LinearOpticalModel::new()?;
    let m1_lom = LinearOpticalModel::new()?;
    let asm_shell_lom = LinearOpticalModel::new()?;
    let asm_rb_lom = LinearOpticalModel::new()?;

    // SCOPES
    let server_ip = env::var("SCOPE_SERVER_IP").unwrap_or(SCOPE_SERVER_IP.into());
    let mut monitor = Monitor::new();
    //  * WFE RMS
    let scope_sampling_frequency = sim_sampling_frequency / 32;
    let wfe_rms_scope = Scope::<WfeRms<-9>>::builder(&server_ip, &mut monitor)
        .sampling_frequency(scope_sampling_frequency as f64)
        .build()?;
    let m1_rbm_wfe_rms_scope = Scope::<M1RbmWfeRms>::builder(&server_ip, &mut monitor)
        .sampling_frequency(scope_sampling_frequency as f64)
        .build()?;
    let asm_shell_wfe_rms_scope = Scope::<AsmShellWfeRms>::builder(&server_ip, &mut monitor)
        .sampling_frequency(scope_sampling_frequency as f64)
        .build()?;
    let asm_refbody_wfe_rms_scope = Scope::<AsmRefBodyWfeRms>::builder(&server_ip, &mut monitor)
        .sampling_frequency(scope_sampling_frequency as f64)
        .build()?;
    //  * diffential piston
    let dp21rss_scope = Scope::<SegmentD21PistonRSS<-9>>::builder(&server_ip, &mut monitor)
        .sampling_frequency(scope_sampling_frequency as f64)
        .build()?;
    let m1_rbm_dp21rss_scope = Scope::<M1RbmSegmentD21PistonRSS>::builder(&server_ip, &mut monitor)
        .sampling_frequency(scope_sampling_frequency as f64)
        .build()?;
    let asm_shell_dp21rss_scope =
        Scope::<AsmShellSegmentD21PistonRSS>::builder(&server_ip, &mut monitor)
            .sampling_frequency(scope_sampling_frequency as f64)
            .build()?;
    //  * segment tip-tilt
    let stt_scope = Scope::<Mas<SegmentTipTilt>>::builder(&server_ip, &mut monitor)
        .sampling_frequency(scope_sampling_frequency as f64)
        .build()?;

    let mount_scope = Scope::<AverageMountEncoders<-6>>::builder(&server_ip, &mut monitor)
        .sampling_frequency(scope_sampling_frequency as f64)
        .build()?;

    actorscript! {
        #[model(name = warmup, state = completed)]
        #[labels(plant = "GMT FEM",  mount = "Mount Control")]

        // mount feedback loop
        1: mount_setpoint[MountSetPoint] -> mount[MountTorques] -> plant[MountEncoders]! -> mount
        // CFD wind loads
        //  * M1 loads
        1: cfd_loads[CFDM1WindLoads] -> m1_smoother
        1: sigmoid[Weight] -> m1_smoother[CFDM1WindLoads] -> plant
        //  * M2 loads
        1: cfd_loads[CFDM2WindLoads] -> m2_smoother
        1: sigmoid[Weight] -> m2_smoother[CFDM2WindLoads] -> plant
        //  * mount loads
        1: cfd_loads[CFDMountWindLoads] -> mount_smoother
        1: sigmoid[Weight] -> mount_smoother[CFDMountWindLoads] -> plant
        // M1 hardpoints/actuators force loop
        1: rbm[assembly::M1RigidBodyMotions]
                -> {m1}[assembly::M1HardpointsForces]
                    -> plant[assembly::M1HardpointsMotion]! -> {m1}
        1: actuators[assembly::M1ActuatorCommandForces]
                -> {m1}[assembly::M1ActuatorAppliedForces] -> plant
        // ASMS positioners feedback loop
        1: m2_rbm[M2RigidBodyMotions] -> positioners[M2PositionerForces] -> plant[M2PositionerNodes]! -> positioners
        // ASMS voice coils feedback loop
        1: asm_cmd[M2ASMAsmCommand]
                 -> {asms}[M2ASMVoiceCoilsForces]-> plant
        1: {asms}[M2ASMFluidDampingForces] -> plant[M2ASMVoiceCoilsMotion]! -> {asms}

        // MONITORING
        32: lom[WfeRms<-9>].. -> wfe_rms_scope
        32: lom[Mas<SegmentTipTilt>].. -> stt_scope
        32: lom[SegmentD21PistonRSS<-9>].. -> dp21rss_scope
        250: lom[Wavefront]${262144}
        1: plant[M1RigidBodyMotions].. -> lom
        1: plant[M2RigidBodyMotions].. -> lom

        32: m1_lom[M1RbmWfeRms].. -> m1_rbm_wfe_rms_scope
        32: m1_lom[M1RbmSegmentD21PistonRSS].. -> m1_rbm_dp21rss_scope
        1: plant[M1RigidBodyMotions].. -> m1_lom

        32: asm_shell_lom[AsmShellWfeRms].. -> asm_shell_wfe_rms_scope
        32: asm_shell_lom[AsmShellSegmentD21PistonRSS].. -> asm_shell_dp21rss_scope
        1: plant[M2RigidBodyMotions].. -> asm_shell_lom

        32: asm_rb_lom[AsmRefBodyWfeRms].. -> asm_refbody_wfe_rms_scope
        1: plant[M2ASMReferenceBodyNodes].. -> asm_rb_lom

        32: plant[AverageMountEncoders<-6>].. -> mount_scope

    }

    let sim_duration = 2_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let mount_setpoint = Signals::new(3, n_step);

    let rbm = Signals::new(6 * 7, n_step);
    let actuators = Signals::new(6 * 335 + 306, n_step);
    let m2_rbm = Signals::new(6 * 7, n_step); //.channel(2, Signal::Constant(1e-6));

    // EDGE SENSORS INTEGRAL CONTROLLERS:
    //  * M1
    let m1_es_int = Integrator::new(42).gain(0.2);
    //  * M2
    let m2_es_int = Integrator::new(42).gain(0.2);

    let m1_add = Operator::new("+");
    let m2_add = Operator::new("+");

    // RIGID-BODY MOTIONS 2 ASMS KARHUNEN-LOEVE MODES TRANSFORM
    //  * M1
    let m1_rbm_2_kls = Gain::new(
        m1_rbm_2_mode
            .insert_columns(36, 6, 0f64)
            .insert_rows(36, 6, 0f64),
    );
    //  * M2
    let m2_rbm_2_kls = Gain::new(
        m2_rbm_2_mode
            .insert_columns(36, 6, 0f64)
            .insert_rows(36, 6, 0f64),
    );

    let add_asm_cmd = Operator::new("+");

    actorscript! {
        #[model(name = model, state = completed)]
        #[labels(plant = "GMT FEM", mount = "Mount Control",
            m1_add = "Add M1\nRBM", m2_add = "Add M2\nRBM",
            m1_rbm_2_kls = "M1 RBM\nto\nASMS KLS",
            m2_rbm_2_kls = "M2 RBM\nto\nASMS KLS",
            add_asm_cmd = "Add M1 & M2\nedge sensors (as KLs)")]
        #[images(plant = "gmt-fem.png")]

        // mount feedback loop
        1: mount_setpoint[MountSetPoint] -> mount[MountTorques] -> plant[MountEncoders]! -> mount
        // CFD wind loads
        //  * M1 loads
        1: cfd_loads[CFDM1WindLoads] -> plant
        //  * M2 loads
        1: cfd_loads[CFDM2WindLoads] -> plant
        //  * mount loads
        1: cfd_loads[CFDMountWindLoads] -> plant
        // M1 hardpoints/actuators force loop
        1: rbm[Right<RBMCmd>] -> m1_add[assembly::M1RigidBodyMotions]
                -> {m1}[assembly::M1HardpointsForces]
                    -> plant[assembly::M1HardpointsMotion]! -> {m1}
        1: actuators[assembly::M1ActuatorCommandForces]
                -> {m1}[assembly::M1ActuatorAppliedForces] -> plant
        // M1 edge sensors feed-forward loop to ASMS KL modes
        1: plant[OSSM1EdgeSensors]!
            -> m1_rbm_2_kls
        8: m1_rbm_2_kls[Right<M2modes>] -> add_asm_cmd
        // M1 edge sensors feedback loop to rigid body motions
        1000: plant[OSSM1EdgeSensors]! -> m1_es_int
        1: m1_es_int[Left<OSSM1EdgeSensors>]! -> m1_add
        // ASMS positioners feedback loop
        1: m2_rbm[Right<M2RigidBodyMotions>] ->  m2_add[M2RigidBodyMotions]
                -> positioners[M2PositionerForces] -> plant[M2PositionerNodes]! -> positioners
        // ASMS voice coils feedback loop
        1: add_asm_cmd[M2ASMAsmCommand]
                 -> {asms}[M2ASMVoiceCoilsForces]-> plant
        1: {asms}[M2ASMFluidDampingForces] -> plant[M2ASMVoiceCoilsMotion]! -> {asms}
        // ASMS edge sensors feedback loop to ASMS positioners
        500: plant[M2EdgeSensors]! -> m2_es_int
        1: m2_es_int[Left<M2RigidBodyMotions>]!->  m2_add
         // M2 edge sensors feed-forward loop to ASMS KL modes
        1: plant[M2EdgeSensors]! -> m2_rbm_2_kls
        8: m2_rbm_2_kls[Left<M2EdgeSensors>] ->  add_asm_cmd

        // MONITORING
        32: lom[WfeRms<-9>].. -> wfe_rms_scope
        32: lom[Mas<SegmentTipTilt>].. -> stt_scope
        32: lom[SegmentD21PistonRSS<-9>].. -> dp21rss_scope
        250: lom[Wavefront]${262144}
        1: plant[M1RigidBodyMotions].. -> lom
        1: plant[M2RigidBodyMotions].. -> lom

        32: m1_lom[M1RbmWfeRms].. -> m1_rbm_wfe_rms_scope
        32: m1_lom[M1RbmSegmentD21PistonRSS].. -> m1_rbm_dp21rss_scope
        1: plant[M1RigidBodyMotions].. -> m1_lom

        32: asm_shell_lom[AsmShellWfeRms].. -> asm_shell_wfe_rms_scope
        32: asm_shell_lom[AsmShellSegmentD21PistonRSS].. -> asm_shell_dp21rss_scope
        1: plant[M2RigidBodyMotions].. -> asm_shell_lom

        32: asm_rb_lom[AsmRefBodyWfeRms].. -> asm_refbody_wfe_rms_scope
        1: plant[M2ASMReferenceBodyNodes].. -> asm_rb_lom

        32: plant[AverageMountEncoders<-6>].. -> mount_scope

    }

    let sim_duration = 12_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let mount_setpoint = Signals::new(3, n_step);
    let rbm = Signals::new(6 * 7, n_step);
    let actuators = Signals::new(6 * 335 + 306, n_step);
    let m2_rbm = Signals::new(6 * 7, n_step); //.channel(2, Signal::Constant(1e-

    let lpf = LowPassFilter::new(2, 0.01);
    let grab = Select::<f64>::new(vec![13, 6]);
    let fill = Fill::new(0.0, 3);

    let int = Integrator::new(2).gain(0.1);
    let mount_add = Operator::new("-");

    let oiwfs = LinearOpticalModel::new()?;

    actorscript! {
        #[model(name = mount_offload, state = completed)]
        #[labels(plant = "GMT FEM", mount = "Mount Control",
            m1_add = "Add M1\nRBM", m2_add = "Add M2\nRBM",
            m1_rbm_2_kls = "M1 RBM\nto\nASMS KLS",
            m2_rbm_2_kls = "M2 RBM\nto\nASMS KLS",
            add_asm_cmd = "Add M1 & M2\nedge sensors (as KLs)",
            oiwfs = "On-instrument\nWFS")]
        #[images(plant = "gmt-fem.png")]

        // mount feedback loop
        1: mount_setpoint[Right<MountSetPoint>] -> mount_add[MountSetPoint]
            -> mount[MountTorques] -> plant[MountEncoders]! -> mount
        // CFD wind loads
        //  * M1 loads
        1: cfd_loads[CFDM1WindLoads] -> plant
        //  * M2 loads
        1: cfd_loads[CFDM2WindLoads] -> plant
        //  * mount loads
        1: cfd_loads[CFDMountWindLoads] -> plant
        // M1 hardpoints/actuators force loop
        1: rbm[Right<RBMCmd>] -> m1_add[assembly::M1RigidBodyMotions]
                -> {m1}[assembly::M1HardpointsForces]
                    -> plant[assembly::M1HardpointsMotion]! -> {m1}
        1: actuators[assembly::M1ActuatorCommandForces]
                -> {m1}[assembly::M1ActuatorAppliedForces] -> plant
        // M1 edge sensors feed-forward loop to ASMS KL modes
        1: plant[OSSM1EdgeSensors]!
            -> m1_rbm_2_kls
        8: m1_rbm_2_kls[Right<M2modes>] -> add_asm_cmd
        // M1 edge sensors feedback loop to rigid body motions
        1000: plant[OSSM1EdgeSensors]! -> m1_es_int
        1: m1_es_int[Left<OSSM1EdgeSensors>]! -> m1_add
        // ASMS positioners feedback loop
        1: m2_rbm[Right<M2RigidBodyMotions>] ->  m2_add[M2RigidBodyMotions]
                -> positioners[M2PositionerForces] -> plant[M2PositionerNodes]! -> positioners
        // ASMS voice coils feedback loop
        1: add_asm_cmd[M2ASMAsmCommand]
                 -> {asms}[M2ASMVoiceCoilsForces]-> plant
        1: {asms}[M2ASMFluidDampingForces] -> plant[M2ASMVoiceCoilsMotion]! -> {asms}
        // ASMS edge sensors feedback loop to ASMS positioners
        500: plant[M2EdgeSensors]! -> m2_es_int
        1: m2_es_int[Left<M2RigidBodyMotions>]!->  m2_add
         // M2 edge sensors feed-forward loop to ASMS KL modes
        1: plant[M2EdgeSensors]! -> m2_rbm_2_kls
        8: m2_rbm_2_kls[Left<M2EdgeSensors>] ->  add_asm_cmd

        1: plant[M1RigidBodyMotions] -> oiwfs
        1: plant[M2RigidBodyMotions] -> oiwfs
        1: oiwfs[SegmentTipTilt] -> grab
        8000: grab[SegmentTiptilt7]-> int[LpfIntSegmentTiptilt]!
                -> fill[Left<MountSetPoint>] -> mount_add

        // MONITORING
        32: lom[WfeRms<-9>].. -> wfe_rms_scope
        32: lom[Mas<SegmentTipTilt>].. -> stt_scope
        32: lom[SegmentD21PistonRSS<-9>].. -> dp21rss_scope
        250: lom[Wavefront]${262144}
        1: plant[M1RigidBodyMotions].. -> lom
        1: plant[M2RigidBodyMotions].. -> lom

        32: m1_lom[M1RbmWfeRms].. -> m1_rbm_wfe_rms_scope
        32: m1_lom[M1RbmSegmentD21PistonRSS].. -> m1_rbm_dp21rss_scope
        1: plant[M1RigidBodyMotions].. -> m1_lom

        32: asm_shell_lom[AsmShellWfeRms].. -> asm_shell_wfe_rms_scope
        32: asm_shell_lom[AsmShellSegmentD21PistonRSS].. -> asm_shell_dp21rss_scope
        1: plant[M2RigidBodyMotions].. -> asm_shell_lom

        32: asm_rb_lom[AsmRefBodyWfeRms].. -> asm_refbody_wfe_rms_scope
        1: plant[M2ASMReferenceBodyNodes].. -> asm_rb_lom

        32: plant[AverageMountEncoders<-6>].. -> mount_scope

    }

    monitor.await?;

    Ok(())
}

#[derive(UID)]
pub enum LpfSegmentTiptilt {}
#[derive(UID)]
pub enum LpfIntSegmentTiptilt {}
#[derive(UID)]
pub enum SegmentTiptilt7 {}
