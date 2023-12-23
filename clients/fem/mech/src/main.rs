use gmt_dos_actors::{actorscript, prelude::*};
use gmt_dos_clients::{
    operator::{Left, Operator, Right},
    print::Print,
    Gain, Integrator, Signal, Signals,
};
use gmt_dos_clients_fem::{
    fem_io::actors_outputs::OSSM1Lcl, DiscreteModalSolver, ExponentialMatrix, Model, Switch,
};
use gmt_dos_clients_io::{
    gmt_fem::outputs::OSSM1EdgeSensors,
    gmt_m1::{assembly, M1RigidBodyMotions},
    gmt_m2::asm::{
        segment::VoiceCoilsMotion, M2ASMAsmCommand, M2ASMFluidDampingForces, M2ASMVoiceCoilsForces,
        M2ASMVoiceCoilsMotion,
    },
    mount::{MountEncoders, MountSetPoint, MountTorques},
    optics::{M2modes, Wavefront},
};
use gmt_dos_clients_lom::RigidBodyMotionsToLinearOpticalModel;
use gmt_dos_clients_m1_ctrl::{assembly::M1, Calibration};
use gmt_dos_clients_m2_ctrl::ASMS;
use gmt_dos_clients_mount::Mount;
use gmt_fem::FEM;
use interface::{Data, Read, UniqueIdentifier, Update, Write, UID};
use matio_rs::MatFile;
use nalgebra as na;
use nanorand::{Rng, WyRand};
use std::time::Instant;
use std::{env, path::Path, sync::Arc};

const ACTUATOR_RATE: usize = 80;

#[derive(Debug, Default)]
pub struct Multiplex {
    data: Arc<Vec<f64>>,
    slices: Vec<usize>,
}
impl Multiplex {
    fn new(slices: Vec<usize>) -> Self {
        Self {
            slices,
            ..Default::default()
        }
    }
}
#[derive(UID)]
pub enum RBMCmd {}
#[derive(UID)]
pub enum ActuatorCmd {}

impl Update for Multiplex {}
impl<U: UniqueIdentifier<DataType = Vec<f64>>> Read<U> for Multiplex {
    fn read(&mut self, data: Data<U>) {
        self.data = data.into_arc();
    }
}
impl<U: UniqueIdentifier<DataType = Vec<Arc<Vec<f64>>>>> Write<U> for Multiplex {
    fn write(&mut self) -> Option<Data<U>> {
        let mut mx_data = vec![];
        let data = self.data.as_slice();
        let mut a = 0_usize;
        for s in &self.slices {
            let b = a + *s;
            mx_data.push(Arc::new(data[a..b].to_vec()));
            a = b;
        }
        Some(mx_data.into())
    }
}

