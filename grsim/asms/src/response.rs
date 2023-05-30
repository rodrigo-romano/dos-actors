use std::ops::Deref;

use crate::if64;
use nalgebra::DMatrix;
use serde::{Deserialize, Serialize};

/* #[derive(Debug, Serialize, Deserialize)]
pub struct MIMO {
    io: (usize, usize),
    re: f64,
    im: f64,
} */

#[derive(Debug, Serialize, Deserialize)]
pub struct MIMO {
    nu: f64,
    frequency_response: DMatrix<if64>,
}

impl From<(f64, DMatrix<if64>)> for MIMO {
    fn from((nu, frequency_response): (f64, DMatrix<if64>)) -> Self {
        MIMO {
            nu,
            frequency_response,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sys(pub(crate) Vec<MIMO>);
impl Deref for Sys {
    type Target = Vec<MIMO>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<(Vec<f64>, Vec<DMatrix<if64>>)> for Sys {
    fn from((nu, frequency_response): (Vec<f64>, Vec<DMatrix<if64>>)) -> Self {
        Sys(nu
            .into_iter()
            .zip(frequency_response.into_iter())
            .map(|x| MIMO::from(x))
            .collect())
    }
}
impl Sys {
    /*     pub fn get(&self, io: (usize, usize)) -> Option<(Vec<f64>, Vec<if64>)> {
        let data: (Vec<f64>, Vec<if64>) = self
            .iter()
            .filter_map(|tf| {
                let mimo = tf.mimo.iter().find(|&mimo| mimo.io == io);
                let z = mimo.map(|mimo| Complex::new(mimo.re, mimo.im));
                let q = Some(tf.nu).zip(z);
                q
            })
            .unzip();
        if data == (vec![], vec![]) {
            None
        } else {
            Some(data)
        }
    } */
    pub fn frequencies(&self) -> Vec<f64> {
        self.iter().map(|tf| tf.nu).collect()
    }
    /*     pub fn get_map<F>(&self, io: (usize, usize), fun: F) -> Option<(Vec<f64>, Vec<f64>)>
    where
        F: Fn(if64) -> f64,
    {
        let data: (Vec<f64>, Vec<f64>) = self
            .iter()
            .filter_map(|tf| {
                let mimo = tf.mimo.iter().find(|&mimo| mimo.io == io);
                let z = mimo.map(|mimo| fun(Complex::new(mimo.re, mimo.im)));
                let q = Some(tf.nu).zip(z);
                q
            })
            .unzip();
        if data == (vec![], vec![]) {
            None
        } else {
            Some(data)
        }
    } */
}
