use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub sampling: usize,
    pub damping: f64,
}
impl Config {
    pub fn load() -> ron::error::SpannedResult<Self> {
        let config_str = include_str!("config.ron");
        ron::from_str(config_str)
    }
}