// export FEM_REPO=/home/rconan/mnt/20230131_1605_zen_30_M1_202110_ASM_202208_Mount_202111/

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder().format_timestamp(None).init();

    env::set_var(
        "DATA_REPO",
        Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("data"),
    );

    let sim_sampling_frequency = 8000;
    let m1_freq = 100; // Hz
    assert!(m1_freq == sim_sampling_frequency / ACTUATOR_RATE);
    let sim_duration = 3_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let mut fem = FEM::from_env()?;
    // println!("{fem}");
    let m1_calibration = Calibration::new(&mut fem);

    let fem_var = env::var("FEM_REPO").expect("`FEM_REPO` is not set");
    let fem_path = Path::new(&fem_var);

    let path = fem_path.join("KLmodesGS36p.mat");
    let now = Instant::now();
    let mut vc_f2d = vec![];
    let mut kl_modes: Vec<na::DMatrix<f64>> = vec![];
    let mut asms_nact = vec![];
    for i in 1..=7 {
        fem.switch_inputs(Switch::Off, None)
            .switch_outputs(Switch::Off, None);

        vc_f2d.push(
            fem.switch_inputs_by_name(vec![format!("MC_M2_S{i}_VC_delta_F")], Switch::On)
                .and_then(|fem| {
                    fem.switch_outputs_by_name(vec![format!("MC_M2_S{i}_VC_delta_D")], Switch::On)
                })
                .map(|fem| {
                    fem.reduced_static_gain()
                        .unwrap_or_else(|| fem.static_gain())
                })?,
        );
        // println!("{:?}", vc_f2d.shape());
        let mat: na::DMatrix<f64> = MatFile::load(&path)?.var(format!("KL_{i}"))?;
        let (nact, n_mode) = mat.shape();
        asms_nact.push(nact);
        kl_modes.push(mat);
    }
    fem.switch_inputs(Switch::On, None)
        .switch_outputs(Switch::On, None);
    println!("stiffness from FEM in {}ms", now.elapsed().as_millis());

    let mat = MatFile::load(fem_path.join("M1_edge_sensor_conversion.mat"))?;
    let es_nodes_2_data: na::DMatrix<f64> = mat.var("A1")?;
    let mat = MatFile::load(
        fem_path.join("m1_es_reconstructor_07.mat"),
        // .join("M1M2ESRecs.mat"),
    )?;
    let m1_es_recon: na::DMatrix<f64> = mat.var("m1_r_es")?;
    // let m1_es_recon: na::DMatrix<f64> = mat.var("Rm1es")?;
    let es_2_rbm = m1_es_recon.insert_rows(36, 6, 0f64) * es_nodes_2_data;

    let mat = MatFile::load(&fem_path.join("rbm_2_asm_kl.mat"))?;
    let rbm_2_mode: na::DMatrix<f64> = mat.var("r2kl")?;

    let mat = MatFile::load(&fem_path.join("rbm_2_asm_vcf.mat"))?;
    let rbm_2_voice_coil_forces: na::DMatrix<f64> = mat.var("r2vcf")?;

    let kl_modes_t: Vec<_> = kl_modes.iter().map(|mat| mat.transpose()).collect();
    let vcd_2_kl: Vec<_> = kl_modes_t
        .iter()
        .map(|mat| mat.view((0, 0), (6, 675)))
        .collect();

    let sids = vec![1, 2, 3, 4, 5, 6, 7];
    let fem_dss = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        // .truncate_hankel_singular_values(1e-7)
        // .hankel_frequency_lower_bound(50.)
        .including_mount()
        .including_m1(Some(sids.clone()))?
        .including_asms(Some(sids.clone()), None, None)?
        .outs::<OSSM1Lcl>()
        .outs_with::<OSSM1EdgeSensors>(es_2_rbm.as_view())
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
    let rbm = Signals::new(6 * 7, n_step * 10).channel(
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

    let calibration = &m1_calibration;

    let actuators = Signals::new(6 * 335 + 306, 10 * n_step);
    let actuators_mx = Multiplex::new(vec![335, 335, 335, 335, 335, 335, 306]);

    let rbm_mx = Multiplex::new(vec![6; 7]);

    let mut m1 = SubSystem::new(M1::<ACTUATOR_RATE>::new(calibration)?)
        .name("M1 Control")
        .build()?
        .flowchart();

    let mut m1_clone = m1.clone();

    // MOUNT CONTROL
    let mount_setpoint = Signals::new(3, n_step);
    let mount = Mount::new();

    let lom = RigidBodyMotionsToLinearOpticalModel::new()?;

    actorscript! {
        #[model(name = warmup, state = completed)]
        1: mount_setpoint[MountSetPoint] -> &mount[MountTorques] -> &plant("GMT FEM")[MountEncoders]! -> &mount

        1: &rbm[Right<RBMCmd>] -> &rbm_mx[assembly::M1RigidBodyMotions]
            -> {m1}[assembly::M1HardpointsForces]
                -> &plant("GMT FEM")[assembly::M1HardpointsMotion]! -> {m1}
        1: &actuators[ActuatorCmd]
            -> &actuators_mx[assembly::M1ActuatorCommandForces]
                -> {m1}[assembly::M1ActuatorAppliedForces] -> &plant("GMT FEM")

        4000: &plant("GMT FEM")[M1RigidBodyMotions] -> &lom[Wavefront]${262144}

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

    let es_int = Integrator::new(42).gain(0.2);
    let add = Operator::new("+");
    dbg!(rbm_2_voice_coil_forces.shape());
    // let gain = Gain::new(rbm_2_mode.insert_columns(36, 6, 0f64));
    let gain = Gain::new(rbm_2_voice_coil_forces.insert_columns(36, 6, 0f64));

    let ks: Vec<_> = vc_f2d.iter().map(|x| Some(x.as_slice().to_vec())).collect();
    let mut asms = SubSystem::new(ASMS::new(asms_nact, ks))
        .name("ASMS")
        .build()?
        .flowchart();

    let asms_mx = Multiplex::new(vec![675; 7]);

    let m1s1_vcd_2_kl = Gain::new(kl_modes_t[0].clone());

    // let print = Print::default();

    actorscript! {
    #[model(name = model, state = completed, resume = True)]
        1: mount_setpoint[MountSetPoint] -> *mount[MountTorques] -> *plant("GMT FEM")[MountEncoders]! -> *mount

        1: add("Add")[RBMCmd] -> *rbm_mx[assembly::M1RigidBodyMotions]
        -> {m1_clone}[assembly::M1HardpointsForces]
            -> *plant("GMT FEM")[assembly::M1HardpointsMotion]! -> {m1_clone}
        1: *actuators[ActuatorCmd]
            -> *actuators_mx[assembly::M1ActuatorCommandForces]
                -> {m1_clone}[assembly::M1ActuatorAppliedForces] -> *plant("GMT FEM")

        1: *rbm[Right<RBMCmd>] -> add("Add")
        1000: *plant("GMT FEM")[OSSM1EdgeSensors]! -> es_int[Left<OSSM1EdgeSensors>]! -> add("Add")

        1: *plant("GMT FEM")[OSSM1EdgeSensors]!
            -> gain[M2modes]
            -> asms_mx[M2ASMAsmCommand]
                 -> {asms}[M2ASMVoiceCoilsForces]-> *plant("GMT FEM")
         1: {asms}[M2ASMFluidDampingForces] -> *plant("GMT FEM")[M2ASMVoiceCoilsMotion]! -> {asms}

         1: *plant("GMT FEM")[VoiceCoilsMotion<1>]! -> m1s1_vcd_2_kl[M2modes]~
        // 1: es_int[Left<OSSM1EdgeSensors>]! -> print
        // 1: *plant("GMT FEM")[M1RigidBodyMotions]~

        // 4000: *plant("GMT FEM")[M1RigidBodyMotions] -> *lom[Wavefront]${262144}

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
