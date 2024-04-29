/*!
# M2 ASMS Edge Sensors Feed-Forward and Shell Off-load

Model of the GMT servo-mechanisms with M2 edge sensors feed-forward to the the
voice coil actuators and off-load from the voice coil actuators to the reference bodies.

*/
use anyhow::Result;
use edge_sensors::{
    AsmsToHexOffload, EdgeSensorsFeedForward, HexToRbm, M1EdgeSensorsToRbm, M1Lom,
    M2EdgeSensorsToRbm, M2Lom, M2RBLom, RbmToShell, Scopes, VoiceCoilToRbm, N_ACTUATOR,
};
use gmt_dos_actors::{actorscript, system::Sys};
use gmt_dos_clients::{
    low_pass_filter::LowPassFilter,
    operator::{Left, Operator, Right},
    Integrator, Signal, Signals, Timer,
};
use gmt_dos_clients_io::gmt_m1::assembly;
use gmt_dos_clients_io::{
    gmt_fem::outputs::MCM2SmHexD,
    gmt_m1::{M1EdgeSensors, M1RigidBodyMotions},
    gmt_m2::{
        asm::{M2ASMAsmCommand, M2ASMReferenceBodyNodes, M2ASMVoiceCoilsMotion},
        M2EdgeSensors, M2RigidBodyMotions,
    },
    optics::{SegmentPiston, SegmentTipTilt},
};
use gmt_dos_clients_lom::LinearOpticalModel;
use gmt_dos_clients_scope::server::{Monitor, Scope};
use gmt_dos_clients_servos::{
    asms_servo, AsmsServo, EdgeSensors, GmtFem, GmtM1, GmtM2, GmtM2Hex, GmtServoMechanisms,
    WindLoads,
};
#[cfg(feature = "gmt_dos-clients_windloads")]
use gmt_dos_clients_windloads::{
    system::{Mount, SigmoidCfdLoads, M1, M2},
    CfdLoads,
};
use gmt_fem::FEM;
use interface::{filing::Filing, units::Mas};
use interface::{Tick, UID};
use io::{
    M1SegmentPiston, M2ASMVoiceCoilsMotionAsRbms, M2EdgeSensorsAsRbms, M2RBSegmentPiston, M2S1Tz,
    M2S1VcAsTz, M2SegmentPiston, RbmAsShell,
};
use matio_rs::MatFile;
use nalgebra as na;
use std::{env, path::Path};

const ACTUATOR_RATE: usize = 80; // 100Hz

