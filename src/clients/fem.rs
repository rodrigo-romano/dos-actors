/*!
# GMT Finite Element Model client

The module implements the client interface for the [GMT FEM Rust API](https://docs.rs/gmt-fem)

The client is enabled with the `fem` feature.

# Example

Simulation of the FEM and the mount controller together (requires the `fem`, `mount-ctrl` and `apache-arrow` features):
```no_run
# tokio_test::block_on(async {
use dos_actors::clients::mount::{Mount, MountEncoders, MountSetPoint, MountTorques};
use dos_actors::{clients::arrow_client::Arrow, prelude::*};
use fem::{
    dos::{DiscreteModalSolver, ExponentialMatrix},
    fem_io::*,
    FEM,
};
let sim_sampling_frequency = 1000;
let sim_duration = 4_usize;
let n_step = sim_sampling_frequency * sim_duration;

let state_space = {
    let fem = FEM::from_env()?.static_from_env()?;
    let n_io = (fem.n_inputs(), fem.n_outputs());
    DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        .ins::<OSSElDriveTorque>()
        .ins::<OSSAzDriveTorque>()
        .ins::<OSSRotDriveTorque>()
        .outs::<OSSAzEncoderAngle>()
        .outs::<OSSElEncoderAngle>()
        .outs::<OSSRotEncoderAngle>()
        .outs::<OSSM1Lcl>()
        .outs::<MCM2Lcl6D>()
        .use_static_gain_compensation(n_io)
        .build()?
};

let mut source: Initiator<_> = Signals::new(3, n_step).into();
// FEM
let mut fem: Actor<_> = state_space.into();
// MOUNT
let mut mount: Actor<_> = Mount::new().into();

let logging = Arrow::builder(n_step)
    .entry::<f64, OSSM1Lcl>(42)
    .entry::<f64, MCM2Lcl6D>(42)
    .no_save()
    .build()
    .into_arcx();
let mut sink = Terminator::<_>::new(logging.clone());

type D = Vec<f64>;
source
    .add_output()
    .build::<D, MountSetPoint>()
    .into_input(&mut mount);
mount
    .add_output()
    .build::<D, MountTorques>()
    .into_input(&mut fem);
fem.add_output()
    .bootstrap()
    .build::<D, MountEncoders>()
    .into_input(&mut mount);
fem.add_output()
    .unbounded()
    .build::<D, OSSM1Lcl>()
    .into_input(&mut sink);
fem.add_output()
    .unbounded()
    .build::<D, MCM2Lcl6D>()
    .into_input(&mut sink);

Model::new(vec![Box::new(source),
                Box::new(mount),
                Box::new(fem),
                Box::new(sink)])
       .check()?
       .run()
       .wait()
       .await?;
# Ok::<(), anyhow::Error>(())
# });
```
*/

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

// MOUNT CONTROL ----------------------------------------------------------------

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
            .take()
            .map(|data| Arc::new(Data::new(data)))
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

// M1 CONTROL ----------------------------------------------------------------

#[cfg(feature = "crseo")]
impl<S> Get<crate::clients::ceo::M1modes> for DiscreteModalSolver<S>
where
    S: Solver + Default,
{
    fn get(&self) -> Option<Vec<f64>> {
        let mut encoders = <DiscreteModalSolver<S> as Get<fem_io::M1Segment1AxialD>>::get(self)?;
        encoders.extend(
            <DiscreteModalSolver<S> as Get<fem_io::M1Segment2AxialD>>::get(self)?.as_slice(),
        );
        encoders.extend(
            <DiscreteModalSolver<S> as Get<fem_io::M1Segment3AxialD>>::get(self)?.as_slice(),
        );
        encoders.extend(
            <DiscreteModalSolver<S> as Get<fem_io::M1Segment4AxialD>>::get(self)?.as_slice(),
        );
        encoders.extend(
            <DiscreteModalSolver<S> as Get<fem_io::M1Segment5AxialD>>::get(self)?.as_slice(),
        );
        encoders.extend(
            <DiscreteModalSolver<S> as Get<fem_io::M1Segment6AxialD>>::get(self)?.as_slice(),
        );
        encoders.extend(
            <DiscreteModalSolver<S> as Get<fem_io::M1Segment7AxialD>>::get(self)?.as_slice(),
        );
        Some(encoders)
    }
}

#[cfg(feature = "crseo")]
impl<S> Write<Vec<f64>, crate::clients::ceo::M1modes> for DiscreteModalSolver<S>
where
    S: Solver + Default,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, crate::clients::ceo::M1modes>>> {
        <DiscreteModalSolver<S> as Get<crate::clients::ceo::M1modes>>::get(self)
            .take()
            .map(|data| Arc::new(Data::new(data)))
    }
}
