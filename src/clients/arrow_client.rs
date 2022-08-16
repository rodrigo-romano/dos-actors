/*!
# Actor client for Apache Arrow

A simulation data logger that records the data in the [Apache Arrow] format and
automatically saves the data into a [Parquet] file (`data.parquet`) at the end of a simulation.

*The [Arrow] client is enabled with the `apache-arrow` feature.*

[Apache Arrow]: https://docs.rs/arrow
[Parquet]: https://docs.rs/parquet

# Example

An Arrow logger setup for 1000 time steps
```no_run
use dos_actors::clients::arrow_client::Arrow;
use dos_actors::prelude::*;
let logging = Arrow::builder(1000).build();
```
setting the name of the Parquet file
```no_run
# use dos_actors::clients::arrow_client::Arrow;
# use dos_actors::prelude::*;
let logging = Arrow::builder(1000)
                       .filename("my_data.parquet")
                       .build();
```
opting out of saving the data to the Parquet file
```
# use dos_actors::clients::arrow_client::Arrow;
# use dos_actors::prelude::*;
let logging = Arrow::builder(1000)
                       .no_save()
                       .build();
```
Logging an output into an [Arrow] logger:
```
# tokio_test::block_on(async {
use dos_actors::prelude::*;
use dos_actors::clients::arrow_client::Arrow;
let logging = Arrow::builder(1000).build().into_arcx();
let mut sink = Terminator::<_>::new(logging);
let mut source: Initiator<_> = Signals::new(1, 100).into();
#[derive(UID)]
enum Source {};
source.add_output().build::<Source>().log(&mut sink, 42).await;
# Ok::<(), dos_actors::model::ModelError>(())
# });
```
*/

use crate::{
    io::{Data, Read},
    print_error, Entry, UniqueIdentifier, Update, Who,
};
use arrow::{
    array::{Array, ArrayData, BufferBuilder, ListArray, PrimitiveArray},
    buffer::Buffer,
    datatypes::{ArrowNativeType, ArrowPrimitiveType, DataType, Field, Schema, ToByteSlice},
    record_batch::RecordBatch,
};
use parquet::{
    arrow::{arrow_writer::ArrowWriter, ArrowReader, ParquetFileArrowReader},
    file::{properties::WriterProperties, reader::SerializedFileReader},
};
use std::{
    any::Any, collections::HashMap, env, fmt::Display, fs::File, marker::PhantomData, mem::size_of,
    path::Path, sync::Arc,
};

