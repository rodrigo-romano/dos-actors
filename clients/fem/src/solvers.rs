mod bilinear;
mod cuda_solver;
mod exponential;
mod exponential_matrix;

pub use bilinear::Bilinear;
pub use cuda_solver::CuStateSpace;
pub use exponential::Exponential;
pub use exponential_matrix::ExponentialMatrix;

pub trait Solver: Send + Sync {
    fn from_second_order(
        tau: f64,
        omega: f64,
        zeta: f64,
        continuous_bb: Vec<f64>,
        continuous_cc: Vec<f64>,
    ) -> Self;
    fn solve(&mut self, u: &[f64]) -> &[f64];
}
