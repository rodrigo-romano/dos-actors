use super::{Data, Read, UniqueIdentifier, Update, Write};
use nalgebra as na;

/// Gain
pub struct Gain {
    u: na::DVector<f64>,
    y: na::DVector<f64>,
    mat: na::DMatrix<f64>,
}
impl Gain {
    pub fn new(mat: na::DMatrix<f64>) -> Self {
        Self {
            u: na::DVector::zeros(mat.ncols()),
            y: na::DVector::zeros(mat.nrows()),
            mat,
        }
    }
}
impl Update for Gain {
    fn update(&mut self) {
        self.y = &self.mat * &self.u;
    }
}
impl<U: UniqueIdentifier<DataType = Vec<f64>>> Read<U> for Gain {
    fn read(&mut self, data: Data<U>) {
        self.u = na::DVector::from_row_slice(&data);
    }
}
impl<U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for Gain {
    fn write(&mut self) -> Option<Data<U>> {
        Some(Data::new(self.y.as_slice().to_vec()))
    }
}
