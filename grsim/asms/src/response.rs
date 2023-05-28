use std::ops::Deref;

use crate::if64;
use nalgebra::DMatrix;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MIMO {
    io: (usize, usize),
    re: f64,
    im: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferFunction {
    nu: f64,
    mimo: Vec<MIMO>,
}

impl From<(f64, DMatrix<if64>)> for TransferFunction {
    fn from((nu, fr): (f64, DMatrix<if64>)) -> Self {
        let (_, m) = fr.shape();
        let mimo: Vec<_> = fr
            .into_iter()
            .enumerate()
            .map(|(k, fr)| {
                let i = k / m;
                let j = k % m;
                MIMO {
                    io: (i, j),
                    re: fr.re,
                    im: fr.im,
                }
            })
            .collect();
        TransferFunction { nu, mimo }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sys(pub(crate) Vec<TransferFunction>);
impl Deref for Sys {
    type Target = Vec<TransferFunction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
use num_complex::Complex;
impl Sys {
    pub fn get(&self, io: (usize, usize)) -> Option<(Vec<f64>, Vec<if64>)> {
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
    }
    pub fn get_map<F>(&self, io: (usize, usize), fun: F) -> Option<(Vec<f64>, Vec<f64>)>
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
    }
}
