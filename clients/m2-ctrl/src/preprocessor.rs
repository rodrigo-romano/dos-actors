use gmt_dos_clients::interface::{Data, Read, Update, Write};
use gmt_dos_clients_io::gmt_m2::asm::segment::AsmCommand;
use nalgebra::{DMatrix, DMatrixView, DVector};
use polars::prelude::*;
use std::{fmt::Display, fs::File, ops::Mul, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum PreprocessorError {
    #[error("nodes file not found")]
    NodesFile(#[from] std::io::Error),
    #[error("failed to read nodes polars dataframe")]
    Nodes(#[from] PolarsError),
}

type Result<T> = std::result::Result<T, PreprocessorError>;

/// ASM voicecoil postion pre-processor
#[derive(Debug, Default)]
pub struct Preprocessor {
    masks: (Mask, Mask, Mask),
    mats: Option<(DMatrix<f64>, DMatrix<f64>)>, // (-k3^{-1}*k1, k3^{-1}*k2)
    data: Option<Arc<Vec<f64>>>,
}
type Mask = Vec<bool>;
impl Preprocessor {
    /// Creates a new pre-processor for an ASM voicecoils position command vector
    pub fn new<'a, P: AsRef<Path>>(
        path: P,
        sid: u8,
        stiffness: DMatrixView<'a, f64>,
    ) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        let df = ParquetReader::new(file).finish()?;
        let label = format!("S{sid}");
        let nodes: Vec<_> = df[label.as_str()]
            .iter()
            .filter_map(|series| {
                if let AnyValue::List(series) = series {
                    series
                        .f64()
                        .ok()
                        .map(|x| x.into_iter().take(2).filter_map(|x| x).collect::<Vec<_>>())
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        let m1 = Self::nodes_by(&nodes, |x| x > 0.28);
        let m2 = Self::nodes_by(&nodes, |x| x > 0.21 && x < 0.28);
        let m3 = Self::nodes_by(&nodes, |x| x < 0.21);
        let mats = Self::processor((&m1, &m2, &m3), stiffness);

        Ok(Self {
            masks: (m1, m2, m3),
            mats,
            ..Default::default()
        })
    }
    /// Returns the indices of the nodes which radius match the predicate
    pub fn nodes_by<F>(nodes: &[f64], pred: F) -> Mask
    where
        F: Fn(f64) -> bool,
    {
        nodes
            .chunks(2)
            .map(|node| node[0].hypot(node[1]))
            .map(|r| pred(r))
            .collect()
    }
    pub fn processor<'a>(
        masks: (&Mask, &Mask, &Mask),
        stiffness: DMatrixView<'a, f64>,
    ) -> Option<(DMatrix<f64>, DMatrix<f64>)> {
        let (m1, m2, m3) = masks;

        let k23: DMatrix<f64> = Kij::new(stiffness, m2, m3).into();
        let k33: DMatrix<f64> = Kij::new(stiffness, m3, m3).into();
        let k3 = k23.transpose() * &k23 + k33.transpose() * &k33;

        let k21: DMatrix<f64> = Kij::new(stiffness, m2, m1).into();
        let k31: DMatrix<f64> = Kij::new(stiffness, m3, m1).into();
        let k1 = k23.transpose() * &k21 + k33.transpose() * &k31;

        let k22: DMatrix<f64> = Kij::new(stiffness, m2, m2).into();
        let k32: DMatrix<f64> = Kij::new(stiffness, m3, m2).into();
        let k2 = k23.transpose() * &k22 + k33.transpose() * &k32;

        k3.try_inverse().map(|ik3| (-&ik3 * k1, ik3 * k2))
    }
}

impl Mul<&[f64]> for &mut Preprocessor {
    type Output = Vec<f64>;

    fn mul(self, rhs: &[f64]) -> Self::Output {
        let (m1, m2, m3) = &self.masks;
        let p1: DVector<f64> = VCMi::new(rhs, m1).into();
        let p2: DVector<f64> = VCMi::new(rhs, m2).into();
        if let Some((a, b)) = &self.mats {
            let p3 = a * p1 + b * p2;
            let mut p = rhs.to_vec();
            p.iter_mut()
                .zip(m3)
                .filter_map(|(p, &m)| if m { Some(p) } else { None })
                .zip(p3.as_slice())
                .for_each(|(p, &p3)| *p = p3);
            p
        } else {
            vec![]
        }
    }
}

impl Update for Preprocessor {}
impl Write<AsmCommand<7>> for Preprocessor {
    fn write(&mut self) -> Option<Data<AsmCommand<7>>> {
        self.data
            .clone()
            .as_ref()
            .map(|x| self * x)
            .map(|x| Data::new(x))
    }
}
impl Read<AsmCommand<7>> for Preprocessor {
    fn read(&mut self, data: Data<AsmCommand<7>>) {
        self.data = Some(data.as_arc());
    }
}
/// Masked voice coil motion vector
#[derive(Debug)]
pub struct VCMi<'a> {
    vcm: &'a [f64],
    mask: &'a Mask,
}
impl<'a> VCMi<'a> {
    pub fn new(vcm: &'a [f64], mask: &'a Mask) -> Self {
        Self { vcm, mask }
    }
    pub fn len(&self) -> usize {
        self.mask
            .iter()
            .filter(|x| **x)
            .enumerate()
            .map(|(i, _)| i)
            .last()
            .unwrap()
            + 1
    }
    pub fn iter(&self) -> impl Iterator<Item = f64> + 'a {
        self.vcm
            .iter()
            .zip(self.mask)
            .filter_map(|(v, &m)| if m { Some(*v) } else { None })
    }
}
impl<'a> From<VCMi<'a>> for DVector<f64> {
    fn from(value: VCMi<'a>) -> Self {
        let n = value.len();
        DVector::from_iterator(n, value.iter())
    }
}
impl<'a> Display for VCMi<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.iter().collect::<Vec<f64>>())
    }
}

