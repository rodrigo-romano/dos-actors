use super::DeltaOPD;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct DeltaOPDSubset<'a>(pub(crate) Vec<&'a DeltaOPD>);
impl<'a> Deref for DeltaOPDSubset<'a> {
    type Target = Vec<&'a DeltaOPD>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> DerefMut for DeltaOPDSubset<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