#[derive(Debug, thiserror::Error)]
pub enum ArrowError {
    #[error("cannot open a parquet file")]
    ArrowToFile(#[from] std::io::Error),
    #[error("cannot build Arrow data")]
    ArrowError(#[from] arrow::error::ArrowError),
    #[error("cannot save data to Parquet")]
    ParquetError(#[from] parquet::errors::ParquetError),
    #[error("no record available")]
    NoRecord,
    #[error("Field {0} not found")]
    FieldNotFound(String),
    #[error("Parsing field {0} failed")]
    ParseField(String),
}

type Result<T> = std::result::Result<T, ArrowError>;

// Buffers 1GB max capacity
const MAX_CAPACITY_BYTE: usize = 2 << 29;

/// Buffers generic interface
trait BufferObject: Send + Sync {
    fn who(&self) -> String;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
    fn into_list(&mut self, n_step: usize, n: usize, data_type: DataType) -> Result<ListArray>;
}

/// Arrow buffer type match to a dos-actors Data type
struct ArrowBuffer<U: UniqueIdentifier>(PhantomData<U>);
impl<T: ArrowNativeType, U: UniqueIdentifier<Data = Vec<T>>> UniqueIdentifier for ArrowBuffer<U> {
    type Data = BufferBuilder<T>;
}

impl<T, U> BufferObject for Data<ArrowBuffer<U>>
where
    T: ArrowNativeType,
    U: 'static + Send + Sync + UniqueIdentifier<Data = Vec<T>>,
{
    fn who(&self) -> String {
        Who::who(self)
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
    type ArrayType = arrow::datatypes::[<$arw Type>];
    fn buffer_data_type() -> DataType {
        arrow::datatypes::DataType::$arw
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

/// Arrow format logger builder
pub struct ArrowBuilder {
    n_step: usize,
    capacities: Vec<usize>,
    buffers: Vec<(Box<dyn BufferObject>, DataType)>,
    metadata: Option<HashMap<String, String>>,
    n_entry: usize,
    drop_option: DropOption,
    decimation: usize,
}

impl ArrowBuilder {
    /// Creates a new Arrow logger builder
    pub fn new(n_step: usize) -> Self {
        Self {
            n_step,
            capacities: Vec::new(),
            buffers: Vec::new(),
            metadata: None,
            n_entry: 0,
            drop_option: DropOption::Save(None),
            decimation: 1,
        }
    }
    /// Adds an entry to the logger
    #[deprecated = "replaced by the log method of the InputLogs trait"]
    pub fn entry<T: BufferDataType, U>(self, size: usize) -> Self
    where
        T: 'static + ArrowNativeType + Send + Sync,
        U: 'static + Send + Sync + UniqueIdentifier<Data = Vec<T>>,
    {
        let mut buffers = self.buffers;
        let mut capacity = size * (1 + self.n_step / self.decimation);
        //log::info!("Buffer capacity: {}", capacity);
        if capacity * size_of::<T>() > MAX_CAPACITY_BYTE {
            capacity = MAX_CAPACITY_BYTE / size_of::<T>();
            log::info!("Capacity limit of 1GB exceeded, reduced to : {}", capacity);
        }
        let buffer: Data<ArrowBuffer<U>> = Data::new(BufferBuilder::<T>::new(capacity));
        buffers.push((Box::new(buffer), T::buffer_data_type()));
        let mut capacities = self.capacities;
        capacities.push(size);
        Self {
            buffers,
            capacities,
            n_entry: self.n_entry + 1,
            ..self
        }
    }
    /// Sets the name of the file to save the data to (default: "data.parquet")
    pub fn filename<S: Into<String>>(self, filename: S) -> Self {
        Self {
            drop_option: DropOption::Save(Some(filename.into())),
            ..self
        }
    }
    /// No saving to parquet file
    pub fn no_save(self) -> Self {
        Self {
            drop_option: DropOption::NoSave,
            ..self
        }
    }
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
    /// Decimate the data by the given factor
    pub fn decimation(self, decimation: usize) -> Self {
        Self { decimation, ..self }
    }
    /// Builds the Arrow logger
    pub fn build(self) -> Arrow {
        /*if self.n_entry == 0 {
            panic!("There are no entries in the Arrow data logger.");
        }*/
        Arrow {
            n_step: self.n_step,
            capacities: self.capacities,
            buffers: self.buffers,
            metadata: self.metadata,
            step: 0,
            n_entry: self.n_entry,
            record: None,
            drop_option: self.drop_option,
            decimation: self.decimation,
            count: 0,
        }
    }
}
/// Apache [Arrow](https://docs.rs/arrow) client
pub struct Arrow {
    n_step: usize,
    capacities: Vec<usize>,
    buffers: Vec<(Box<dyn BufferObject>, DataType)>,
    metadata: Option<HashMap<String, String>>,
    step: usize,
    n_entry: usize,
    record: Option<RecordBatch>,
    drop_option: DropOption,
    decimation: usize,
    count: usize,
}
impl Default for Arrow {
    fn default() -> Self {
        Arrow {
            n_step: 0,
            capacities: Vec::new(),
            buffers: Vec::new(),
            metadata: None,
            step: 0,
            n_entry: 0,
            record: None,
            drop_option: DropOption::NoSave,
            decimation: 1,
            count: 0,
        }
    }
}
impl Arrow {
    /// Creates a new Apache [Arrow](https://docs.rs/arrow) data logger
    ///
    ///  - `n_step`: the number of time step
    pub fn builder(n_step: usize) -> ArrowBuilder {
        ArrowBuilder::new(n_step)
    }
    fn data<T, U>(&mut self) -> Option<&mut Data<ArrowBuffer<U>>>
    where
        T: 'static + ArrowNativeType,
        U: 'static + UniqueIdentifier<Data = Vec<T>>,
    {
        self.buffers
            .iter_mut()
            .find_map(|(b, _)| b.as_mut_any().downcast_mut::<Data<ArrowBuffer<U>>>())
    }
    pub fn pct_complete(&self) -> usize {
        self.step / self.n_step / self.n_entry
    }
    pub fn size(&self) -> usize {
        self.step / self.n_entry
    }
}

impl<T, U> Entry<U> for Arrow
where
    T: 'static + BufferDataType + ArrowNativeType + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<Data = Vec<T>>,
{
    fn entry(&mut self, size: usize) {
        let mut capacity = size * (1 + self.n_step / self.decimation);
        //log::info!("Buffer capacity: {}", capacity);
        if capacity * size_of::<T>() > MAX_CAPACITY_BYTE {
            capacity = MAX_CAPACITY_BYTE / size_of::<T>();
            log::info!("Capacity limit of 1GB exceeded, reduced to : {}", capacity);
        }
        let buffer: Data<ArrowBuffer<U>> = Data::new(BufferBuilder::<T>::new(capacity));
        self.buffers.push((Box::new(buffer), T::buffer_data_type()));
        self.capacities.push(size);
        self.n_entry += 1;
    }
}
/*
impl<T, U> Entry<Vec<T>, U> for Arrow
where
    T: 'static + BufferDataType + ArrowNativeType + Send + Sync,
    U: 'static + Send + Sync,
{
    fn entry(&mut self, size: usize) {
        <Arrow as Entry<T, U>>::entry(self, size);
    }
}
 */
impl Display for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.n_entry > 0 {
            writeln!(f, "Arrow logger:")?;
            writeln!(f, " - data:")?;
            for ((buffer, _), capacity) in self.buffers.iter().zip(self.capacities.iter()) {
                writeln!(f, "   - {:>8}:{:>4}", buffer.who(), capacity)?;
            }
            write!(
                f,
                " - steps #: {}/{}/{}",
                self.n_step,
                self.step / self.n_entry,
                self.count / self.n_entry
            )?;
        }
        Ok(())
    }
}

impl Drop for Arrow {
    fn drop(&mut self) {
        println!("{self}");
        match self.drop_option {
            DropOption::Save(ref filename) => {
                let file_name = filename
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| "data.parquet".to_string());
                if let Err(e) = self.to_parquet(file_name) {
                    print_error("Arrow error", &e);
                }
            }
            DropOption::NoSave => {
                println!("Dropping Arrow logger without saving.");
            }
        }
    }
}
impl Arrow {
    /// Returns the data record
    pub fn record(&mut self) -> Result<&RecordBatch> {
        if self.record.is_none() {
            let mut lists: Vec<Arc<dyn Array>> = vec![];
            for ((buffer, buffer_data_type), n) in self.buffers.iter_mut().zip(&self.capacities) {
                let list =
                    buffer.into_list(self.count / self.n_entry, *n, buffer_data_type.clone())?;
                lists.push(Arc::new(list));
            }

            let fields: Vec<_> = self
                .buffers
                .iter()
                .map(|(buffer, data_type)| {
                    Field::new(
                        &buffer
                            .who()
                            .split("::")
                            .last()
                            .unwrap_or("no name")
                            .replace(">", ""),
                        DataType::List(Box::new(Field::new("values", data_type.clone(), false))),
                        false,
                    )
                })
                .collect();
            let schema = Arc::new(if let Some(metadata) = self.metadata.as_ref() {
                Schema::new_with_metadata(fields, metadata.clone())
            } else {
                Schema::new(fields)
            });

            self.record = Some(RecordBatch::try_new(Arc::clone(&schema), lists)?);
        }
        self.record.as_ref().ok_or(ArrowError::NoRecord)
    }
    /// Saves the data to a [Parquet](https://docs.rs/parquet) data file
    ///
    /// The [Parquet](https://docs.rs/parquet) data file is saved in the current directory
    /// unless the environment variable `DATA_REPO` is set to another directory
    pub fn to_parquet<P: AsRef<Path> + std::fmt::Debug>(&mut self, path: P) -> Result<()> {
        let batch = self.record()?;
        let root_env = env::var("DATA_REPO").unwrap_or_else(|_| ".".to_string());
        let root = Path::new(&root_env).join(&path).with_extension("parquet");
        let file = File::create(&root)?;
        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, Arc::clone(&batch.schema()), Some(props))?;
        writer.write(&batch)?;
        writer.close()?;
        println!("Arrow data saved to {root:?}");
        Ok(())
    }
    /// Loads data from a [Parquet](https://docs.rs/parquet) data file
    ///
    /// The [Parquet](https://docs.rs/parquet) data file is loaded from the current
    /// directory unless the environment variable `DATA_REPO` is set to another directory
    pub fn from_parquet<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let root_env = env::var("DATA_REPO").unwrap_or_else(|_| ".".to_string());
        let root = Path::new(&root_env);
        let filename = root.join(&path).with_extension("parquet");
        let file = File::open(&filename)?;
        log::info!("Loading {:?}", filename);
        let file_reader = SerializedFileReader::new(file)?;
        let mut arrow_reader = ParquetFileArrowReader::new(Arc::new(file_reader));
        let records = arrow_reader
            .get_record_reader(2048)
            .unwrap()
            .collect::<std::result::Result<Vec<RecordBatch>, arrow::error::ArrowError>>()?;
        let schema = records.get(0).unwrap().schema();
        Ok(Arrow {
            n_step: 0,
            capacities: Vec::new(),
            buffers: Vec::new(),
            metadata: None,
            step: 0,
            n_entry: 0,
            record: Some(RecordBatch::concat(&schema, &records)?),
            drop_option: DropOption::NoSave,
            decimation: 1,
            count: 0,
        })
    }
}
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
impl<T, U> Read<Vec<T>, U> for Arrow
where
    T: ArrowNativeType,
    U: 'static + UniqueIdentifier<Data = Vec<T>>,
{
    fn read(&mut self, data: Arc<Data<U>>) {
        let r = 1 + (self.step as f64 / self.n_entry as f64).floor() as usize;
        self.step += 1;
        if r % self.decimation > 0 {
            return;
        }
        if let Some(buffer) = self.data::<T, U>() {
            buffer.append_slice(data.as_slice());
            self.count += 1;
        }
    }
}
