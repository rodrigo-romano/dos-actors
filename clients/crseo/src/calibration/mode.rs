mod segment;

pub use segment::CalibrationMode;
mod mirror;
pub use mirror::MirrorMode;

use super::algebra::Modality;

impl Modality for CalibrationMode {
    fn n_cols(&self) -> usize {
        match self {
            CalibrationMode::RBM(tr_xyz) => tr_xyz.iter().filter_map(|&x| x).count(),
            &CalibrationMode::Modes {
                n_mode,
                start_idx,
                end_id,
                ..
            } => end_id.unwrap_or(n_mode) - start_idx,
            // _ => unimplemented!(),
        }
    }
    fn fill(&self, iter: impl Iterator<Item = f64>) -> Vec<f64> {
        match self {
            CalibrationMode::RBM(tr_xyz) => {
                let mut out = vec![0.; 6];
                out.iter_mut()
                    .zip(tr_xyz)
                    .filter_map(|(out, v)| v.and_then(|_| Some(out)))
                    .zip(iter)
                    .for_each(|(out, e)| *out = e);
                out
            }
            &CalibrationMode::Modes {
                n_mode,
                start_idx,
                end_id,
                ..
            } => {
                let end = end_id.unwrap_or(n_mode);
                vec![0.; start_idx]
                    .into_iter()
                    .chain(iter.take(end - start_idx))
                    .chain(vec![0.; n_mode - end])
                    .collect()
            } /*             CalibrationMode::Mirror(segments) => segments
              .iter()
              .filter_map(|segment| {
                  segment.as_ref().map(|s| match s.deref() {
                      CalibrationMode::RBM(tr_xyz) => {
                          let mut out = vec![0.; 6];
                          out.iter_mut()
                              .zip(tr_xyz)
                              .filter_map(|(out, v)| v.and_then(|_| Some(out)))
                              .zip(iter.by_ref())
                              .for_each(|(out, e)| *out = e);
                          out
                      }
                      &CalibrationMode::Modes {
                          n_mode,
                          start_idx,
                          end_id,
                          ..
                      } => {
                          let end = end_id.unwrap_or(n_mode);
                          vec![0.; start_idx]
                              .into_iter()
                              .chain(iter.by_ref().take(end - start_idx))
                              .chain(vec![0.; n_mode - end])
                              .collect()
                      }
                      _ => unimplemented!(),
                  })
              })
              .flatten()
              .collect(), */
        }
    }
}

impl Modality for MirrorMode {
    fn n_cols(&self) -> usize {
        self.iter()
            .filter_map(|segment| segment.as_ref().map(|s| s.n_cols()))
            .sum()
    }
    fn fill(&self, mut iter: impl Iterator<Item = f64>) -> Vec<f64> {
        self.iter()
            .filter_map(|segment| {
                segment.as_ref().map(|s| match s {
                    CalibrationMode::RBM(tr_xyz) => {
                        let mut out = vec![0.; 6];
                        out.iter_mut()
                            .zip(tr_xyz)
                            .filter_map(|(out, v)| v.and_then(|_| Some(out)))
                            .zip(iter.by_ref())
                            .for_each(|(out, e)| *out = e);
                        out
                    }
                    &CalibrationMode::Modes {
                        n_mode,
                        start_idx,
                        end_id,
                        ..
                    } => {
                        let end = end_id.unwrap_or(n_mode);
                        vec![0.; start_idx]
                            .into_iter()
                            .chain(iter.by_ref().take(end - start_idx))
                            .chain(vec![0.; n_mode - end])
                            .collect()
                    } // _ => unimplemented!(),
                })
            })
            .flatten()
            .collect()
    }
}