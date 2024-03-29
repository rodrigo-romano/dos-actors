/*!
# M2 ASMS OFF-LOAD

## M2 VOICE COILS TO RBMS

```shell
cargo run --release -p gmt_dos-clients_fem --bin static_gain --features="serde clap" -- -i MC_M2_S1_VC_delta_F -i MC_M2_S2_VC_delta_F -i MC_M2_S3_VC_delta_F -i MC_M2_S4_VC_delta_F -i MC_M2_S5_VC_delta_F -i MC_M2_S6_VC_delta_F -i MC_M2_S7_VC_delta_F -o MC_M2_lcl_6D -f vcf_2_rbm.pkl
cargo run --release -p gmt_dos-clients_fem --bin static_gain --features="serde clap" -- -i MC_M2_S1_VC_delta_F -i MC_M2_S2_VC_delta_F -i MC_M2_S3_VC_delta_F -i MC_M2_S4_VC_delta_F -i MC_M2_S5_VC_delta_F -i MC_M2_S6_VC_delta_F -i MC_M2_S7_VC_delta_F -o MC_M2_S1_VC_delta_D -o MC_M2_S2_VC_delta_D -o MC_M2_S3_VC_delta_D -o MC_M2_S4_VC_delta_D -o MC_M2_S5_VC_delta_D -o MC_M2_S6_VC_delta_D -o MC_M2_S7_VC_delta_D -f vcf.pkl
```

```python
import numpy as np
from scipy.io import savemat

data = np.load("vcf_2_rbm.pkl",allow_pickle=True)
k1 = np.asarray(data[0],order="F").reshape(data[2],data[1]).T
data = np.load("vcf.pkl",allow_pickle=True)
k2 = np.asarray(data[0],order="F").reshape(data[2],data[1]).T
m2_vc_r = {f"m2_s{i+1}_vc_r": np.linalg.lstsq(k2[i*675:(i+1)*675,i*675:(i+1)*675].T,k1[i*6:(i+1)*6,i*675:(i+1)*675].T,rcond=None)[0].T for i in range(7)}
savemat("m2_vc_r.mat",m2_vc_r)
```

## M2 RBMS TO FACESHEET

```shell
cargo run --release -p gmt_dos-clients_fem --bin static_gain --features="serde clap" -- -i MC_M2_SmHex_F -o M2_segment_1_axial_d -o M2_segment_2_axial_d -o M2_segment_3_axial_d -o M2_segment_4_axial_d -o M2_segment_5_axial_d -o M2_segment_6_axial_d -o M2_segment_7_axial_d -f rbm_2_facesheet.pkl
cargo run --release -p gmt_dos-clients_fem --bin static_gain --features="serde clap" -- -i MC_M2_SmHex_F -o MC_M2_RB_6D -f hex_2_rbm.pkl
```

```python
import numpy as np
from scipy.io import savemat

data = np.load("rbm_2_facesheet.pkl",allow_pickle=True)
k1 = np.asarray(data[0],order="F").reshape(data[2],data[1]).T
data = np.load("hex_2_rbm.pkl",allow_pickle=True)
k2 = np.asarray(data[0],order="F").reshape(data[2],data[1]).T
k1p = k1[:,::2] - k1[:,1::2]
k2p = k2[:,::2] - k2[:,1::2]
rbm_2_faceheet = {f"m2_s{i+1}_rbm_2_shell": k1p[i*675:(i+1)*675,i*6:(i+1)*6] @ np.linalg.inv(k2p[i*6:(i+1)*6,i*6:(i+1)*6]) for i in range(7)}
savemat("rbm_2_faceheet.mat",rbm_2_faceheet)
```
*/
use anyhow::{Context, Result};
use gmt_dos_actors::{actorscript, system::Sys};
use gmt_dos_clients::{
    low_pass_filter::LowPassFilter,
    operator::{Left, Operator, Right},
    Gain, Integrator, Signals,
};
use gmt_dos_clients_io::{
    gmt_m2::{
        asm::{M2ASMAsmCommand, M2ASMReferenceBodyNodes, M2ASMVoiceCoilsMotion},
        M2EdgeSensors, M2RigidBodyMotions,
    },
    optics::SegmentPiston,
};
use gmt_dos_clients_lom::LinearOpticalModel;
use gmt_dos_clients_servos::{
    asms_servo::{self, ReferenceBody},
    AsmsServo, EdgeSensors, GmtFem, GmtM2, GmtM2Hex, GmtServoMechanisms,
};
use gmt_fem::FEM;
use interface::{filing::Filing, units::NM, Data, Read, UniqueIdentifier, Update, Write, UID};
use matio_rs::MatFile;
use nalgebra::{self as na, DMatrix, DVector};
use std::{env, mem, path::Path, sync::Arc};

