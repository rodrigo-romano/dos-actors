use std::{collections::VecDeque, sync::Arc, time::Duration};

use interface::{Read, UniqueIdentifier, Units, Update, Write, UID};
use tai_time::MonotonicTime;

use crate::DcsIO;

/// DCS mount trajectory data
///
/// Data structure where the OCS mount trajectory is collating
#[derive(Debug, Clone, Default)]
pub struct MountTrajectory {
    pub azimuth: VecDeque<f64>,
    pub elevation: VecDeque<f64>,
    pub gir: VecDeque<f64>,
    pub tai: VecDeque<Duration>,
}

impl Units for MountTrajectory {}

pub trait MountTrajectoryIO {}

#[derive(UID)]
#[uid(port = 7777)]
pub enum OcsMountTrajectory {}

impl MountTrajectoryIO for OcsMountTrajectory {}
impl DcsIO for OcsMountTrajectory {}

impl Update for MountTrajectory {}

impl<U: DcsIO + MountTrajectoryIO + UniqueIdentifier<DataType = Vec<f64>>> Write<U>
    for MountTrajectory
{
    fn write(&mut self) -> Option<interface::Data<U>> {
        vec![
            self.azimuth.pop_front(),
            self.elevation.pop_front(),
            self.gir.pop_front(),
        ]
        .into_iter()
        .collect::<Option<Vec<f64>>>()
        .map(|x| x.into())
    }
}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Read<U> for MountTrajectory {
    fn read(&mut self, data: interface::Data<U>) {
        self.azimuth.push_back(data[0]);
        self.elevation.push_back(data[1]);
        self.gir.push_back(data[2]);
        let now = MonotonicTime::now();
        self.tai.push_back(Duration::from_nanos(
            now.as_secs() as u64 * 1_000_000_000 + now.subsec_nanos() as u64,
        ));
    }
}

/// Differential mount trajectory data
///
/// The trajectory is relative to the zero position
/// given by the 1st elements of the mount trajectory
#[derive(Debug, Clone, Default)]
pub struct RelativeMountTrajectory {
    trajectory: Arc<Vec<f64>>,
    zero: Option<Box<RelativeMountTrajectory>>,
}

impl Units for RelativeMountTrajectory {}

#[derive(UID)]
#[uid(port = 7778)]
pub enum RelativeMountAxes {}

impl Update for RelativeMountTrajectory {}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Read<U> for RelativeMountTrajectory {
    fn read(&mut self, data: interface::Data<U>) {
        self.trajectory = data.into_arc();
    }
}

impl MountTrajectoryIO for RelativeMountAxes {}

impl<U: MountTrajectoryIO + UniqueIdentifier<DataType = Vec<f64>>> Write<U>
    for RelativeMountTrajectory
{
    fn write(&mut self) -> Option<interface::Data<U>> {
        Some(
            self.zero
                .get_or_insert(Box::new(self.clone()))
                .trajectory
                .iter()
                .zip(self.trajectory.iter())
                .map(|(z, t)| t - z)
                .collect::<Vec<f64>>()
                .into(),
        )
    }
}
