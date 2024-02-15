#[derive(Debug, Clone)]
pub enum WindLoaded {
    Mount,
    M1,
    M2,
    None,
}

/// ASMS facesheet builder
#[derive(Debug, Clone)]
pub struct WindLoads {
    mount: WindLoaded,
    m1: WindLoaded,
    m2: WindLoaded,
}

impl Default for WindLoads {
    fn default() -> Self {
        Self {
            mount: WindLoaded::Mount,
            m1: WindLoaded::M1,
            m2: WindLoaded::M2,
        }
    }
}

impl WindLoads {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn no_mount(&mut self) {
        self.mount = WindLoaded::None;
    }
    pub fn no_m1(&mut self) {
        self.m1 = WindLoaded::None;
    }
    pub fn no_m2(&mut self) {
        self.m2 = WindLoaded::None;
    }
}
