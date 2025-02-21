use fem_cuda_solver::{mode_state_space, state_space};

use crate::{
    solvers::{Exponential, ExponentialMatrix, Solver},
    DiscreteModalSolver,
};

impl From<&Exponential> for mode_state_space {
    fn from(so: &Exponential) -> Self {
        mode_state_space {
            x0: so.x.0,
            x1: so.x.1,
            a0: so.q.0,
            a1: so.q.2,
            a2: so.q.1,
            a3: so.q.3,
            b2: so.m.0,
            b3: so.m.1,
        }
    }
}
impl From<Exponential> for (mode_state_space, (Vec<f64>, Vec<f64>)) {
    fn from(so: Exponential) -> Self {
        ((&so).into(), (so.b, so.c))
    }
}
impl From<&ExponentialMatrix> for mode_state_space {
    fn from(so: &ExponentialMatrix) -> Self {
        mode_state_space {
            x0: so.x.0,
            x1: so.x.1,
            a0: so.phi.0,
            a1: so.phi.2,
            a2: so.phi.1,
            a3: so.phi.3,
            b2: so.gamma.0,
            b3: so.gamma.1,
        }
    }
}
impl From<ExponentialMatrix> for (mode_state_space, (Vec<f64>, Vec<f64>)) {
    fn from(so: ExponentialMatrix) -> Self {
        ((&so).into(), (so.b, so.c))
    }
}

/// State space model using [fem_cuda_solver]
#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct CuStateSpace {
    mss: Vec<mode_state_space>,
    cu_ss: state_space,
    n_mode: usize,
    n_input: usize,
    n_output: usize,
    i2m_rows: Vec<f64>,
    m2o_cols: Vec<f64>,
}
impl CuStateSpace {
    /// Creates a new instance of [CuStateSpace]
    pub fn new<T>(second_orders: Vec<T>) -> Self
    where
        (mode_state_space, (Vec<f64>, Vec<f64>)): From<T>,
    {
        let n_mode = second_orders.len();
        let (mut mss, (i2m_rows, m2o_cols)): (Vec<_>, (Vec<_>, Vec<_>)) =
            second_orders.into_iter().map(|so| so.into()).unzip();
        let n_input = i2m_rows[0].len();
        let n_output = m2o_cols[0].len();
        let mut i2m_rows: Vec<_> = i2m_rows.into_iter().flatten().collect();
        let mut m2o_cols: Vec<_> = m2o_cols.into_iter().flatten().collect();
        let mut cu_ss = state_space::default();
        unsafe {
            cu_ss.build(
                n_mode as i32,
                mss.as_mut_ptr(),
                n_input as i32,
                i2m_rows.as_mut_ptr(),
                n_output as i32,
                m2o_cols.as_mut_ptr(),
            )
        };
        Self {
            mss,
            cu_ss,
            n_mode,
            n_input,
            n_output,
            i2m_rows,
            m2o_cols,
        }
    }
    /// Set the DC gain compensation matrix
    pub fn set_dc_gain_compensator(&mut self, dcg: &[f64]) {
        unsafe {
            self.cu_ss.dc_gain_compensator(dcg.as_ptr() as *mut _);
        }
    }
    /// Steps the state space model
    pub fn step(&mut self, u: &mut [f64], y: &mut [f64]) {
        unsafe {
            self.cu_ss.step(u.as_mut_ptr(), y.as_mut_ptr());
        }
    }
}

impl Drop for CuStateSpace {
    fn drop(&mut self) {
        unsafe { self.cu_ss.free() };
    }
}

impl Solver for CuStateSpace {
    fn n_input(&self) -> usize {
        self.n_input
    }
    fn n_output(&self) -> usize {
        self.n_output
    }
    fn get_b(&self) -> &[f64] {
        self.i2m_rows.as_slice()
    }
    fn get_c(&self) -> &[f64] {
        self.m2o_cols.as_slice()
    }
}