#[derive(UID)]
pub enum RBMCmd {}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().init(); //.format_timestamp(None).init();

    let data_repo = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("data");
    env::set_var("DATA_REPO", &data_repo);

    let sim_sampling_frequency = 8000;
    let m1_freq = 100; // Hz
    assert!(m1_freq == sim_sampling_frequency / ACTUATOR_RATE);
    let sim_duration = 3_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let mut fem = Option::<FEM>::None;

    // Voice coils displacements to rigid body motions
    // let voice_coil_to_rbm =
    //     VoiceCoilToRbm::from_data_repo_or_else("voice_coil_to_rbm.bin", || {
    //         fem.get_or_insert_with(|| FEM::from_env().unwrap())
    //     })?;
    let lag = 2000f64.recip();
    let asms_to_pos = Sys::new(AsmsToHexOffload::new(
        fem.get_or_insert_with(|| FEM::from_env().unwrap()),
        lag,
    )?)
    .build()?;
    // Rigid body motions to facesheet displacements
    // let rbm_2_shell = RbmToShell::new()?;
    let edge_sensors_feedfwd = Sys::new(EdgeSensorsFeedForward::new()?).build()?;

    // EDGE SENSORS
    //  * M1 EDGE SENSORS NODES
    let es_nodes_2_data: na::DMatrix<f64> =
        MatFile::load(data_repo.join("M1_edge_sensor_conversion.mat"))?.var("A1")?;
    //  * EDGE SENSORS TO RIGID-BODY MOTIONS TRANSFORM (M1 & M2)
    let es_2_m1_rbm = {
        let mat = MatFile::load(data_repo.join("m12_r_es.mat"))?;
        let m1_es_recon: na::DMatrix<f64> = mat.var("m1_r_es")?;
        m1_es_recon.insert_rows(36, 6, 0f64) * es_nodes_2_data
    };
    dbg!(es_2_m1_rbm.shape());

    // GMT Servo-Mechanisms
    #[cfg(feature = "gmt_dos-clients_windloads")]
    let (cfd_loads, gmt_servos) = {
        let mut fem = Option::<FEM>::None;
        // The CFD wind loads must be called next afer the FEM as it is modifying the FEM CFDMountWindLoads inputs
        let cfd_loads = Sys::<SigmoidCfdLoads>::from_data_repo_or("preloaded_windloads.bin", {
            CfdLoads::foh(data_repo.to_str().unwrap(), sim_sampling_frequency)
                .duration(10f64)
                .mount(fem.get_or_insert_with(|| FEM::from_env().unwrap()), 0, None)
                .m1_segments()
                .m2_segments()
        })?;

        let gmt_servos = Sys::<GmtServoMechanisms<ACTUATOR_RATE, 1>>::from_data_repo_or_else(
            "preloaded_servos.bin",
            || {
                GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(
                    sim_sampling_frequency as f64,
                    fem.unwrap(),
                )
                .wind_loads(WindLoads::new())
                .asms_servo(AsmsServo::new().reference_body(asms_servo::ReferenceBody::new()))
                .edge_sensors(EdgeSensors::both().m1_with(es_2_m1_rbm))
            },
        )?;

        (cfd_loads, gmt_servos)
    };
    #[cfg(not(feature = "gmt_dos-clients_windloads"))]
    let gmt_servos =
        Sys::<GmtServoMechanisms<ACTUATOR_RATE, 1>>::from_data_repo_or_else("servos.bin", || {
            GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(
                sim_sampling_frequency as f64,
                fem.unwrap_or_else(|| FEM::from_env().unwrap()),
            )
            .wind_loads(WindLoads::new())
            .asms_servo(AsmsServo::new().reference_body(asms_servo::ReferenceBody::new()))
            .edge_sensors(EdgeSensors::both().m1_with(es_2_m1_rbm))
        })?;

    // Linear optical model
    // let lom = LinearOpticalModel::new()?;
    // let lom1 = lom.clone();
    // let lom2 = lom.clone();
    // let lom2rb = lom.clone();

    // Low pass filter of ASMS command (actuator displacement)
    // let lpf = LowPassFilter::new(N_ACTUATOR * 7, 0.05);

    // Lag-compensator? or leaky integral controller
    // let lag = 2000f64.recip();
    // let rbm_int = Integrator::new(42).gain(lag); //.forgetting_factor(1. - lag);

    // Substraction operator
    // let substract_m2_rbms = Operator::new("-");

    // ASMS off-load to positionners
    // let asms_offloading = M2EdgeSensorsToRbm::new()?;

    /*     let asms_cmd = Signals::from((
        vec![1e-6; N_ACTUATOR]
            .into_iter()
            .chain([0f64; N_ACTUATOR].repeat(5).into_iter())
            .chain([1e-6; N_ACTUATOR].into_iter())
            .collect::<Vec<f64>>(),
        n_step,
    )); */
    let asms_cmd = Signals::from((vec![0f64; N_ACTUATOR * 7], n_step));

    // ASMS positioners to RBM
    // let hex_2_rbm = HexToRbm::new()?;

    // SCOPES
    let mut monitor = Monitor::new();
    //  * WFE RMS
    // let scope_sampling_frequency = sim_sampling_frequency / 32;
    /*     let segment_piston_scope = Scope::<SegmentPiston<-9>>::builder(&mut monitor)
        .sampling_frequency(sim_sampling_frequency as f64)
        .build()?;
    let m1_segment_piston_scope = Scope::<M1SegmentPiston>::builder(&mut monitor)
        .sampling_frequency(sim_sampling_frequency as f64)
        .build()?;
    let m2_segment_piston_scope = Scope::<M2SegmentPiston>::builder(&mut monitor)
        .sampling_frequency(sim_sampling_frequency as f64)
        .build()?;
    let m2rb_segment_piston_scope = Scope::<M2RBSegmentPiston>::builder(&mut monitor)
        .sampling_frequency(sim_sampling_frequency as f64)
        .build()?; */
    /*     let segment_tiptilt_scope = Scope::<Mas<SegmentTipTilt>>::builder(&mut monitor)
        .sampling_frequency(sim_sampling_frequency as f64)
        .build()?;
    let ref_body_rbm_scope = Scope::<M2S1Tz>::builder(&mut monitor)
        .sampling_frequency(sim_sampling_frequency as f64)
        .build()?;
    let voicecoil_rbm_scope = Scope::<M2S1VcAsTz>::builder(&mut monitor)
        .sampling_frequency(sim_sampling_frequency as f64)
        .build()?; */
    let scopes = Sys::new(Scopes::new(sim_sampling_frequency as f64, &mut monitor)?).build()?;

    // Select a vector element and convert it to nanometer
    // let m2_s1_tz = USelect::<NM<_>>::new(2);
    // let m2_s1_vc_as_tz = USelect::<NM<_>>::new(2);

    let m1_rbm = Signals::new(6 * 7, n_step).channel(2, Signal::Constant(1e-6));
    let m1_rbm_lpf = LowPassFilter::new(6 * 7, 0.001);

    // Integrated Model
    let metronome: Timer = Timer::new(sim_sampling_frequency);
    actorscript! {
        1: metronome[Tick] -> {gmt_servos::GmtFem}
        1: m1_rbm[RBMCmd] -> m1_rbm_lpf[assembly::M1RigidBodyMotions] -> {gmt_servos::GmtM1}

        1: {gmt_servos::GmtFem}[M1RigidBodyMotions].. -> {scopes::LinearOpticalModel}
        1: {gmt_servos::GmtFem}[M2RigidBodyMotions].. -> {scopes::LinearOpticalModel}
        1: {gmt_servos::GmtFem}[M1RigidBodyMotions].. -> {scopes::M1Lom}
        1: {gmt_servos::GmtFem}[M2RigidBodyMotions].. -> {scopes::M2Lom}
        1: {gmt_servos::GmtFem}[M2ASMReferenceBodyNodes].. -> {scopes::M2RBLom}
        // 1: lom[SegmentPiston<-9>] -> segment_piston_scope

        // 1: {gmt_servos::GmtFem}[M1RigidBodyMotions].. -> lom1
        // 1: lom1[M1SegmentPiston] -> m1_segment_piston_scope

        // 1: {gmt_servos::GmtFem}[M2RigidBodyMotions].. -> lom2
        // 1: lom2[M2SegmentPiston] -> m2_segment_piston_scope

        // 1: {gmt_servos::GmtFem}[M2ASMReferenceBodyNodes]!.. -> lom2rb
        // 1: lom2rb[M2RBSegmentPiston] -> m2rb_segment_piston_scope
    }

    // Integrated Model
    type Operatorf64 = Operator<f64>;
    type LowPassFilterf64 = LowPassFilter<f64>;
    let metronome: Timer = Timer::new(sim_sampling_frequency);
    actorscript! {
        #[labels(asms_cmd = "ASMS Actuators\nCommand")]
            // m2_s1_tz = "M2S1Tz", m2_s1_vc_as_tz = "M2S1Tz")]
        1: metronome[Tick] -> {gmt_servos::GmtFem}

        1: m1_rbm[RBMCmd] -> m1_rbm_lpf[assembly::M1RigidBodyMotions] -> {gmt_servos::GmtM1}

        1: {gmt_servos::GmtFem}[M1EdgeSensors]! -> {edge_sensors_feedfwd::RbmToShell}

        // send the ASMS command (actuator displacement) to the ASMS controller
        1: asms_cmd[Left<M2ASMAsmCommand>] -> {edge_sensors_feedfwd::Operatorf64}
        1: {edge_sensors_feedfwd::LowPassFilterf64}[M2ASMAsmCommand] -> {gmt_servos::GmtM2}
        // read the voice coil displacement from the FEM
        1: {gmt_servos::GmtFem}[M2ASMVoiceCoilsMotion]!
            // transfrom them to rigid body motions (RBMS)
            -> {asms_to_pos}[M2RigidBodyMotions] -> {gmt_servos::GmtM2Hex}
            // -> voice_coil_to_rbm[M2ASMVoiceCoilsMotionAsRbms]
            //     // integrate the RBMS
            //     -> rbm_int[M2RigidBodyMotions]
            //         // send the RBMS to the ASMS positioners
            //         -> {gmt_servos::GmtM2Hex}

        // send the edge sensors data to the ASMS off-loading algorithm
        1: {gmt_servos::GmtFem}[M2EdgeSensors]! -> {edge_sensors_feedfwd::M2EdgeSensorsToRbm}
        // send the reference body RBMS to the ASMS off-loading algorithm
        1: {gmt_servos::GmtFem}[MCM2SmHexD]! -> {edge_sensors_feedfwd::HexToRbm}
            // -> hex_2_rbm[M2ASMReferenceBodyNodes]
            //     -> asms_offloading[M2EdgeSensorsAsRbms]
            //         // transforms the RBMS to facesheet displacements
            //         -> rbm_2_shell[Right<RbmAsShell>] -> substract_m2_rbms

        1: {gmt_servos::GmtFem}[M1RigidBodyMotions].. -> {scopes::LinearOpticalModel}
        1: {gmt_servos::GmtFem}[M2RigidBodyMotions].. -> {scopes::LinearOpticalModel}
        1: {gmt_servos::GmtFem}[M1RigidBodyMotions].. -> {scopes::M1Lom}
        1: {gmt_servos::GmtFem}[M2RigidBodyMotions].. -> {scopes::M2Lom}
        1: {gmt_servos::GmtFem}[M2ASMReferenceBodyNodes].. -> {scopes::M2RBLom}
        // 1: lom[Mas<SegmentTipTilt>] -> segment_tiptilt_scope
    }

    // EDGE SENSORS INTEGRAL CONTROLLERS:
    //  * M1
    // let m1_es_int = Integrator::<M1EdgeSensors>::new(42).gain(1e-3);
    // let m1_add = Operator::<f64>::new("+");
    let m1_es_to_rbm = Sys::new(M1EdgeSensorsToRbm::new()).build()?;

    // Integrated Model
    type M1EdgeSensorsIntegrator = Integrator<M1EdgeSensors>;
    actorscript! {
        #[labels(asms_cmd = "ASMS Actuators\nCommand")]
            // m2_s1_tz = "M2S1Tz", m2_s1_vc_as_tz = "M2S1Tz")]
        // 1: metronome[Tick] -> {gmt_servos::GmtFem}

        1: m1_rbm[RBMCmd] -> m1_rbm_lpf[Right<RBMCmd>]
            -> {m1_es_to_rbm::Operatorf64}[assembly::M1RigidBodyMotions]
                -> {gmt_servos::GmtM1}
        1: {gmt_servos::GmtFem}[M1EdgeSensors]! -> {m1_es_to_rbm::M1EdgeSensorsIntegrator}
        // 1: m1_es_to_rbm[Left<M1EdgeSensors>]! -> m1_add

        1: {gmt_servos::GmtFem}[M1EdgeSensors]! -> {edge_sensors_feedfwd::RbmToShell}

        // 1: {cfd_loads::M1}[CFDM1WindLoads] -> {gmt_servos::GmtFem}
        // 1: {cfd_loads::M2}[CFDM2WindLoads] -> {gmt_servos::GmtFem}
        // 1: {cfd_loads::Mount}[CFDMountWindLoads] -> {gmt_servos::GmtFem}

        // send the ASMS command (actuator displacement) to the ASMS controller
        1: asms_cmd[Left<M2ASMAsmCommand>] -> {edge_sensors_feedfwd::Operatorf64}
        1: {edge_sensors_feedfwd::LowPassFilterf64}[M2ASMAsmCommand] -> {gmt_servos::GmtM2}

        // read the voice coil displacement from the FEM
        1: {gmt_servos::GmtFem}[M2ASMVoiceCoilsMotion]!
            // transfrom them to rigid body motions (RBMS)
            -> {asms_to_pos}[M2RigidBodyMotions] -> {gmt_servos::GmtM2Hex}
            // -> voice_coil_to_rbm[M2ASMVoiceCoilsMotionAsRbms]
            //     // integrate the RBMS
            //     -> rbm_int[M2RigidBodyMotions]
            //         // send the RBMS to the ASMS positioners
            //         -> {gmt_servos::GmtM2Hex}

        // send the edge sensors data to the ASMS off-loading algorithm
        1: {gmt_servos::GmtFem}[M2EdgeSensors]! -> {edge_sensors_feedfwd::M2EdgeSensorsToRbm}
        // send the reference body RBMS to the ASMS off-loading algorithm
        1: {gmt_servos::GmtFem}[MCM2SmHexD]! -> {edge_sensors_feedfwd::HexToRbm}
            // -> hex_2_rbm[M2ASMReferenceBodyNodes]
            //     -> asms_offloading[M2EdgeSensorsAsRbms]
            //         // transforms the RBMS to facesheet displacements
            //         -> rbm_2_shell[Right<RbmAsShell>] -> substract_m2_rbms

        // 1: voice_coil_to_rbm[M2ASMVoiceCoilsMotionAsRbms]
        //     -> m2_s1_vc_as_tz[M2S1VcAsTz] -> voicecoil_rbm_scope
        // 1: {gmt_servos::GmtFem}[M2ASMReferenceBodyNodes]! -> m2_s1_tz[M2S1Tz] -> ref_body_rbm_scope
        // 1: {gmt_servos::GmtFem}[M2RigidBodyMotions].. -> lom

        1: {gmt_servos::GmtFem}[M1RigidBodyMotions].. -> {scopes::LinearOpticalModel}
        1: {gmt_servos::GmtFem}[M2RigidBodyMotions].. -> {scopes::LinearOpticalModel}
        1: {gmt_servos::GmtFem}[M1RigidBodyMotions].. -> {scopes::M1Lom}
        1: {gmt_servos::GmtFem}[M2RigidBodyMotions].. -> {scopes::M2Lom}
        1: {gmt_servos::GmtFem}[M2ASMReferenceBodyNodes].. -> {scopes::M2RBLom}

        // 1: lom[Mas<SegmentTipTilt>] -> segment_tiptilt_scope
    }

    monitor.await?;

    Ok(())
}
