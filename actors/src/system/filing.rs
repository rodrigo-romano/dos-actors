use crate::system::{Sys, System};

impl<T> interface::filing::Codec for Sys<T>
where
    T: System,
    T: Sized + serde::ser::Serialize,
    for<'de> T: serde::de::Deserialize<'de>,
{
}

impl<T> interface::filing::Filing for Sys<T>
where
    T: System,
    T: Sized + serde::ser::Serialize,
    for<'de> T: serde::de::Deserialize<'de>,
{
}
