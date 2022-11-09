use domeseeing::DomeSeeingOpd;
use dos_actors::{
    io::{Data, Read, Write},
    prelude::*,
    Update,
};
use std::{fs::File, sync::Arc};

/// Karhunen-Loeve modal basis
pub struct KarhunenLoeve {
    // Karhunen-Loeve modes
    basis: Vec<f64>,
    // Karhunen-Loeve coefficients (for correction)
    coefficients: Arc<Data<KarhunenLoeveCoefficients>>,
    // Karhunen-Loeve residual coefficients (from the residual opd `opd_res` after correction)
    residual_coefficients: Arc<Data<KarhunenLoeveResidualCoefficients>>,
    mask: Vec<bool>,
    merged_mask: Vec<bool>,
    n_merged_mask: usize,
    // Residual OPD
    opd_res: Arc<Data<ResidualOpd>>,
}
impl KarhunenLoeve {
    /// Creates a new Karhunen-Loeve object with `n_mode`<=1000
    ///
    /// The mask of the OPD in the exit pupil `projection mask` may be passed
    pub fn new(n_mode: usize, projection_mask: Option<Vec<bool>>) -> Self {
        assert!(n_mode <= 1000);
        // loading the basis creates with KlBasis for CPPAOapi
        let basis: Vec<f64> = serde_pickle::from_reader(
            File::open("domeseeing-kl.pkl").expect("cannot open domeseeing-kl.pk"),
            Default::default(),
        )
        .expect("failed to load Karhunen-Loeve modes");

        // truncated the basis to the give # ofmodes
        let n_px = 104;
        let n_px2 = n_px * n_px;
        let basis: Vec<_> = basis.into_iter().take(n_px2 * n_mode).collect();
        dbg!(basis.len());

        let mask = projection_mask.unwrap_or(vec![true; n_px2]);

        // Fusioning the KL modes and OPD valid domain
        let merged_mask: Vec<_> = basis
            .iter()
            .zip(&mask)
            .map(|(k, &m)| !k.is_nan() && m)
            .collect();
        let n_mask: usize = mask.iter().filter_map(|&m| m.then_some(1)).sum();
        let n_merged_mask: usize = merged_mask.iter().filter_map(|&m| m.then_some(1)).sum();
        println!("opd mask/kl+opd mask: {}/{}", n_mask, n_merged_mask);

        // Truncating the KL modes to the valid domain
        let kl_on_mask: Vec<_> = basis
            .chunks(n_px2)
            .flat_map(|kl| {
                kl.iter()
                    .zip(&merged_mask)
                    .filter_map(|(k, &m)| m.then_some(*k))
            })
            .collect();
        assert_eq!(kl_on_mask.iter().find(|x| x.is_nan()), None);
        assert_eq!(kl_on_mask.len(), n_merged_mask * n_mode);

        // Gram-Schmidt ortho-normalization of the KL modes on the valid domain
        let kl_on_mask_orthonorm = zernike::gram_schmidt(kl_on_mask.as_slice(), n_mode);
        println!("KL {}", kl_on_mask_orthonorm.len());

        Self {
            basis: kl_on_mask_orthonorm,
            coefficients: Arc::new(Data::new(vec![0f64; n_mode])),
            residual_coefficients: Arc::new(Data::new(vec![0f64; n_mode])),
            mask,
            merged_mask,
            n_merged_mask,
            opd_res: Arc::new(Data::new(Vec::new())),
        }
    }
}
impl Update for KarhunenLoeve {}
impl Read<DomeSeeingOpd> for KarhunenLoeve {
    /// Processing of the dome seeing OPD
    fn read(&mut self, data: Arc<Data<DomeSeeingOpd>>) {
        let mut opd_iter = data.iter();
        let opd: Vec<_> = self
            .mask
            .iter()
            .zip(&self.merged_mask)
            .filter_map(|(m, mm)| m.then(|| opd_iter.next().map(|o| (mm, o))).flatten())
            .filter_map(|(m, o)| m.then_some(*o))
            .collect();

        // Substracting the KL modes out-of the OPD to get the residual OPD
        let c: &[f64] = &self.coefficients;
        let opd_res = self.basis.chunks(self.n_merged_mask).zip(c).fold(
            opd.iter().map(|&x| x).collect::<Vec<f64>>(),
            |mut a, (k, &c)| {
                a.iter_mut().zip(k).for_each(|(a, k)| *a += k * c);
                a
            },
        );
        // Projecting the residual OPD onto the KL modes
        let residual_coefficients = self
            .basis
            .chunks(self.n_merged_mask)
            .map(|kl| kl.iter().zip(&opd_res).map(|(a, &b)| a * b).sum::<f64>())
            .collect();
        self.opd_res = Arc::new(Data::new(opd_res));
        self.residual_coefficients = Arc::new(Data::new(residual_coefficients));
    }
}
#[derive(UID)]
pub enum ResidualOpd {}
impl Write<ResidualOpd> for KarhunenLoeve {
    /// Exporting the residual OPD
    fn write(&mut self) -> Option<Arc<Data<ResidualOpd>>> {
        Some(self.opd_res.clone())
    }
}
#[derive(UID)]
pub enum KarhunenLoeveCoefficients {}
impl Read<KarhunenLoeveCoefficients> for KarhunenLoeve {
    /// Importing the KL mode coefficients (for correction)
    fn read(&mut self, data: Arc<Data<KarhunenLoeveCoefficients>>) {
        self.coefficients = data.clone();
    }
}
#[derive(UID)]
pub enum KarhunenLoeveResidualCoefficients {}
impl Write<KarhunenLoeveResidualCoefficients> for KarhunenLoeve {
    /// Exporting the residual KL mode coefficients of the residual OPD after correction
    fn write(&mut self) -> Option<Arc<Data<KarhunenLoeveResidualCoefficients>>> {
        Some(self.residual_coefficients.clone())
    }
}
