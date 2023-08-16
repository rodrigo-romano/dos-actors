use super::{Data, Read, UniqueIdentifier, Update, Write};
use nalgebra as na;
use num_traits::{One, Zero};
use std::fmt::Debug;
use std::ops::{AddAssign, Mul, MulAssign};

#[doc(hidden)]
pub enum GainKind<T> {
    Matrix(na::DMatrix<T>),
    Scalar(Vec<T>),
}
impl<T> Mul<&na::DVector<T>> for &GainKind<T>
where
    T: Zero + Clone + PartialEq + Debug + One + AddAssign + Mul + MulAssign + 'static,
{
    type Output = na::DVector<T>;
    fn mul(self, rhs: &na::DVector<T>) -> Self::Output {
        match self {
            GainKind::Matrix(mat) => mat * rhs,
            GainKind::Scalar(val) => {
                let y = rhs.into_iter().zip(val).map(|(u, v)| v.clone() * u.clone());
                na::DVector::from_iterator(val.len(), y)
            }
        }
    }
}
impl<T> From<na::DMatrix<T>> for GainKind<T> {
    fn from(value: na::DMatrix<T>) -> Self {
        Self::Matrix(value)
    }
}
impl<T> From<Vec<T>> for GainKind<T> {
    fn from(value: Vec<T>) -> Self {
        Self::Scalar(value)
    }
}
impl<T> GainKind<T> {
    pub fn ncols(&self) -> usize {
        match self {
            GainKind::Matrix(mat) => mat.ncols(),
            GainKind::Scalar(val) => val.len(),
        }
    }
    pub fn nrows(&self) -> usize {
        match self {
            GainKind::Matrix(mat) => mat.nrows(),
            GainKind::Scalar(val) => val.len(),
        }
    }
}

/// Gain client
///
/// Applies the gain to the input signal either
/// as a matrix multiplication or
/// as an element wise vector multiplication
pub struct Gain<T> {
    u: na::DVector<T>,
    y: na::DVector<T>,
    gain: GainKind<T>,
}
impl<T> Gain<T>
where
    T: Zero + Clone + PartialEq + Debug + 'static,
{
    /// Creates a new [Gain] clients
    ///
    /// The gain is either a matrix of dimensions `Ny`x`Nu`
    /// or a vector of size `Ny`=`Nu`
    pub fn new<G: Into<GainKind<T>>>(gain: G) -> Self {
        let gain: GainKind<T> = gain.into();
        Self {
            u: na::DVector::zeros(gain.ncols()),
            y: na::DVector::zeros(gain.nrows()),
            gain,
        }
    }
}
impl<T> Update for Gain<T>
where
    T: Zero + Clone + PartialEq + Debug + One + AddAssign + Mul + MulAssign + 'static,
{
    fn update(&mut self) {
        self.y = &self.gain * &self.u;
    }
}
impl<T, U> Read<U> for Gain<T>
where
    T: Zero + Clone + PartialEq + Debug + 'static,
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn read(&mut self, data: Data<U>) {
        self.u = na::DVector::from_row_slice(&data);
    }
}
impl<T, U> Write<U> for Gain<T>
where
    T: Zero + Clone + PartialEq + Debug + 'static,
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn write(&mut self) -> Option<Data<U>> {
        Some(Data::new(self.y.as_slice().to_vec()))
    }
}