const ACTUATOR_RATE: usize = 80; // 100Hz

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp(None).init();

    let data_repo = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("bin")
        .join("m2");
    env::set_var("DATA_REPO", &data_repo);
    let fem_var = env::var("FEM_REPO").expect("`FEM_REPO` is not set");
    let gmt_dos_clients_scopefem_path = Path::new(&fem_var);

    let sim_sampling_frequency = 8000;
    let m1_freq = 100; // Hz
    assert!(m1_freq == sim_sampling_frequency / ACTUATOR_RATE);
    let _sim_duration = 2_usize; // second
    let n_step = 8000; //sim_sampling_frequency * sim_duration;

    let gmt_servos = Sys::<GmtServoMechanisms<ACTUATOR_RATE, 1>>::from_data_repo_or_else(
        "edge-sensors_servos.bin",
        || {
            GmtServoMechanisms::new(
                sim_sampling_frequency as f64,
                FEM::from_env().expect("failed to load the FEM"),
            )
            .asms_servo(AsmsServo::new().reference_body(asms_servo::ReferenceBody::new()))
            .edge_sensors(EdgeSensors::both())
        },
    )?;

    /*     // VOICE COILS DISPLACEMENT 2 RBMS
    let mat_file = MatFile::load(data_repo.join("m2_vc_r.mat"))?;
    let mut vc_2_rbm = Vec::<DMatrix<f64>>::new();
    for i in 1..=7 {
        vc_2_rbm.push(mat_file.var(format!("m2_s{i}_vc_r"))?);
    }
    // RBMS 2 FACESHEETS
    let mat_file = MatFile::load(data_repo.join("rbm_2_faceheet.mat"))?;
    let mut rbm_2_shell = Vec::<DMatrix<f64>>::new();
    for i in 1..=7 {
        rbm_2_shell.push(mat_file.var(format!("m2_s{i}_rbm_2_shell"))?);
    } */

    /*     let rbm_2_vc = vc_2_rbm[0].clone().pseudo_inverse(0f64).unwrap();
    dbg!(rbm_2_vc.shape());
    let mut asm_cmd = rbm_2_vc.column(0).map(|x| x * 1e-6).as_slice().to_vec();
    asm_cmd.append(&mut vec![0f64; 675 * 6]); */

    let asms_cmd = Signals::from((
        vec![1e-6; 675]
            .into_iter()
            .chain([0f64; 675].repeat(5).into_iter())
            .chain([1e-6; 675].into_iter())
            .collect::<Vec<f64>>(),
        n_step,
    ));

    let lom = LinearOpticalModel::new()?;

    // let vc_2_rbm_1 = Gain::new(vc_2_rbm[0].clone());
    // let vc_2_rbm_7 = Gain::new(vc_2_rbm[6].clone());

    let voice_coil_to_rbm = VoiceCoilToRbm::new()?;

    let rbm_2_shell = Rbm2Shell::new()?;

    let lpf = LowPassFilter::new(675 * 7, 0.1);

    // Lag-compensator
    let lag = 4000f64.recip();
    let rbm_int = Integrator::new(42).gain(lag).forgetting_factor(1. - lag);

    let substract_m2_rbms = Operator::new("-");

    let asms_offloading = AsmsOffLoading::new()?;

    // facesheet off-load to positionners
    actorscript! {
        #[labels(rbm_int="Leaky integral\nController",substract_m2_rbms="Sub")]
        1: asms_cmd[Left<M2ASMAsmCommand>] -> substract_m2_rbms[M2ASMAsmCommand]
            -> lpf[M2ASMAsmCommand] -> {gmt_servos::GmtM2}
        1: {gmt_servos::GmtFem}[M2ASMVoiceCoilsMotion]
            -> voice_coil_to_rbm[M2ASMVoiceCoilsMotionAsRbms]
                -> rbm_int[M2RigidBodyMotions]
                    -> {gmt_servos::GmtM2Hex}

        1: {gmt_servos::GmtFem}[M2EdgeSensors]! -> asms_offloading
        1: {gmt_servos::GmtFem}[M2ASMReferenceBodyNodes]! -> asms_offloading[EdgeSensorsAsRbms]
            -> rbm_2_shell[Right<RbmAsShell>] -> substract_m2_rbms

        1: {gmt_servos::GmtFem}[M2RigidBodyMotions].. -> lom[SegmentPiston]~
    }
    Ok(())
}

