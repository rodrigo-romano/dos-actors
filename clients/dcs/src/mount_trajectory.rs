use std::{collections::VecDeque, time::Duration};

use interface::{Read, UniqueIdentifier, Update, Write, UID};
use tai_time::MonotonicTime;

#[derive(Debug, Clone, Default)]
pub struct MountTrajectory {
    pub azimuth: VecDeque<f64>,
    pub elevation: VecDeque<f64>,
    pub gir: VecDeque<f64>,
    pub tai: VecDeque<Duration>,
}

#[derive(UID)]
#[uid(port = 7777)]
pub enum OcsMountTrajectory {}

impl Update for MountTrajectory {}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for MountTrajectory {
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
