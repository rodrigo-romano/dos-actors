use std::{collections::HashMap, mem::size_of};

use apache_arrow::{
    array::BufferBuilder,
    datatypes::{ArrowNativeType, DataType},
};
use interface::UniqueIdentifier;

use crate::{
    Arrow, ArrowBuffer, BufferDataType, BufferObject, DropOption, FileFormat, LogData,
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
    batch_size: Option<usize>,
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
            batch_size: None,
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
    /// Sets the size of the record batch in number of time steps
    ///
    /// The record will be written to file every time step that is a multiple of the batch size
    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = Some(batch_size);
        self
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
            batch: None,
            drop_option: self.drop_option,
            decimation: self.decimation,
            count: 0,
            file_format: self.file_format,
            batch_size: self.batch_size,
        }
    }
}
