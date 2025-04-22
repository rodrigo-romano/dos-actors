use gmt_dos_clients_crseo::{
    calibration::Reconstructor, centroiding::CentroidsProcessing, crseo::FromBuilder,
    sensors::Camera, DeviceInitialize, OpticalModelBuilder,
};
use gmt_dos_clients_io::{
    gmt_m2::{fsm::M2FSMFsmCommand, M2RigidBodyMotions},
    optics::{Dev, Frame, SensorData},
};

use crate::kernels::{KernelError, KernelSpecs};

use super::{Sh24, Sh24TT};

type Result<T> = std::result::Result<T, KernelError>;

impl<const I: usize> KernelSpecs for Sh24<I> {
    type Sensor = Camera<I>;

    type Processor = CentroidsProcessing;

    type Estimator = Reconstructor;

    type Integrator = gmt_dos_clients::integrator::Integrator<M2FSMFsmCommand>;

    type Input = Frame<Dev>;

    type Data = SensorData;

    type Output = M2FSMFsmCommand;

    fn processor(
        model: &OpticalModelBuilder<<Self::Sensor as FromBuilder>::ComponentBuilder>,
    ) -> Result<Self::Processor> {
        let mut centroids = CentroidsProcessing::try_from(model)?;
        model.initialize(&mut centroids);
        Ok(centroids)
    }
}
impl<const I: usize> KernelSpecs for Sh24TT<I> {
    type Sensor = Camera<I>;

    type Processor = CentroidsProcessing;

    type Estimator = Reconstructor;

    type Integrator = gmt_dos_clients::integrator::Integrator<M2RigidBodyMotions>;

    type Input = Frame<Dev>;

    type Data = SensorData;

    type Output = M2RigidBodyMotions;

    fn processor(
        model: &OpticalModelBuilder<<Self::Sensor as FromBuilder>::ComponentBuilder>,
    ) -> Result<Self::Processor> {
        let mut centroids = CentroidsProcessing::try_from(model)?;
        model.initialize(&mut centroids);
        Ok(centroids)
    }
}
