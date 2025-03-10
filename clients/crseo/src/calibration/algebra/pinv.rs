use std::ops::Mul;

use faer::{mat::from_column_major_slice, Mat, MatRef};
use serde::{Deserialize, Serialize};

use crate::calibration::mode::Modality;

use super::{Calib, CalibProps, CalibrationMode};

/// Calibration matrix pseudo-inverse
///
/// The 1st generic parameter `T` is the type of the matrix values and the
/// 2nd generic parameter `M` indicates if the matrix correspond to a single segment ([CalibrationMode])
/// or to a full mirror ([MirrorMode](super::MirrorMode),[MixedMirrorMode](crate::calibration::MixedMirrorMode)).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalibPinv<T, M = CalibrationMode>
where
    T: faer::Entity,
    M: Modality,
{
    pub(crate) mat: Mat<T>,
    pub(crate) cond: T,
    pub(crate) mode: M,
}

impl<M: Modality, T: faer::Entity> CalibPinv<T, M> {
    /// Returns the condition number of the calibration matrix
    #[inline]
    pub fn cond(&self) -> T {
        self.cond
    }
    /// Transforms the pseudo-inverse matrix
    pub fn transform<F: Fn(MatRef<'_, T>) -> Mat<T>>(&mut self, fun: F) {
        self.mat = fun(self.mat.as_ref());
    }
    pub fn mat_ref(&self) -> MatRef<'_, T> {
        self.mat.as_ref()
    }
}

impl<M: Modality> Mul<Vec<f64>> for &CalibPinv<f64, M> {
    type Output = Vec<f64>;
    fn mul(self, rhs: Vec<f64>) -> Self::Output {
        let e =
            self.mat.as_ref() * from_column_major_slice::<f64, _, _>(rhs.as_slice(), rhs.len(), 1);
        let iter = e
            .row_iter()
            .flat_map(|r| r.iter().cloned().collect::<Vec<_>>());
        /*         match self.mode {
            CalibrationMode::RBM(tr_xyz) => {
                if n < 6 {
                    let mut out = vec![0.; 6];
                    out.iter_mut()
                        .zip(&tr_xyz)
                        .filter_map(|(out, v)| v.and_then(|_| Some(out)))
                        .zip(iter)
                        .for_each(|(out, e)| *out = e);
                    out
                } else {
                    iter.collect()
                }
            }
            CalibrationMode::Modes {
                n_mode,
                start_idx,
                end_id,
                ..
            } => {
                if n < n_mode {
                    if let Some(end) = end_id {
                        vec![0.; start_idx]
                            .into_iter()
                            .chain(iter)
                            .chain(vec![0.; n_mode - end])
                            .collect()
                    } else {
                        vec![0.; start_idx].into_iter().chain(iter).collect()
                    }
                } else {
                    iter.collect()
                }
            }
            _ => unimplemented!(),
        } */
        self.mode.fill(iter)
    }
}

impl<M: Modality> Mul<MatRef<'_, f64>> for &CalibPinv<f64, M> {
    type Output = Mat<f64>;
    fn mul(self, rhs: MatRef<'_, f64>) -> Self::Output {
        self.mat.as_ref() * rhs
    }
}

impl Mul<MatRef<'_, f64>> for CalibPinv<f64> {
    type Output = Mat<f64>;
    fn mul(self, rhs: MatRef<'_, f64>) -> Self::Output {
        self.mat.as_ref() * rhs
    }
}

// impl Mul<&Calib> for &CalibPinv<f64> {
//     type Output = Mat<f64>;
//     fn mul(self, rhs: &Calib) -> Self::Output {
//         self.mat.as_ref() * rhs.mat_ref()
//     }
// }

impl<M: Modality, C: CalibProps<M>> Mul<&C> for &CalibPinv<f64, M> {
    type Output = Mat<f64>;
    fn mul(self, rhs: &C) -> Self::Output {
        self.mat.as_ref() * rhs.mat_ref()
    }
}

impl<M: Modality, C: CalibProps<M>> Mul<&C> for &mut CalibPinv<f64, M> {
    type Output = Mat<f64>;
    fn mul(self, rhs: &C) -> Self::Output {
        self.mat.as_ref() * rhs.mat_ref()
    }
}

// impl Mul<&Calib> for &mut CalibPinv<f64> {
//     type Output = Mat<f64>;
//     fn mul(self, rhs: &Calib) -> Self::Output {
//         self.mat.as_ref() * rhs.mat_ref()
//     }
// }

impl Mul<Calib> for CalibPinv<f64> {
    type Output = Mat<f64>;
    fn mul(self, rhs: Calib) -> Self::Output {
        self.mat.as_ref() * rhs.mat_ref()
    }
}

impl Mul<Calib> for &CalibPinv<f64> {
    type Output = Mat<f64>;
    fn mul(self, rhs: Calib) -> Self::Output {
        self.mat.as_ref() * rhs.mat_ref()
    }
}