#[derive(UID)]
pub enum EdgeSensorsAsRbms {}

pub struct AsmsOffLoading {
    // 36x36
    // rbm_2_mode: na::DMatrix<f64>,
    // 36x6
    r7_2_es: na::DMatrix<f64>,
    // 36x48
    es_2_r: na::DMatrix<f64>,
    // 42
    rbms: Arc<Vec<f64>>,
    // 42
    edge_sensors: Arc<Vec<f64>>,
    // 42
    data: Arc<Vec<f64>>,
}
impl AsmsOffLoading {
    pub fn new() -> Result<Self> {
        //  * M2S7 RIGID-BODY MOTIONS TO EDGE SENSORS
        let r7_2_es: na::DMatrix<f64> = MatFile::load(
            "/home/rconan/Dropbox/projects/dos-actors/clients/fem/mech/data/m12_e_rs/m2_r7_es.mat",
        )
        .context("Failed to read from m2_r7_es.mat")?
        .var("m2_r7_es")?;
        //  * EDGE SENSORS TO M2 RBMS
        let fem_var = env::var("FEM_REPO").expect("`FEM_REPO` is not set");
        let fem_path = Path::new(&fem_var);
        let es_2_r = MatFile::load(fem_path.join("m12_e_rs").join("m12_r_es.mat"))
            .context("Failed to read from m12_r_es.mat")?
            .var("m2_r_es")?;
        Ok(Self {
            r7_2_es,
            es_2_r,
            rbms: Default::default(),
            edge_sensors: Default::default(),
            data: Default::default(),
        })
    }
}
impl Update for AsmsOffLoading {
    fn update(&mut self) {
        let r7 = &self.rbms[36..];
        let es_from_r7 = &self.r7_2_es * na::DVector::from_column_slice(r7);
        // let rbm_2_mode =
        //     &self.rbm_2_mode * na::DVector::from_column_slice(&self.edge_sensors[..36]);
        let data: Vec<_> = self
            .edge_sensors
            .iter()
            .zip(es_from_r7.into_iter())
            .map(|(x, y)| x - y)
            .collect();
        let rbm = &self.es_2_r * na::DVector::from_column_slice(&data);
        let data: Vec<_> = rbm
            .into_iter()
            .map(|x| *x)
            .chain(r7.into_iter().map(|x| *x))
            .collect::<Vec<_>>();
        self.data = Arc::new(data);
    }
}
impl Read<M2ASMReferenceBodyNodes> for AsmsOffLoading {
    fn read(&mut self, data: Data<M2ASMReferenceBodyNodes>) {
        self.rbms = data.into_arc();
    }
}
impl Read<M2EdgeSensors> for AsmsOffLoading {
    fn read(&mut self, data: Data<M2EdgeSensors>) {
        self.edge_sensors = data.into_arc();
    }
}
impl Write<EdgeSensorsAsRbms> for AsmsOffLoading {
    fn write(&mut self) -> Option<Data<EdgeSensorsAsRbms>> {
        Some(self.data.clone().into())
    }
}

