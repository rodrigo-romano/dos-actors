//! # Units conversion
//!
//! Converts data given in the [MKS] system of units
//!
//! The conversion is apply within an implementation of the [Write] trait.
//!
//! The conversion is performed by wrapping a type `U` in one of the 4 conversion types: [NM], [MuM], [Arcsec], [Mas]
//! e.g. `NM<U>` will apply the conversion into nanometers to the data represented by `U` when invoking [Write]`::<NM<U>>::write()`.
//!
//! `U` must implements the [UniqueIdentifier] trait with `Vec` as [UniqueIdentifier::DataType]
//! i.e. the bound on `U` is  `U: UniqueIdentifier<DataType = Vec<T>>`
//!
//!
//! [MKS]: https://en.wikipedia.org/wiki/MKS_system_of_units

use std::{any::type_name, f64::consts::PI, marker::PhantomData, ops::Mul};

use crate::Units;

use super::{Data, UniqueIdentifier, Update, Write};

/// Conversion to nanometers
pub struct NM<U>(PhantomData<U>);
impl<U: UniqueIdentifier> UnitsConversion for NM<U> {
    const UNITS: f64 = 1e9;
    type ID = U;
}

/// Conversion to micrometers
pub struct MuM<U: UniqueIdentifier>(PhantomData<U>);
impl<U: UniqueIdentifier> UnitsConversion for MuM<U> {
    const UNITS: f64 = 1e6;
    type ID = U;
}

/// Conversion to arcseconds
pub struct Arcsec<U: UniqueIdentifier>(PhantomData<U>);
impl<U: UniqueIdentifier> UnitsConversion for Arcsec<U> {
    const UNITS: f64 = (180. * 3600.) / PI;
    type ID = U;
}

/// Conversion to milli-arcseconds
pub struct Mas<U: UniqueIdentifier>(PhantomData<U>);
impl<U: UniqueIdentifier> UnitsConversion for Mas<U> {
    const UNITS: f64 = (180. * 3600e3) / PI;
    type ID = U;
}

/*
------------------------------------------------------------------------------------------
                            Below is where the magic happens!
------------------------------------------------------------------------------------------
*/

/// Blanket implementation of [UniqueIdentifier] for types that implement [UnitsConversion]
impl<U, W> UniqueIdentifier for W
where
    U: UniqueIdentifier,
    W: UnitsConversion<ID = U> + Send + Sync,
{
    const PORT: u32 = <U as UniqueIdentifier>::PORT;
    type DataType = <U as UniqueIdentifier>::DataType;
}

/// Trait performing the units conversion
pub trait UnitsConversion {
    /// Conversion scale factor
    const UNITS: f64;
    type ID: UniqueIdentifier;

    /// Converts data given in MKSA system
    fn conversion<T>(data: &Data<Self::ID>) -> Result<Data<Self>, String>
    where
        Self::ID: UniqueIdentifier<DataType = Vec<T>>,
        T: Copy + TryFrom<f64> + Mul<T, Output = T>,
        <T as TryFrom<f64>>::Error: std::fmt::Debug,
        Self: UniqueIdentifier<DataType = Vec<T>> + Sized,
    {
        let msg = format!(
            "failed to convert f64 to {} in Write<{}>::write",
            type_name::<T>(),
            type_name::<Self>()
        );
        let s: T = T::try_from(Self::UNITS).map_err(|_| msg)?;
        let data: Vec<_> = Into::<&[T]>::into(data).iter().map(|x| *x * s).collect();
        Ok(data.into())
    }
}

/// Blanket implementation of [Write] for clients that implement [Update], [Write] and [Units]
impl<T, U, C, W> Write<W> for C
where
    T: Copy + TryFrom<f64> + Mul<T, Output = T>,
    <T as TryFrom<f64>>::Error: std::fmt::Debug,
    U: UniqueIdentifier<DataType = Vec<T>>,
    C: Update + Write<U> + Units,
    W: UniqueIdentifier<DataType = Vec<T>> + UnitsConversion<ID = U>,
{
    fn write(&mut self) -> Option<Data<W>> {
        <C as Write<U>>::write(self)
            .as_ref()
            .map(|data| <W as UnitsConversion>::conversion(data).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub enum W {}
    impl UniqueIdentifier for W {
        type DataType = Vec<f64>;
    }
    #[derive(Default)]
    pub struct Client {
        pub data: Vec<f64>,
    }
    impl Update for Client {}
    impl Write<W> for Client {
        fn write(&mut self) -> Option<Data<W>> {
            Some(vec![1e-9, 2e-6, 3e-3].into())
        }
    }

    #[derive(Default)]
    pub struct ClientAngle {
        pub data: Vec<f64>,
    }
    impl Update for ClientAngle {}
    impl Write<W> for ClientAngle {
        fn write(&mut self) -> Option<Data<W>> {
            Some(vec![1., 1e-3].into())
        }
    }

    impl Units for Client {}
    impl Units for ClientAngle {}

    #[test]
    fn units_nm() {
        let mut client = Client::default();
        let data = <Client as Write<NM<W>>>::write(&mut client);
        dbg!(data);
    }

    #[test]
    fn units_mum() {
        let mut client = Client::default();
        let data = <Client as Write<MuM<W>>>::write(&mut client);
        dbg!(data);
    }

    #[test]
    fn units_arcsec() {
        let mut client = ClientAngle::default();
        let data = <ClientAngle as Write<Arcsec<W>>>::write(&mut client);
        dbg!(data);
    }

    #[test]
    fn units_milli_arcsec() {
        let mut client = ClientAngle::default();
        let data = <ClientAngle as Write<Mas<W>>>::write(&mut client);
        dbg!(data);
    }
}
