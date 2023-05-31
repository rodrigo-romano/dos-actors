use crate::{
    if64, structural::StructuralBuilder, BesselFilter, BuilderTrait, FirstOrderLowPass,
    FrequencyResponse, PICompensator, Structural, StructuralError,
};

#[derive(Debug, thiserror::Error)]
pub enum ASMError {
    #[error(transparent)]
    Structural(#[from] StructuralError),
}
type Result<T> = std::result::Result<T, ASMError>;

const N: usize = 675;

#[derive(Debug)]
#[allow(dead_code)]
/// ASM parameters
struct Parameters {
    kfd: f64,
    kp: f64,
    ki: f64,
    kd: f64,
    km: f64,
    kb: f64,
}
impl Default for Parameters {
    fn default() -> Self {
        Self {
            kfd: 9.1,
            kp: 7e4,
            ki: 5e5,
            kd: 24.5,
            km: 1.12e-2,
            kb: 33.6,
        }
    }
}
/// ASM control model
#[derive(Debug)]
pub struct ASM {
    // 1st-order low-pass filter
    h_pd: FirstOrderLowPass,
    // 4th-order Bessel filter
    f_pre: BesselFilter,
    // proportional–integral compensator
    c_pi: PICompensator,
    // ASM FEM
    structural: Structural,
    // ASM stiffness
    ks: DMatrix<if64>,
    // Parameters
    params: Parameters,
    // Matrix transformation from modes to nodes
    modes_to_nodes: Option<DMatrix<if64>>,
}
impl ASM {
    /// Creates a new ASM control model for segment #`sid`
    pub fn new(sid: u8) -> Result<Self> {
        let inputs = vec![
            format!("MC_M2_S{sid}_VC_delta_F"),
            format!("MC_M2_S{sid}_fluid_damping_F"),
        ];
        let outputs = vec![
            format!("M2_segment_{sid}_axial_d"),
            format!("MC_M2_S{sid}_VC_delta_D"),
        ];
        let structural = Structural::builder(inputs, outputs)
            .filename("asm-structural")
            .build()?;
        let ks = structural
            .static_gain((0, 0), (N, N))
            .expect("failed to get FEM static gain")
            .try_inverse()
            .expect("failed to inverse the static gain for the ASM transfer function")
            .map(|x| Complex::new(x, 0f64));
        Ok(Self {
            h_pd: FirstOrderLowPass::new(),
            f_pre: BesselFilter::new(),
            c_pi: PICompensator::new(),
            structural,
            ks,
            params: Default::default(),
            modes_to_nodes: None,
        })
    }
    /// ASM builder
    pub fn builder(sid: u8) -> Builder {
        let inputs = vec![
            format!("MC_M2_S{sid}_VC_delta_F"),
            format!("MC_M2_S{sid}_fluid_damping_F"),
        ];
        let outputs = vec![
            format!("M2_segment_{sid}_axial_d"),
            format!("MC_M2_S{sid}_VC_delta_D"),
        ];
        let structural = Structural::builder(inputs, outputs).filename("asm-structural");
        Builder { structural }
    }
    /// Modes to nodes transformation matrix
    pub fn modes(mut self, modes_to_nodes: DMatrix<f64>) -> Self {
        self.modes_to_nodes = Some(modes_to_nodes.map(|x| Complex::new(x, 0f64)));
        self
    }
}

#[derive(Debug)]
pub struct Builder {
    structural: StructuralBuilder,
}
impl BuilderTrait for Builder {
    fn damping(mut self, z: f64) -> Self {
        self.structural = self.structural.damping(z);
        self
    }

    fn filename<S: Into<String>>(mut self, file_name: S) -> Self {
        self.structural = self.structural.filename(file_name);
        self
    }

    fn enable_static_gain_mismatch_compensation(mut self, maybe_delay: Option<f64>) -> Self {
        self.structural = self
            .structural
            .enable_static_gain_mismatch_compensation(maybe_delay);
        self
    }
}
impl Builder {
    /// Creates a new ASM control model for segment #`sid`
    pub fn build(self) -> Result<ASM> {
        let structural = self.structural.build()?;
        let ks = structural
            .static_gain((0, 0), (N, N))
            .expect("failed to get FEM static gain")
            .try_inverse()
            .expect("failed to inverse the static gain for the ASM transfer function")
            .map(|x| Complex::new(x, 0f64));
        Ok(ASM {
            h_pd: FirstOrderLowPass::new(),
            f_pre: BesselFilter::new(),
            c_pi: PICompensator::new(),
            structural,
            ks,
            params: Default::default(),
            modes_to_nodes: None,
        })
    }
}

use nalgebra::DMatrix;
use num_complex::Complex;
impl FrequencyResponse for ASM {
    type Output = DMatrix<if64>;

    // **GMT-DOC-XXXXX**: *ASM segment modal tranfer function*, Eq.(14)
    fn j_omega(&self, jw: if64) -> Self::Output {
        let Parameters {
            kfd,
            kp: _,
            ki: _,
            kd,
            km,
            kb,
        } = self.params;
        let g = self.structural.j_omega(jw);

        let hpd = self.h_pd.j_omega(jw);
        let c_pi = self.c_pi.j_omega(jw);
        let c_pi_d = c_pi + kd * hpd;
        let kfd_hpd = kfd * hpd;

        let g11 = g.view((0, 0), (N, N));
        let g12 = g.view((0, N), (N, N));
        let g21 = g.view((N, 0), (N, N));
        let g22 = g.view((N, N), (N, N));

        let eye = DMatrix::<if64>::identity(N, N);
        let a = &eye;
        let b = g11 * c_pi_d + g12 * kfd_hpd;
        let d = &eye + g21 * c_pi_d + g22 * kfd_hpd;

        let mut q = DMatrix::<if64>::zeros(2 * N, 2 * N);
        q.view_mut((0, 0), (N, N)).copy_from(&a);
        q.view_mut((0, N), (N, N)).copy_from(&b);
        q.view_mut((N, N), (N, N)).copy_from(&d);

        let iq = q
            .try_inverse()
            .expect("failed to inverse a matrix for the ASM transfer function");
        let iqg = iq * g.view((0, 0), (2 * N, N));

        let f_pre = self.f_pre.j_omega(jw);
        let h_f1d = jw * f_pre;
        let h_f2d = jw * h_f1d;
        let c_ff_plus = &self.ks * f_pre + &eye * (kb * h_f1d + km * h_f2d + c_pi * f_pre);
        let t = iqg * c_ff_plus;
        if let Some(m2n) = &self.modes_to_nodes {
            m2n.transpose() * t.view((0, 0), (N, N)) * m2n
        } else {
            t.view((0, 0), (N, N)).into()
        }
    }
}
