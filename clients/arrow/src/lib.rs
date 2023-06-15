/*!
# Actor client for Apache Arrow

A simulation data logger that records the data in the [Apache Arrow] format and
automatically saves the data into a [Parquet] file (`data.parquet`) at the end of a simulation.

[Apache Arrow]: https://docs.rs/arrow
[Parquet]: https://docs.rs/parquet

# Example

An Arrow logger setup for 1000 time steps
```
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_actors::prelude::*;
let logging = Arrow::builder(1000).build();
```
setting the name of the Parquet file
```
# use gmt_dos_clients_arrow::Arrow;
# use gmt_dos_actors::prelude::*;

let logging = Arrow::builder(1000)
                       .filename("my_data.parquet")
                       .build();
```
opting out of saving the data to the Parquet file
```
# use gmt_dos_clients_arrow::Arrow;
# use gmt_dos_actors::prelude::*;

let logging = Arrow::builder(1000)
                       .no_save()
                       .build();
```
Logging an output into an [Arrow] logger:
```
# tokio_test::block_on(async {
use gmt_dos_actors::prelude::*;
use gmt_dos_clients::Signals;
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients::interface::UID;

let logging = Arrow::builder(1000).build().into_arcx();
let mut sink = Terminator::<_>::new(logging);
let mut source: Initiator<_> = Signals::new(1, 100).into();
#[derive(UID)]
enum Source {};
source.add_output().build::<Source>().logn(&mut sink, 42).await;
# Ok::<(), gmt_dos_actors::model::ModelError>(())
# });
```
or if `Signals` implements the trait: `Size<Source>`
```
# tokio_test::block_on(async {
use gmt_dos_actors::prelude::*;
use gmt_dos_clients::Signals;
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients::interface::{Size, UID};

let logging = Arrow::builder(1000).build().into_arcx();
let mut sink = Terminator::<_>::new(logging);
let mut source: Initiator<_> = Signals::new(1, 100).into();
#[derive(UID)]
enum Source {};
impl Size<Source> for Signals {
    fn len(&self) -> usize {
        42
    }
}
source.add_output().build::<Source>().log(&mut sink).await;
# Ok::<(), gmt_dos_actors::model::ModelError>(())
# });
```
*/

use apache_arrow::{
    array::{Array, ArrayData, BufferBuilder, ListArray, PrimitiveArray},
    buffer::Buffer,
    datatypes::{ArrowNativeType, ArrowPrimitiveType, DataType, Field, ToByteSlice},
};
use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update};
use regex::Regex;
use std::{
    any::{type_name, Any},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Debug, thiserror::Error)]
