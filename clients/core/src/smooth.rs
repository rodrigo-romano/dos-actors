use super::{Data, Read, UniqueIdentifier, Update, Write};

/// Smooth a signal with a time varying [Weight] input
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Smooth {
    weight: f64,
    data: Vec<f64>,
    data0: Option<Vec<f64>>,
}
impl Smooth {
    pub fn new() -> Self {
        Self {
            weight: 0f64,
            data: Vec::new(),
            data0: None,
        }
    }
}
impl Update for Smooth {}
/// Weight signal
pub enum Weight {}
impl UniqueIdentifier for Weight {
    type DataType = f64;
}
impl Read<Weight> for Smooth {
    fn read(&mut self, data: Data<Weight>) {
        let w: &f64 = &data;
        self.weight = *w;
    }
}
impl<U: UniqueIdentifier<DataType = Vec<f64>>> Read<U> for Smooth {
    fn read(&mut self, data: Data<U>) {
        let u: &[f64] = &data;
        self.data = u.to_vec();
        if self.data0.is_none() {
            self.data0 = Some(self.data.clone());
        }
    }
}
impl<U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for Smooth {
    fn write(&mut self) -> Option<Data<U>> {
        let y: Vec<_> = self.data.iter().map(|&u| u * self.weight).collect();
        Some(Data::new(y))
    }
}
