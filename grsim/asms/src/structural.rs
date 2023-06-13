/*!
FINITE ELEMENT MODEL

Model: `20230131_1605_zen_30_M1_202110_ASM_202208_Mount_202111`

FEM_INPUTS:

  000: OSS_ElDrive_Torque <=> OSSElDriveTorque
  001: OSS_AzDrive_Torque <=> OSSAzDriveTorque
  002: OSS_RotDrive_Torque <=> OSSRotDriveTorque
  003: OSS_payloads_6F <=> OSSPayloads6F
  004: OSS_GIR_6F <=> OSSGIR6F
  005: OSS_Harpoint_delta_F <=> OSSHarpointDeltaF
  006: M1_actuators_segment_1 <=> M1ActuatorsSegment1
  007: M1_actuators_segment_2 <=> M1ActuatorsSegment2
  008: M1_actuators_segment_3 <=> M1ActuatorsSegment3
  009: M1_actuators_segment_4 <=> M1ActuatorsSegment4
  010: M1_actuators_segment_5 <=> M1ActuatorsSegment5
  011: M1_actuators_segment_6 <=> M1ActuatorsSegment6
  012: M1_actuators_segment_7 <=> M1ActuatorsSegment7
  013: OSS_M1_lcl_6F <=> OSSM1Lcl6F
  014: OSS_TrussTEIF_6F <=> OSSTrussTEIF6F
  015: MC_M2_SmHex_F <=> MCM2SmHexF
  016: MC_M2_CP_6F <=> MCM2CP6F
  017: MC_M2_RB_6F <=> MCM2RB6F
  018: MC_M2_lcl_6F <=> MCM2Lcl6F
  019: MC_M2_S1_VC_delta_F <=> MCM2S1VCDeltaF
  020: MC_M2_S1_fluid_damping_F <=> MCM2S1FluidDampingF
  021: MC_M2_S2_VC_delta_F <=> MCM2S2VCDeltaF
  022: MC_M2_S2_fluid_damping_F <=> MCM2S2FluidDampingF
  023: MC_M2_S3_VC_delta_F <=> MCM2S3VCDeltaF
  024: MC_M2_S3_fluid_damping_F <=> MCM2S3FluidDampingF
  025: MC_M2_S4_VC_delta_F <=> MCM2S4VCDeltaF
  026: MC_M2_S4_fluid_damping_F <=> MCM2S4FluidDampingF
  027: MC_M2_S5_VC_delta_F <=> MCM2S5VCDeltaF
  028: MC_M2_S5_fluid_damping_F <=> MCM2S5FluidDampingF
  029: MC_M2_S6_VC_delta_F <=> MCM2S6VCDeltaF
  030: MC_M2_S6_fluid_damping_F <=> MCM2S6FluidDampingF
  031: MC_M2_S7_VC_delta_F <=> MCM2S7VCDeltaF
  032: MC_M2_S7_fluid_damping_F <=> MCM2S7FluidDampingF
  033: MC_M2_PMA_1F <=> MCM2PMA1F
  034: CFD_202110_6F <=> CFD2021106F

FEM_OUTPUTS:
  000: OSS_ElEncoder_Angle <=> OSSElEncoderAngle
  001: OSS_AzEncoder_Angle <=> OSSAzEncoderAngle
  002: OSS_RotEncoder_Angle <=> OSSRotEncoderAngle
  003: OSS_payloads_6D <=> OSSPayloads6D
  004: OSS_GIR_6d <=> OSSGIR6d
  005: OSS_Hardpoint_D <=> OSSHardpointD
  006: OSS_M1_lcl <=> OSSM1Lcl
  007: M1_segment_1_axial_d <=> M1Segment1AxialD
  008: M1_segment_2_axial_d <=> M1Segment2AxialD
  009: M1_segment_3_axial_d <=> M1Segment3AxialD
  010: M1_segment_4_axial_d <=> M1Segment4AxialD
  011: M1_segment_5_axial_d <=> M1Segment5AxialD
  012: M1_segment_6_axial_d <=> M1Segment6AxialD
  013: M1_segment_7_axial_d <=> M1Segment7AxialD
  014: OSS_M1_edge_sensors <=> OSSM1EdgeSensors
  015: OSS_TrussIF_6D <=> OSSTrussIF6D
  016: MC_M2_SmHex_D <=> MCM2SmHexD
  017: MC_M2_CP_6D <=> MCM2CP6D
  018: MC_M2_RB_6D <=> MCM2RB6D
  019: MC_M2_lcl_6D <=> MCM2Lcl6D
  020: MC_M2_S1_VC_delta_D <=> MCM2S1VCDeltaD
  021: MC_M2_S2_VC_delta_D <=> MCM2S2VCDeltaD
  022: MC_M2_S3_VC_delta_D <=> MCM2S3VCDeltaD
  023: MC_M2_S4_VC_delta_D <=> MCM2S4VCDeltaD
  024: MC_M2_S5_VC_delta_D <=> MCM2S5VCDeltaD
  025: MC_M2_S6_VC_delta_D <=> MCM2S6VCDeltaD
  026: MC_M2_S7_VC_delta_D <=> MCM2S7VCDeltaD
  027: MC_M2_PMA_1D <=> MCM2PMA1D
  028: M2_segment_1_axial_d <=> M2Segment1AxialD
  029: M2_segment_2_axial_d <=> M2Segment2AxialD
  030: M2_segment_3_axial_d <=> M2Segment3AxialD
  031: M2_segment_4_axial_d <=> M2Segment4AxialD
  032: M2_segment_5_axial_d <=> M2Segment5AxialD
  033: M2_segment_6_axial_d <=> M2Segment6AxialD
  034: M2_segment_7_axial_d <=> M2Segment7AxialD
  035: M2_edge_sensors <=> M2EdgeSensors
  036: M2_fiducials_3D <=> M2Fiducials3D
  037: CFD_202110_6D <=> CFD2021106D
*/