/// Subset of the stiffness matrix
#[derive(Debug)]
pub struct Kij<'a> {
    mat: DMatrixView<'a, f64>,
    rows_mask: &'a Mask,
    columns_mask: &'a Mask,
}
impl<'a> Kij<'a> {
    pub fn new(mat: DMatrixView<'a, f64>, rows_mask: &'a Mask, columns_mask: &'a Mask) -> Self {
        Self {
            mat,
            rows_mask,
            columns_mask,
        }
    }
}
impl<'a> From<Kij<'a>> for DMatrix<f64> {
    fn from(value: Kij<'a>) -> Self {
        let columns: Vec<_> = value
            .mat
            .column_iter()
            .zip(value.columns_mask)
            .filter_map(|(c, &m)| if m { Some(c) } else { None })
            .collect();
        let mat = DMatrix::<f64>::from_columns(&columns);
        let rows: Vec<_> = mat
            .row_iter()
            .zip(value.rows_mask)
            .filter_map(|(r, &m)| if m { Some(r) } else { None })
            .collect();
        DMatrix::<f64>::from_rows(&rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Calibration;

    #[test]
    fn nodes() {
        let calibration_file_name =
            Path::new(env!("FEM_REPO")).join("asms_zonal_kl66qr_calibration.bin");
        let asms_calibration = Calibration::try_from(calibration_file_name).unwrap();
        let stiffness = DMatrix::<f64>::from_column_slice(675, 675, asms_calibration.stiffness(7));
        let path = Path::new("/home/ec2-user/projects/dos-actors/grsim/asms");
        let pp =
            Preprocessor::new(path.join("ASMS-nodes.parquet"), 7, stiffness.as_view()).unwrap();
        // dbg!(pp.nodes.len());
        let (m1, m2, m3) = &pp.masks;
        let p = vec![0f64; 675];
        let p1 = VCMi::new(p.as_slice(), m1);
        let n1 = p1.len();
        dbg!(n1);
        let p2 = VCMi::new(p.as_slice(), m2);
        let n2 = p2.len();
        dbg!(n2);
        let p3 = VCMi::new(p.as_slice(), m3);
        let n3 = p3.len();
        dbg!(n3);
        dbg!(n1 + n2 + n3);
    }

    #[test]
    fn processor() {
        let n = 675;
        let mut stiffness = DMatrix::<f64>::zeros(n, n);
        stiffness.fill(1f64);
        let path = Path::new("/home/ec2-user/projects/dos-actors/grsim/asms");
        let pp =
            Preprocessor::new(path.join("ASMS-nodes.parquet"), 7, stiffness.as_view()).unwrap();
        let (m1, m2, m3) = &pp.masks;
        let mut stiffness = DMatrix::<f64>::zeros(n, n);
        let m2_iter = m2
            .iter()
            .enumerate()
            .filter_map(|(i, &m)| if m { Some(i) } else { None });
        let m3_iter = m3
            .iter()
            .enumerate()
            .filter_map(|(i, &m)| if m { Some(i) } else { None });
        for i in m2_iter.clone() {
            stiffness[(i, i)] = 1f64;
        }
        for i in m3_iter.clone() {
            stiffness[(i, i)] = 1f64;
        }
        for (i, j) in m2_iter.zip(m3_iter) {
            stiffness[(i, j)] = 0.5f64;
            stiffness[(j, i)] = 0.5f64;
        }
        /*         stiffness.row_iter().for_each(|r| {
            r.iter()
                .for_each(|r| if *r != 0f64 { print!("*") } else { print!(".") });
            println!("");
        }); */
        let pp =
            Preprocessor::new(path.join("ASMS-nodes.parquet"), 7, stiffness.as_view()).unwrap();
        /*         if let Some((a, b)) = &pp.mats {
            println!("{}", a.sum());
            println!("{}", b);
        } else {
            todo!()
        } */
        let mut p = vec![0f64; n];
        p.iter_mut()
            .zip(m2)
            .filter_map(|(p, &m)| if m { Some(p) } else { None })
            .for_each(|p| *p = 1f64);
        let pf = &pp * &p;
        let p1 = VCMi::new(&pf, &m1);
        let p2 = VCMi::new(&pf, &m2);
        let p3 = VCMi::new(&pf, &m3);
        println!("{p1}");
        println!("{p2}");
        println!("{p3}");
    }
}
