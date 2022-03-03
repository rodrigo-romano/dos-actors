//! GMT FEM client

use crate::{
    io::{Data, Read, Write},
    Update,
};
use fem::{
    dos::{DiscreteModalSolver, Get, Set, Solver},
    fem_io,
};
use std::sync::Arc;

impl<S> Update for DiscreteModalSolver<S>
where
    DiscreteModalSolver<S>: Iterator,
    S: Solver + Default,
{
    fn update(&mut self) {
        log::debug!("update");
        self.next();
    }
}

impl<S, U> Read<Vec<f64>, U> for DiscreteModalSolver<S>
where
    Vec<Option<fem_io::Inputs>>: fem_io::FemIo<U>,
    S: Solver + Default,
    U: 'static,
{
    fn read(&mut self, data: Arc<Data<Vec<f64>, U>>) {
        <DiscreteModalSolver<S> as Set<U>>::set(self, &**data)
    }
}

impl<S, U> Write<Vec<f64>, U> for DiscreteModalSolver<S>
where
    Vec<Option<fem_io::Outputs>>: fem_io::FemIo<U>,
    S: Solver + Default,
    U: 'static,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, U>>> {
        <DiscreteModalSolver<S> as Get<U>>::get(self).map(|data| Arc::new(Data::new(data.to_vec())))
    }
}

#[cfg(feature = "mount-ctrl")]
impl<S> Get<crate::clients::mount::MountEncoders> for DiscreteModalSolver<S>
where
    S: Solver + Default,
{
    fn get(&self) -> Option<Vec<f64>> {
        let mut encoders = <DiscreteModalSolver<S> as Get<fem_io::OSSAzEncoderAngle>>::get(self)?;
        encoders.extend(
            <DiscreteModalSolver<S> as Get<fem_io::OSSElEncoderAngle>>::get(self)?.as_slice(),
        );
        encoders.extend(
            <DiscreteModalSolver<S> as Get<fem_io::OSSRotEncoderAngle>>::get(self)?.as_slice(),
        );
        Some(encoders)
    }
}
#[cfg(feature = "mount-ctrl")]
impl<S> Write<Vec<f64>, crate::clients::mount::MountEncoders> for DiscreteModalSolver<S>
where
    S: Solver + Default,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, crate::clients::mount::MountEncoders>>> {
        <DiscreteModalSolver<S> as Get<crate::clients::mount::MountEncoders>>::get(self)
            .map(|data| Arc::new(Data::new(data.to_vec())))
    }
}
#[cfg(feature = "mount-ctrl")]
impl<S: Solver + Default> Set<crate::clients::mount::MountTorques> for DiscreteModalSolver<S> {
    fn set(&mut self, u: &[f64]) {
        let (azimuth, others) = u.split_at(12);
        <DiscreteModalSolver<S> as Set<fem_io::OSSAzDriveTorque>>::set(self, azimuth);
        let (elevation, gir) = others.split_at(4);
        <DiscreteModalSolver<S> as Set<fem_io::OSSElDriveTorque>>::set(self, elevation);
        <DiscreteModalSolver<S> as Set<fem_io::OSSRotDriveTorque>>::set(self, gir);
    }
}
#[cfg(feature = "mount-ctrl")]
impl<S> Read<Vec<f64>, crate::clients::mount::MountTorques> for DiscreteModalSolver<S>
where
    S: Solver + Default,
{
    fn read(&mut self, data: Arc<Data<Vec<f64>, crate::clients::mount::MountTorques>>) {
        <DiscreteModalSolver<S> as Set<crate::clients::mount::MountTorques>>::set(self, &**data);
    }
}