impl<T> DiscreteModalSolver<T>
where
    T: Solver + Default,
    (mode_state_space, (Vec<f64>, Vec<f64>)): From<T>,
{
    /// Replace the CPU based solver with a GPU based solver
    pub fn with_cuda_solver(self) -> DiscreteModalSolver<CuStateSpace> {
        let Self {
            u,
            y,
            y_sizes,
            state_space,
            psi_dcg,
            psi_times_u,
            ins,
            outs,
            facesheet_nodes,
            m1_figure_nodes,
        } = self;
        let mut cu_ss = CuStateSpace::new(state_space);
        if let Some(dcg) = &psi_dcg {
            cu_ss.set_dc_gain_compensator(dcg.as_slice());
        }
        DiscreteModalSolver {
            u,
            y,
            y_sizes,
            state_space: vec![cu_ss],
            psi_dcg,
            psi_times_u,
            ins,
            outs,
            facesheet_nodes,
            m1_figure_nodes,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, time::Instant};

    use gmt_fem::FEM;

    use crate::{
        solvers::{Exponential, ExponentialMatrix},
        DiscreteModalSolver,
    };
    use gmt_dos_clients_io::gmt_fem::{
        inputs::{MCM2Lcl6F, MCM2SmHexF, OSSM1Lcl6F, OSSRotDriveTorque, CFD2021106F},
        outputs::{MCM2Lcl6D, MCM2SmHexD, OSSM1Lcl, OSSRotEncoderAngle, MCM2RB6D},
    };

    #[test]
    pub fn gir() -> Result<(), Box<dyn Error>> {
        let fem = FEM::from_env()?;
        println!("{fem}");
        let mut builder = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem.clone())
            .sampling(1000f64)
            .proportional_damping(2. / 100.)
            // .use_static_gain_compensation()
            .ins::<OSSRotDriveTorque>()
            .outs::<OSSRotEncoderAngle>();
        let max_hsv = builder.max_hankel_singular_values().unwrap();
        let mut model = builder
            // .truncate_hankel_singular_values(max_hsv * 0.01)
            .build()?;
        println!("{model}");
        dbg!(&model.psi_dcg);

        let mut builder = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(1000f64)
            .proportional_damping(2. / 100.)
            .ins::<OSSRotDriveTorque>()
            .outs::<OSSRotEncoderAngle>();
        let max_hsv = builder.max_hankel_singular_values().unwrap();
        let mut cu_model = builder
            // .truncate_hankel_singular_values(max_hsv * 0.01)
            .build()?
            .with_cuda_solver();
        /* let cu_ss = &cu_model.state_space[0];
        let ss = &model.state_space;
        for i in 0..2 {
            dbg!(&ss[i].q);
            dbg!(&ss[i].m);
            dbg!(&cu_ss.mss[i]);
        }
        cu_ss
            .i2m_rows
            .chunks(cu_ss.n_input)
            .enumerate()
            .for_each(|(i, b)| println!("{}\n{:.6?}\n{:.6?}", i, b, ss[i].b));
        cu_ss
            .m2o_cols
            .chunks(cu_ss.n_output)
            .enumerate()
            .for_each(|(i, c)| println!("{}\n{:.6?}\n{:.6?}", i, c, ss[i].c)); */

        model
            .u
            .iter_mut()
            .zip(&mut cu_model.u)
            .for_each(|(u, cu_u)| {
                *u = 10.;
                *cu_u = 10.;
            });
        for i in 0..10 {
            let now = Instant::now();
            model.next();
            let eta = now.elapsed().as_micros();
            let now = Instant::now();
            cu_model.next();
            let cu_eta = now.elapsed().as_micros();
            let rms = (model
                .y
                .iter()
                .zip(&cu_model.y)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                / model.y.len() as f64)
                .sqrt();
            println!(
                "#{} [{}/{}] [{:.3e}]:\n{:+6?}\n{:+6?}",
                i, eta, cu_eta, rms, model.y, cu_model.y
            );
        }
        Ok(())
    }
    #[test]
    #[ignore]
    pub fn servos() -> Result<(), Box<dyn Error>> {
        let fem = FEM::from_env()?;
        println!("{fem}");
        let sids: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7];
        let mut model = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem.clone())
            .sampling(1000f64)
            .proportional_damping(2. / 100.)
            //.max_eigen_frequency(75f64)
            .including_mount()
            .including_m1(Some(sids.clone()))?
            .including_asms(Some(sids.clone()), None, None)?
            .ins::<CFD2021106F>()
            .ins::<OSSM1Lcl6F>()
            .ins::<MCM2Lcl6F>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .ins::<MCM2SmHexF>()
            .outs::<MCM2SmHexD>()
            .outs::<MCM2RB6D>()
            .use_static_gain_compensation()
            .build()?;
        println!("{model}");
        let mut cu_model = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem.clone())
            .sampling(1000f64)
            .proportional_damping(2. / 100.)
            //.max_eigen_frequency(75f64)
            .including_mount()
            .including_m1(Some(sids.clone()))?
            .including_asms(Some(sids.clone()), None, None)?
            .ins::<CFD2021106F>()
            .ins::<OSSM1Lcl6F>()
            .ins::<MCM2Lcl6F>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .ins::<MCM2SmHexF>()
            .outs::<MCM2SmHexD>()
            .outs::<MCM2RB6D>()
            .use_static_gain_compensation()
            .build()?
            .with_cuda_solver();
        model
            .u
            .iter_mut()
            .zip(&mut cu_model.u)
            .for_each(|(u, cu_u)| {
                *u = 1.;
                *cu_u = 1.;
            });
        for i in 0..10 {
            let now = Instant::now();
            model.next();
            let eta = now.elapsed().as_micros();
            let now = Instant::now();
            cu_model.next();
            let cu_eta = now.elapsed().as_micros();
            let rms = (model
                .y
                .iter()
                .zip(&cu_model.y)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                / model.y.len() as f64)
                .sqrt();
            println!(
                "#{} [{}/{}] [{:.3e}]:\n{:+6?}\n{:+6?}",
                i,
                eta,
                cu_eta,
                rms,
                &model.y[..5],
                &cu_model.y[..5]
            );
        }
        Ok(())
    }
}