use std::{
    env,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use gmt_dos_clients_fem::{Model, Switch};
use gmt_fem::FEM;
use nalgebra::{DMatrix, DMatrixView};
use num_complex::Complex;
use serde::{Deserialize, Serialize};

use crate::{if64, BuilderTrait, FrequencyResponse};

#[derive(Debug, thiserror::Error)]
pub enum StructuralError {
    #[error(transparent)]
    FEM(#[from] gmt_fem::FemError),
    #[error(transparent)]
    Bincode(#[from] bincode::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("inputs and outputs do not match model in {0}")]
    IOMismatch(String),
}
type Result<T> = std::result::Result<T, StructuralError>;

#[derive(Debug, Deserialize, Serialize)]
pub struct StaticGainCompensation {
    pub(crate) delay: Option<f64>,
    pub(crate) delta_gain: DMatrix<if64>,
}
impl Default for StaticGainCompensation {
    fn default() -> Self {
        Self {
            delay: Default::default(),
            delta_gain: DMatrix::<if64>::zeros(1, 1),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Structural {
    // inputs labels
    inputs: Vec<String>,
    // outputs labels
    outputs: Vec<String>,
    // modal forces matrix
    b: DMatrix<if64>,
    // modal displacements matrix
    c: DMatrix<if64>,
    // static solution gain matrix
    g_ssol: Option<DMatrix<f64>>,
    // static gain mismatch compensation scheme
    static_gain_mismatch: Option<StaticGainCompensation>,
    // eigen frequencies
    w: Vec<f64>,
    // damping coefficient
    z: f64,
}
#[derive(Debug)]
pub struct StructuralBuilder {
    inputs: Vec<String>,
    outputs: Vec<String>,
    z: f64,
    file_name: String,
    static_gain_mismatch: Option<StaticGainCompensation>,
}
impl BuilderTrait for StructuralBuilder {
    /// Sets the FEM modal damping coefficient
    fn damping(mut self, z: f64) -> Self {
        self.z = z;
        self
    }
    /// Sets the filename where [Structural] is seralize to
    fn filename<S: Into<String>>(mut self, file_name: S) -> Self {
        self.file_name = file_name.into();
        self
    }
    /// Enables the compensation of the static gain mismatch
    ///
    /// An optional delay [s] may be added
    fn enable_static_gain_mismatch_compensation(mut self, maybe_delay: Option<f64>) -> Self {
        self.static_gain_mismatch = Some(Default::default());
        if let Some(value) = maybe_delay {
            self.static_gain_mismatch
                .as_mut()
                .and_then(|sgm| sgm.delay.replace(value));
        }
        self
    }
}
impl StructuralBuilder {
    fn new(inputs: Vec<String>, outputs: Vec<String>) -> Self {
        Self {
            inputs,
            outputs,
            z: 2. / 100.,
            file_name: "structural".into(),
            static_gain_mismatch: None,
        }
    }
    /// Builds the [Structural] model
    pub fn build(mut self) -> Result<Structural> {
        let repo = env::var("DATA_REPO").unwrap_or_else(|_| ".".to_string());
        let path = Path::new(&repo).join(self.file_name).with_extension("bin");
        if let Ok(file) = File::open(&path) {
            println!("loading structural from {:?}", path);
            let buffer = BufReader::new(file);
            let this: Structural = bincode::deserialize_from(buffer)?;
            if !(this.inputs == self.inputs && this.outputs == self.outputs) {
                return Err(StructuralError::IOMismatch(
                    path.to_str().unwrap().to_string(),
                ));
            }
            Ok(this)
        } else {
            println!("building structural from FEM");
            let mut fem = FEM::from_env()?;
            fem.switch_inputs(Switch::Off, None)
                .switch_inputs_by_name(self.inputs.clone(), Switch::On)?
                .switch_outputs(Switch::Off, None)
                .switch_outputs_by_name(self.outputs.clone(), Switch::On)?;
            let b =
                DMatrix::<f64>::from_row_slice(fem.n_modes(), fem.n_inputs(), &fem.inputs2modes())
                    .map(|x| Complex::new(x, 0f64));
            let c = DMatrix::<f64>::from_row_slice(
                fem.n_outputs(),
                fem.n_modes(),
                &fem.modes2outputs(),
            )
            .map(|x| Complex::new(x, 0f64));
            let g_ssol = fem.reduced_static_gain();
            let w = fem.eigen_frequencies_to_radians();

            self.static_gain_mismatch.as_mut().map(|sgm| {
                let g_dsol = fem.static_gain();
                let delta_g = g_ssol.as_ref().expect("failed to get FEM static gain") - g_dsol;
                sgm.delta_gain = delta_g.map(|x| Complex::new(x, 0f64));
            });

            let this = Structural {
                inputs: self.inputs,
                outputs: self.outputs,
                b,
                c,
                g_ssol,
                static_gain_mismatch: self.static_gain_mismatch,
                w,
                z: self.z,
            };
            let file = File::create(&path)?;
            let mut buffer = BufWriter::new(file);
            bincode::serialize_into(&mut buffer, &this)?;
            println!("structural save to {:?}", path);
            Ok(this)
        }
    }
}
impl Structural {
    /// Creates a [Structural] builder
    pub fn builder(inputs: Vec<String>, outputs: Vec<String>) -> StructuralBuilder {
        StructuralBuilder::new(inputs, outputs)
    }
    /// Returns a [view](https://docs.rs/nalgebra/latest/nalgebra/base/struct.Matrix.html#method.view) of the static gain
    pub fn static_gain(&self, ij: (usize, usize), nm: (usize, usize)) -> Option<DMatrixView<f64>> {
        self.g_ssol.as_ref().map(|g| g.view(ij, nm))
    }
}
impl FrequencyResponse for Structural {
    type Output = DMatrix<Complex<f64>>;

    /// *Dynamics and Control of Structures, W.K. Gawronsky*, p.17-18, Eqs.(2.21)-(2.22)
    fn j_omega(&self, jw: if64) -> Self::Output {
        let zeros = DMatrix::<Complex<f64>>::zeros(self.c.nrows(), self.b.ncols());
        let fr = self
            .c
            .column_iter()
            .zip(self.b.row_iter())
            .zip(&self.w)
            .fold(zeros, |a, ((c, b), wi)| {
                let mut cb = c * b;
                let ode = wi * wi + jw * jw + 2f64 * self.z * wi * jw;
                cb /= ode;
                a + cb
            });
        match &self.static_gain_mismatch {
            Some(StaticGainCompensation {
                delay: None,
                delta_gain,
            }) => fr + delta_gain,
            Some(StaticGainCompensation {
                delay: Some(t_s),
                delta_gain,
            }) => fr + (delta_gain * (-jw * t_s).exp()),
            None => fr,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Frequencies, Sys};

    use super::*;

    #[test]
    fn mount() {
        let structural = Structural::builder(
            vec!["OSS_ElDrive_Torque".to_string()],
            vec!["OSS_ElEncoder_Angle".to_string()],
        )
        .build()
        .unwrap();

        let (_, tf) = structural.frequency_response(1f64);
        println!("{}", tf[0]);
    }

    #[test]
    fn mount_el_tf() {
        let structural = Structural::builder(
            vec!["OSS_ElDrive_Torque".to_string()],
            vec!["OSS_ElEncoder_Angle".to_string()],
        )
        .build()
        .unwrap();

        let (nu, tf) = structural.frequency_response(Frequencies::logspace(0.1, 100., 1000));
        println!("{:?}", nu);
        println!("{}", tf[0]);

        let mut file = File::create("mount_el_tf.pkl").unwrap();
        serde_pickle::to_writer(&mut file, &(nu, tf), Default::default()).unwrap();
    }

    #[test]
    fn mount_el_tf_dc() {
        let structural = Structural::builder(
            vec!["OSS_ElDrive_Torque".to_string()],
            vec!["OSS_ElEncoder_Angle".to_string()],
        )
        .enable_static_gain_mismatch_compensation(Some(1. / 8e3))
        .build()
        .unwrap();

        let (nu, tf) = structural.frequency_response(Frequencies::logspace(0.1, 4e3, 1000));
        //println!("{:?}", nu);
        println!("{}", tf[0]);

        let mut file = File::create("mount_el_tf_dc_full-sampling_delay.pkl").unwrap();
        serde_pickle::to_writer(&mut file, &(nu, tf), Default::default()).unwrap();
    }

    #[test]
    fn mount_el_tf_linspace() {
        let structural = Structural::builder(
            vec!["OSS_ElDrive_Torque".to_string()],
            vec!["OSS_ElEncoder_Angle".to_string()],
        )
        .build()
        .unwrap();

        let (nu, tf) = structural.frequency_response(Frequencies::LinSpace {
            lower: 1f64,
            upper: 10f64,
            n: 2,
        });
        println!("{:?}", nu);
        println!("{}", tf[0]);

        let sys = Sys::from((nu, tf));
        dbg!(sys);
    }
}
