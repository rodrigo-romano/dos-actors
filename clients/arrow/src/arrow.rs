use std::{collections::HashMap, env, fmt::Display, fs::File, mem::size_of, path::Path, sync::Arc};

use apache_arrow::{
    array::{Array, BufferBuilder},
    compute::concat_batches,
    datatypes::{ArrowNativeType, DataType, Field, Schema},
    record_batch::{RecordBatch, RecordBatchReader},
};
use gmt_dos_clients::interface::{print_info, Entry, UniqueIdentifier};
use parquet::{
    arrow::{arrow_reader::ParquetRecordBatchReaderBuilder, ArrowWriter},
    file::properties::WriterProperties,
};

use crate::{
    ArrowBuffer, ArrowError, BufferDataType, BufferObject, DropOption, FileFormat, LogData, Result,
    MAX_CAPACITY_BYTE,
};

/// Arrow format logger builder
pub struct ArrowBuilder {
    n_step: usize,
    capacities: Vec<usize>,
    buffers: Vec<(Box<dyn BufferObject>, DataType)>,
    metadata: Option<HashMap<String, String>>,
    n_entry: usize,
    drop_option: DropOption,
    decimation: usize,
    file_format: FileFormat,
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
            file_format: Default::default(),
        }
    }
    /// Adds an entry to the logger
    #[deprecated = "replaced by the log method of the InputLogs trait"]
    pub fn entry<T: BufferDataType, U>(self, size: usize) -> Self
    where
        T: 'static + ArrowNativeType + Send + Sync,
        U: 'static + Send + Sync + UniqueIdentifier<DataType = Vec<T>>,
    {
        let mut buffers = self.buffers;
        let mut capacity = size * (1 + self.n_step / self.decimation);
        //log::info!("Buffer capacity: {}", capacity);
        if capacity * size_of::<T>() > MAX_CAPACITY_BYTE {
            capacity = MAX_CAPACITY_BYTE / size_of::<T>();
            log::info!("Capacity limit of 1GB exceeded, reduced to : {}", capacity);
        }
        let buffer: LogData<ArrowBuffer<U>> = LogData::new(BufferBuilder::<T>::new(capacity));
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
    /// Sets the file format (default: Parquet)
    pub fn file_format(self, file_format: FileFormat) -> Self {
        Self {
            file_format,
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
            file_format: self.file_format,
        }
    }
}

/// Apache [Arrow](https://docs.rs/arrow) client
pub struct Arrow {
    n_step: usize,
    capacities: Vec<usize>,
    buffers: Vec<(Box<dyn BufferObject>, DataType)>,
    metadata: Option<HashMap<String, String>>,
    pub(crate) step: usize,
    pub(crate) n_entry: usize,
    record: Option<RecordBatch>,
    drop_option: DropOption,
    pub(crate) decimation: usize,
    pub(crate) count: usize,
    file_format: FileFormat,
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
            file_format: Default::default(),
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
    pub(crate) fn data<T, U>(&mut self) -> Option<&mut LogData<ArrowBuffer<U>>>
    where
        T: 'static + ArrowNativeType,
        U: 'static + UniqueIdentifier<DataType = Vec<T>>,
    {
        self.buffers
            .iter_mut()
            .find_map(|(b, _)| b.as_mut_any().downcast_mut::<LogData<ArrowBuffer<U>>>())
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
    U: 'static + Send + Sync + UniqueIdentifier<DataType = Vec<T>>,
{
    fn entry(&mut self, size: usize) {
        let mut capacity = size * (1 + self.n_step / self.decimation);
        //log::info!("Buffer capacity: {}", capacity);
        if capacity * size_of::<T>() > MAX_CAPACITY_BYTE {
            capacity = MAX_CAPACITY_BYTE / size_of::<T>();
            log::info!("Capacity limit of 1GB exceeded, reduced to : {}", capacity);
        }
        let buffer: LogData<ArrowBuffer<U>> = LogData::new(BufferBuilder::<T>::new(capacity));
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
        log::info!("{self}");
        match self.drop_option {
            DropOption::Save(ref filename) => {
                let file_name = filename
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| "data".to_string());
                match self.file_format {
                    FileFormat::Parquet => {
                        if let Err(e) = self.to_parquet(file_name) {
                            print_info("Arrow error", Some(&e));
                        }
                    }
                    #[cfg(feature = "matio-rs")]
                    FileFormat::Matlab(_) => {
                        if let Err(e) = self.to_mat(file_name) {
                            print_info("Arrow error", Some(&e));
                        }
                    }
                }
            }
            DropOption::NoSave => {
                log::info!("Dropping Arrow logger without saving.");
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
        log::info!("Arrow data saved to {root:?}");
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
        let parquet_reader = ParquetRecordBatchReaderBuilder::try_new(file)?
            .with_batch_size(2048)
            .build()?;
        let schema = parquet_reader.schema();
        let records: std::result::Result<Vec<_>, apache_arrow::error::ArrowError> =
            parquet_reader.collect();
        let record = concat_batches(&schema, records?.as_slice())?;
        Ok(Arrow {
            n_step: 0,
            capacities: Vec::new(),
            buffers: Vec::new(),
            metadata: None,
            step: 0,
            n_entry: 0,
            record: Some(record),
            drop_option: DropOption::NoSave,
            decimation: 1,
            count: 0,
            file_format: FileFormat::Parquet,
        })
    }
    #[cfg(feature = "matio-rs")]
    /// Saves the data to a Matlab "mat" file
    ///
    /// All data must be of the type `Vec<f64>`
    pub fn to_mat<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        use matio_rs::{MatFile, MatVar, Save};
        let batch = self.record()?;
        let root_env = env::var("DATA_REPO").unwrap_or_else(|_| ".".to_string());
        let root = Path::new(&root_env).join(&path).with_extension("mat");
        let mat_file = MatFile::save(&root)?;
        let mut n_sample = 0;
        for field in batch.schema().fields() {
            let name = field.name();
            let data: Vec<Vec<f64>> = self.get(name)?;
            n_sample = data.len();
            let n_data = data[0].len();
            mat_file.write(MatVar::<Vec<f64>>::array(
                name,
                data.into_iter()
                    .flatten()
                    .collect::<Vec<f64>>()
                    .as_mut_slice(),
                (n_data, n_sample),
            )?);
        }
        if let FileFormat::Matlab(MatFormat::TimeBased(sampling_frequency)) = self.file_format {
            let tau = sampling_frequency.recip();
            let time: Vec<f64> = (0..n_sample).map(|i| i as f64 * tau).collect();
            mat_file.write(MatVar::<Vec<f64>>::new("time", time.as_slice())?);
        }
        log::info!("Arrow data saved to {root:?}");
        Ok(())
    }
}
