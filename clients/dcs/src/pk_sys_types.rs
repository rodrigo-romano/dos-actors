#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct PvatTrajectoryPoint {
    pub position: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub tai: f64,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct ImMountDemands {
    pub azimuth_trajectory: Vec<PvatTrajectoryPoint>,
    pub elevation_trajectory: Vec<PvatTrajectoryPoint>,
    pub gir_trajectory: Vec<PvatTrajectoryPoint>,
    azimuth_motion_mode: String,
    elevation_motion_mode: String,
    gir_motion_mode: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct TrackingStatus {
    tracking_valid: bool,
    interpolation_error: bool,
    error_active: bool,
    traj_running: bool,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct PvatTrajectoryFeedback {
    points: Vec<PvatTrajectoryPoint>,
    tracking_status: TrackingStatus,
    motion_mode: String,
    time_to_target: f64,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct ImMountFeedback {
    pub azimuth_feedback: PvatTrajectoryFeedback,
    pub elevation_feedback: PvatTrajectoryFeedback,
    pub gir_feedback: PvatTrajectoryFeedback,
}

impl From<f64> for PvatTrajectoryPoint {
    fn from(value: f64) -> Self {
        Self {
            position: value,
            velocity: 0.0,
            acceleration: 0.0,
            tai: 0.0,
        }
    }
}

impl PvatTrajectoryPoint {
    pub fn new(position: f64, velocity: f64, acceleration: f64, tai: f64) -> Self {
        Self {
            position,
            velocity,
            acceleration,
            tai,
        }
    }
}

impl TrackingStatus {
    pub fn tracking_valid() -> Self {
        Self {
            tracking_valid: true,
            interpolation_error: false,
            error_active: false,
            traj_running: false,
        }
    }
}

impl From<f64> for PvatTrajectoryFeedback {
    fn from(value: f64) -> Self {
        Self {
            points: vec![value.into()],
            tracking_status: TrackingStatus::tracking_valid(),
            motion_mode: String::from("TRACKING"),
            time_to_target: 0.0,
        }
    }
}

impl PvatTrajectoryFeedback {
    pub fn new(
        position: Vec<f64>,
        velocity: Vec<f64>,
        acceleration: Vec<f64>,
        tai: Vec<f64>,
    ) -> Self {
        Self {
            points: position
                .into_iter()
                .zip(velocity)
                .zip(acceleration)
                .zip(tai)
                .map(
                    |(((position, velocity), acceleration), tai)| PvatTrajectoryPoint {
                        position,
                        velocity,
                        acceleration,
                        tai,
                    },
                )
                .collect(),
            tracking_status: TrackingStatus::tracking_valid(),
            motion_mode: String::from("TRACKING"),
            time_to_target: 0.0,
        }
    }
}

impl ImMountFeedback {
    pub fn new(azimuth: Vec<f64>, elevation: Vec<f64>, gir: Vec<f64>, tai: Vec<f64>) -> Self {
        let velocity: Vec<f64> = vec![0.0; tai.len()];
        let acceleration: Vec<f64> = vec![0.0; tai.len()];
        Self {
            azimuth_feedback: PvatTrajectoryFeedback::new(
                azimuth,
                velocity.clone(),
                acceleration.clone(),
                tai.clone(),
            ),
            elevation_feedback: PvatTrajectoryFeedback::new(
                elevation,
                velocity.clone(),
                acceleration.clone(),
                tai.clone(),
            ),
            gir_feedback: PvatTrajectoryFeedback::new(
                gir,
                velocity.clone(),
                acceleration.clone(),
                tai.clone(),
            ),
        }
    }
}
