use crseo::{wavefrontsensor::PyramidBuilder, Builder, Pyramid, WavefrontSensorBuilder};

use crate::{optical_model::OpticalModelError, OpticalModel, OpticalModelBuilder};

impl OpticalModelBuilder<PyramidBuilder> {
    pub fn build(self) -> Result<OpticalModel<Pyramid>, OpticalModelError> {
        let src = self.sensor.as_ref().unwrap().guide_stars(Some(self.src));
        Ok(OpticalModel {
            gmt: self.gmt.build()?,
            src: src.build()?,
            atm: self.atm_builder.map(|atm| atm.build()).transpose()?,
            sensor: self.sensor.map(|sensor| sensor.build()).transpose()?,
            tau: self.sampling_frequency.map_or_else(|| 0f64, |x| x.recip()),
            phase_offset: None,
        })
    }
}