#[derive(UID)]
pub enum M2ASMVoiceCoilsMotionAsRbms {}

pub struct VoiceCoilToRbm {
    data: Arc<Vec<Arc<Vec<f64>>>>,
    vc_2_rbm: Vec<DMatrix<f64>>,
    y: Vec<f64>,
}

impl VoiceCoilToRbm {
    pub fn new() -> Result<Self> {
        let data_repo = env::var("DATA_REPO").context("`DATA_REPO` is not set")?;
        let mat_file = MatFile::load(Path::new(&data_repo).join("m2_vc_r.mat"))?;
        let mut vc_2_rbm = Vec::<DMatrix<f64>>::new();
        for i in 1..=7 {
            vc_2_rbm.push(mat_file.var(format!("m2_s{i}_vc_r"))?);
        }
        Ok(Self {
            data: Default::default(),
            vc_2_rbm,
            y: vec![0f64; 42],
        })
    }
}

impl Update for VoiceCoilToRbm {
    fn update(&mut self) {
        let _ = mem::replace(
            &mut self.y,
            self.data
                .iter()
                .zip(&self.vc_2_rbm)
                .map(|(data, vc_2_rbm)| -vc_2_rbm * DVector::from_column_slice(data.as_slice()))
                .flat_map(|x| x.as_slice().to_vec())
                .collect::<Vec<_>>(),
        );
    }
}

impl Read<M2ASMVoiceCoilsMotion> for VoiceCoilToRbm {
    fn read(&mut self, data: Data<M2ASMVoiceCoilsMotion>) {
        self.data = data.into_arc();
    }
}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for VoiceCoilToRbm {
    fn write(&mut self) -> Option<Data<U>> {
        Some(self.y.clone().into())
    }
}

#[derive(UID)]
pub enum RbmAsShell {}

pub struct Rbm2Shell {
    data: Arc<Vec<f64>>,
    rbm_2_shell: Vec<DMatrix<f64>>,
    y: Vec<f64>,
}

impl Rbm2Shell {
    pub fn new() -> Result<Self> {
        let data_repo = env::var("DATA_REPO").context("`DATA_REPO` is not set")?;
        let mat_file = MatFile::load(Path::new(&data_repo).join("rbm_2_faceheet.mat"))?;
        let mut rbm_2_shell = Vec::<DMatrix<f64>>::new();
        for i in 1..=7 {
            rbm_2_shell.push(mat_file.var(format!("m2_s{i}_rbm_2_shell"))?);
        }
        Ok(Self {
            data: Default::default(),
            rbm_2_shell,
            y: vec![0f64; 675 * 7],
        })
    }
}

impl Update for Rbm2Shell {
    fn update(&mut self) {
        let _ = mem::replace(
            &mut self.y,
            self.data
                .chunks(6)
                .zip(&self.rbm_2_shell)
                .map(|(data, rbm_2_shell)| rbm_2_shell * DVector::from_column_slice(data))
                .flat_map(|x| x.as_slice().to_vec())
                .collect::<Vec<_>>(),
        );
    }
}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Read<U> for Rbm2Shell {
    fn read(&mut self, data: Data<U>) {
        self.data = data.into_arc();
    }
}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for Rbm2Shell {
    fn write(&mut self) -> Option<Data<U>> {
        Some(self.y.clone().into())
    }
}