pub enum ArrowError {
    #[error("cannot open a parquet file")]
    ArrowToFile(#[from] std::io::Error),
    #[error("cannot build Arrow data")]
    ArrowError(#[from] apache_arrow::error::ArrowError),
    #[error("cannot save data to Parquet")]
    ParquetError(#[from] parquet::errors::ParquetError),
    #[error("no record available")]
    NoRecord,
    #[error("Field {0} not found")]
    FieldNotFound(String),
    #[error("Parsing field {0} failed")]
    ParseField(String),
    #[cfg(feature = "matio-rs")]
    #[error("failed to save data to mat file")]
    MatFile(#[from] matio_rs::MatioError),
}

type Result<T> = std::result::Result<T, ArrowError>;

// Buffers 1GB max capacity
const MAX_CAPACITY_BYTE: usize = 2 << 29;

/// Format to write data to file
///
/// Use parquet as the default file format
pub enum FileFormat {
    Parquet,
    #[cfg(feature = "matio-rs")]
    Matlab(MatFormat),
}
impl Default for FileFormat {
    fn default() -> Self {
        Self::Parquet
    }
}
/// Matlab data format
///
/// The Matlab data format is either `SampleBased` and does not include the time vector
/// or is `TimeBased` and does include a time vector.
/// The default format is `SampledBased`
pub enum MatFormat {
    SampleBased,
    TimeBased(f64),
}
impl Default for MatFormat {
    fn default() -> Self {
        Self::SampleBased
    }
}

/// Buffers generic interface
trait BufferObject: Send + Sync {
    fn who(&self) -> String;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
    fn into_list(&mut self, n_step: usize, n: usize, data_type: DataType) -> Result<ListArray>;
}

/// Arrow buffer type match to a dos-actors Data type
struct ArrowBuffer<U: UniqueIdentifier>(PhantomData<U>);
impl<T: ArrowNativeType, U: UniqueIdentifier<DataType = Vec<T>>> UniqueIdentifier
    for ArrowBuffer<U>
{
    type DataType = BufferBuilder<T>;
}
struct LogData<U: UniqueIdentifier>(<U as UniqueIdentifier>::DataType, PhantomData<U>);
impl<U: UniqueIdentifier> Deref for LogData<U> {
    type Target = <U as UniqueIdentifier>::DataType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<U: UniqueIdentifier> DerefMut for LogData<U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T, U: UniqueIdentifier<DataType = T>> LogData<U> {
    pub fn new(data: T) -> Self {
        Self(data, PhantomData)
    }
}
impl<T, U> BufferObject for LogData<ArrowBuffer<U>>
where
    T: ArrowNativeType,
    U: 'static + Send + Sync + UniqueIdentifier<DataType = Vec<T>>,
{
    fn who(&self) -> String {
        let expression = type_name::<U>().to_string();
        let re = Regex::new(r"(\w+)(?:<(\d+)>)?$").unwrap();
        if let Some(captures) = re.captures(&expression) {
            let last_word = captures.get(1).unwrap().as_str();
            if let Some(number) = captures.get(2).map(|m| m.as_str()) {
                format!("{}#{}", last_word, number)
            } else {
                last_word.to_string()
            }
        } else {
            expression
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn into_list(&mut self, n_step: usize, n: usize, data_type: DataType) -> Result<ListArray> {
        let buffer = &mut *self;
        let data = ArrayData::builder(data_type.clone())
            .len(buffer.len())
            .add_buffer(buffer.finish())
            .build()?;
        let offsets = (0..).step_by(n).take(n_step + 1).collect::<Vec<i32>>();
        let list = ArrayData::builder(DataType::List(Box::new(Field::new(
            "values", data_type, false,
        ))))
        .len(n_step)
        .add_buffer(Buffer::from(&offsets.to_byte_slice()))
        .add_child_data(data)
        .build()?;
        Ok(ListArray::from(list))
    }
}

#[doc(hidden)]
pub trait BufferDataType {
    type ArrayType;
    fn buffer_data_type() -> DataType;
}
use paste::paste;
macro_rules! impl_buffer_types {
    ( $( ($rs:ty,$arw:expr) ),+ ) => {
	    $(
        paste! {
impl BufferDataType for $rs {
    type ArrayType = apache_arrow::datatypes::[<$arw Type>];
    fn buffer_data_type() -> DataType {
        apache_arrow::datatypes::DataType::$arw
    }
}
        }
		)+
    };
}

impl_buffer_types! {
(f64,Float64),
(f32,Float32),
(i64,Int64),
(i32,Int32),
(i16,Int16),
(i8 ,Int8),
(u64,UInt64),
(u32,UInt32),
(u16,UInt16),
(u8 ,UInt8)
}

enum DropOption {
    Save(Option<String>),
    NoSave,
}

mod arrow;
pub use arrow::{Arrow, ArrowBuilder};
pub trait Get<T>
where
    T: BufferDataType,
    <T as BufferDataType>::ArrayType: ArrowPrimitiveType,
    Vec<T>: FromIterator<<<T as BufferDataType>::ArrayType as ArrowPrimitiveType>::Native>,
{
    /// Return the record field entry
    fn get<S>(&mut self, field_name: S) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>;

    /// Return the record field entry skipping the first `skip` elements and taking all (None) or some (Some(`take`)) elements
    fn get_skip_take<S>(
        &mut self,
        field_name: S,
        skip: usize,
        take: Option<usize>,
    ) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>;
    /// Return the record field entry skipping the first `skip` elements
    fn get_skip<S>(&mut self, field_name: S, skip: usize) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>,
    {
        self.get_skip_take(field_name, skip, None)
    }
    /// Return the record field entry taking `take` elements
    fn get_take<S>(&mut self, field_name: S, take: usize) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>,
    {
        self.get_skip_take(field_name, 0, Some(take))
    }
}
impl<'a, T> Get<T> for Arrow
where
    T: BufferDataType,
    <T as BufferDataType>::ArrayType: ArrowPrimitiveType,
    Vec<T>: FromIterator<<<T as BufferDataType>::ArrayType as ArrowPrimitiveType>::Native>,
{
    /// Return the record field entry
    fn get<S>(&mut self, field_name: S) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>,
    {
        match self.record() {
            Ok(record) => match record.schema().column_with_name(field_name.as_ref()) {
                Some((idx, _)) => record
                    .column(idx)
                    .as_any()
                    .downcast_ref::<ListArray>()
                    .map(|data| {
                        data.iter()
                            .map(|data| {
                                data.map(|data| {
                                    data.as_any()
                                        .downcast_ref::<PrimitiveArray<<T as BufferDataType>::ArrayType>>()
                                        .and_then(|data| data.iter().collect::<Option<Vec<T>>>())
                                })
                                .flatten()
                            })
                            .collect::<Option<Vec<Vec<T>>>>()
                    })
                    .flatten()
                    .ok_or_else(|| ArrowError::ParseField(field_name.into())),
                None => Err(ArrowError::FieldNotFound(field_name.into())),
            },
            Err(e) => Err(e),
        }
    }
    /// Return the record field entry skipping the first `skip` elements and taking all (None) or some (Some(`take`)) elements
    fn get_skip_take<S>(
        &mut self,
        field_name: S,
        skip: usize,
        take: Option<usize>,
    ) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>,
    {
        match self.record() {
            Ok(record) => match record.schema().column_with_name(field_name.as_ref()) {
                Some((idx, _)) => record
                    .column(idx)
                    .as_any()
                    .downcast_ref::<ListArray>()
                    .map(|data| {
                        data.iter()
                            .skip(skip)
                            .take(take.unwrap_or(usize::MAX))
                            .map(|data| {
                                data.map(|data| {
                                    data.as_any()
                                        .downcast_ref::<PrimitiveArray<<T as BufferDataType>::ArrayType>>()
                                        .and_then(|data| data.iter().collect::<Option<Vec<T>>>())
                                })
                                .flatten()
                            })
                            .collect::<Option<Vec<Vec<T>>>>()
                    })
                    .flatten()
                    .ok_or_else(|| ArrowError::ParseField(field_name.into())),
                None => Err(ArrowError::FieldNotFound(field_name.into())),
            },
            Err(e) => Err(e),
        }
    }
}

impl Update for Arrow {}
impl<T, U> Read<U> for Arrow
where
    T: ArrowNativeType,
    U: 'static + UniqueIdentifier<DataType = Vec<T>>,
{
    fn read(&mut self, data: Data<U>) {
        let r = 1 + (self.step as f64 / self.n_entry as f64).floor() as usize;
        self.step += 1;
        if r % self.decimation > 0 {
            return;
        }
        if let Some(buffer) = self.data::<T, U>() {
            buffer.append_slice(&data);
            self.count += 1;
            match self.batch_size {
                Some(batch_size) if self.count % (self.n_entry * batch_size) == 0 => {
                    self.save();
                }
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use apache_arrow::datatypes::Schema;
    use gmt_dos_clients::interface::{Data, Entry, UID};

    use super::*;

    #[test]
    fn arrow() {
        let mut arw = Arrow::builder(10).build();
        #[derive(UID)]
        pub enum Data {}
        <Arrow as Entry<Data>>::entry(&mut arw, 1);

        let field = Field::new(
            "Data",
            DataType::List(Box::new(Field::new("values", DataType::Float64, false))),
            false,
        );
        let schema = Arc::new(Schema::new(vec![field]));
        assert_eq!(arw.record().unwrap().schema(), schema);
    }

    #[test]
    fn batch() {
        env_logger::init();
        let n_step = 8;
        let mut arw = Arrow::builder(n_step).batch_size(n_step / 2).build();
        #[derive(UID)]
        pub enum U {}
        <Arrow as Entry<U>>::entry(&mut arw, 1);
        for i in 0..n_step {
            arw.read(Data::<U>::new(vec![i as f64]));
        }
    }

    #[test]
    fn batch2() {
        env_logger::init();
        let n_step = 24;
        let mut arw = Arrow::builder(n_step).batch_size(4).build();
        #[derive(UID)]
        pub enum U {}
        <Arrow as Entry<U>>::entry(&mut arw, 1);
        #[derive(UID)]
        pub enum V {}
        <Arrow as Entry<V>>::entry(&mut arw, 3);
        for i in 0..n_step {
            arw.read(Data::<U>::new(vec![i as f64]));
            arw.read(Data::<V>::new(vec![(10 * i) as f64; 3]));
        }
    }
}
