use std::fmt::Display;

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct AberrationProperties {
    name: String,
    stroke: f64,
}

impl AberrationProperties {
    fn raw_polishing(stroke: Option<f64>) -> Self {
        AberrationProperties {
            name: "raw-polishing".into(),
            stroke: stroke.unwrap_or(1f64),
        }
    }
    fn print_through(stroke: Option<f64>) -> Self {
        AberrationProperties {
            name: "print-through".into(),
            stroke: stroke.unwrap_or(1f64),
        }
    }
    fn soak1deg(stroke: Option<f64>) -> Self {
        AberrationProperties {
            name: "soak1deg".into(),
            stroke: stroke.unwrap_or(1f64),
        }
    }
    fn name(&self) -> &str {
        self.name.as_str()
    }
    fn stroke(&self) -> f64 {
        self.stroke
    }
}

type MayBeProp = Option<AberrationProperties>;
#[derive(Default, Debug)]
pub struct Aberrations {
    raw_polishing: MayBeProp,
    print_through: MayBeProp,
    soak1deg: MayBeProp,
}

impl Aberrations {
    pub fn builder() -> AberrationsBuilder {
        AberrationsBuilder::new()
    }
    pub fn raw_polishing_stroke(&self) -> f64 {
        self.raw_polishing.as_ref().map_or(0f64, |a| a.stroke())
    }
    pub fn print_through_stroke(&self) -> f64 {
        self.print_through.as_ref().map_or(0f64, |a| a.stroke())
    }
    pub fn soak1deg_stroke(&self) -> f64 {
        self.soak1deg.as_ref().map_or(0f64, |a| a.stroke())
    }
}

impl Display for Aberrations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self
            .raw_polishing
            .iter()
            .chain(self.print_through.as_ref())
            .chain(self.soak1deg.as_ref())
            .map(|a| a.name().into())
            .collect::<Vec<String>>()
            .join("_");
        write!(f, "{name}")
    }
}

#[derive(Default, Debug)]
pub struct AberrationsBuilder {
    raw_polishing: MayBeProp,
    print_through: MayBeProp,
    soak1deg: MayBeProp,
}
impl AberrationsBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn raw_polishing(mut self, stroke: Option<f64>) -> Self {
        self.raw_polishing = Some(AberrationProperties::raw_polishing(stroke));
        self
    }
    pub fn print_through(mut self, stroke: Option<f64>) -> Self {
        self.print_through = Some(AberrationProperties::print_through(stroke));
        self
    }
    pub fn soak1deg(mut self, stroke: Option<f64>) -> Self {
        self.soak1deg = Some(AberrationProperties::soak1deg(stroke));
        self
    }
    pub fn build(self) -> Aberrations {
        Aberrations {
            raw_polishing: self.raw_polishing,
            print_through: self.print_through,
            soak1deg: self.soak1deg,
        }
    }
}
