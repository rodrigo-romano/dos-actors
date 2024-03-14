//! M2 ASM

use geotrans::{Quaternion, Vector};

pub mod assembly;
pub mod cold_plate;
pub mod face_sheet;
pub mod reference_body;
pub mod segment;
#[doc(hidden)]
pub use super::prelude;

// Removes rigid body motions from ASM facesheet nodes
fn rbm_removal(rbm: &[f64], nodes: &mut [f64], figure: &[f64]) -> Vec<f64> {
    let tz = rbm[2];
    let q = Quaternion::unit(rbm[5], Vector::k())
        * Quaternion::unit(rbm[4], Vector::j())
        * Quaternion::unit(rbm[3], Vector::i());
    nodes
        .chunks_mut(3)
        .zip(figure)
        .map(|(u, dz)| {
            u[2] = dz - tz;
            let p: Quaternion = From::<&[f64]>::from(u);
            let pp = q.complex_conjugate() * p * &q;
            let v: Vec<f64> = pp.vector_as_slice().to_vec();
            v[2]
        })
        .collect()
}
