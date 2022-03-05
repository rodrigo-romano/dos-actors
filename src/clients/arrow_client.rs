//! Actor client for Apache [Arrow](https://docs.rs/arrow)

use crate::{
    io::{Data, Read},
    Update, Who,
};
use arrow::{
    array::{Array, ArrayData, BufferBuilder, ListArray},
    buffer::Buffer,
    datatypes::{ArrowNativeType, DataType, Field, Schema, ToByteSlice},
    record_batch::RecordBatch,
};
use parquet::{arrow::arrow_writer::ArrowWriter, file::properties::WriterProperties};
use std::{any::Any, collections::HashMap, fmt::Display, fs::File, path::Path, sync::Arc};

type Result<T> = std::result::Result<T, super::ClientError>;

trait BufferObject: Send + Sync {
    fn who(&self) -> String;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
    fn into_list(&mut self, n_step: usize, n: usize) -> Result<ListArray>;
}

impl<T: ArrowNativeType, U: 'static + Send + Sync> BufferObject for Data<BufferBuilder<T>, U> {
    fn who(&self) -> String {
        Who::who(self)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn into_list(&mut self, n_step: usize, n: usize) -> Result<ListArray> {
        let buffer = &mut *self;
        let data = ArrayData::builder(DataType::Float64)
            .len(buffer.len())
            .add_buffer(buffer.finish())
            .build()?;
        let offsets = (0..).step_by(n).take(n_step + 1).collect::<Vec<i32>>();
        let list = ArrayData::builder(DataType::List(Box::new(Field::new(
            "values",
            DataType::Float64,
            false,
        ))))
        .len(n_step)
        .add_buffer(Buffer::from(&offsets.to_byte_slice()))
        .add_child_data(data)
        .build()?;
        Ok(ListArray::from(list))
    }
}

pub struct ArrowBuilder {
    n_step: usize,
    capacities: Vec<usize>,
    buffers: Vec<Box<dyn BufferObject>>,
    metadata: Option<HashMap<String, String>>,
    n_entry: usize,
    filename: Option<String>,
}
impl ArrowBuilder {
    pub fn new(n_step: usize) -> Self {
        Self {
            n_step,
            capacities: Vec::new(),
            buffers: Vec::new(),
            metadata: None,
            n_entry: 0,
            filename: None,
        }
    }
    pub fn entry<T, U>(self, size: usize) -> Self
    where
        T: 'static + ArrowNativeType + Send + Sync,
        U: 'static + Send + Sync,
    {
        let mut buffers = self.buffers;
        let buffer: Data<BufferBuilder<T>, U> =
            Data::new(BufferBuilder::<T>::new(size * self.n_step));
        buffers.push(Box::new(buffer));
        let mut capacities = self.capacities;
        capacities.push(size);
        Self {
            buffers,
            capacities,
            n_entry: self.n_entry + 1,
            ..self
        }
    }
    pub fn build(self) -> Arrow {
        if self.n_entry == 0 {
            panic!("There are no entries in the Arrow data logger.");
        }
        Arrow {
            n_step: self.n_step,
            capacities: self.capacities,
            buffers: self.buffers,
            metadata: self.metadata,
            step: 0,
            n_entry: self.n_entry,
            filename: self.filename,
        }
    }
}

/// Apache [Arrow](https://docs.rs/arrow) client
pub struct Arrow {
    n_step: usize,
    capacities: Vec<usize>,
    buffers: Vec<Box<dyn BufferObject>>,
    metadata: Option<HashMap<String, String>>,
    step: usize,
    n_entry: usize,
    filename: Option<String>,
}
impl Arrow {
    /// Creates a new Apache [Arrow](https://docs.rs/arrow) data logger
    ///
    ///  - `n_step`: the number of time step
    pub fn builder(n_step: usize) -> ArrowBuilder {
        ArrowBuilder::new(n_step)
    }
    fn data<T, U>(&mut self) -> Option<&mut Data<BufferBuilder<T>, U>>
    where
        T: 'static + ArrowNativeType,
        U: 'static,
    {
        self.buffers
            .iter_mut()
            .find_map(|b| b.as_mut_any().downcast_mut::<Data<BufferBuilder<T>, U>>())
    }
    pub fn pct_complete(&self) -> usize {
        self.step / self.n_step / self.n_entry
    }
}

impl Display for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Arrow logger:")?;
        writeln!(
            f,
            " - steps #: {}/{}",
            self.n_step,
            self.step / self.n_entry
        )?;
        writeln!(f, " - data:")?;
        for (buffer, capacity) in self.buffers.iter().zip(self.capacities.iter()) {
            writeln!(f, "   - {:>8}:{:>4}", buffer.who(), capacity)?;
        }
        Ok(())
    }
}

impl Drop for Arrow {
    fn drop(&mut self) {
        println!("{self}");
        let filename = self
            .filename
            .as_ref()
            .cloned()
            .unwrap_or("data.parquet".to_string());
        if let Err(e) = self.to_parquet(filename) {
            println!("{e}");
        }
    }
}
impl Arrow {
    /// Saves the data to a [Parquet](https://docs.rs/parquet) data file
    pub fn to_parquet<P: AsRef<Path> + std::fmt::Debug>(&mut self, path: P) -> Result<()> {
        let mut lists: Vec<Arc<dyn Array>> = vec![];
        for (buffer, n) in self.buffers.iter_mut().zip(self.capacities.iter()) {
            let list = buffer.into_list(self.n_step, *n)?;
            lists.push(Arc::new(list));
        }

        let fields: Vec<_> = self
            .buffers
            .iter()
            .map(|buffer| {
                Field::new(
                    &buffer.who().split("::").last().unwrap_or("no name"),
                    DataType::List(Box::new(Field::new("values", DataType::Float64, false))),
                    false,
                )
            })
            .collect();
        let schema = Arc::new(if let Some(metadata) = self.metadata.as_ref() {
            Schema::new_with_metadata(fields, metadata.clone())
        } else {
            Schema::new(fields)
        });

        let batch = RecordBatch::try_new(Arc::clone(&schema), lists)?;

        let file = File::create(&path)?;
        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, Arc::clone(&schema), Some(props))?;
        writer.write(&batch)?;
        writer.close()?;
        println!("Data saved to {path:?}");
        Ok(())
    }
}

impl Update for Arrow {}
impl<T, U> Read<Vec<T>, U> for Arrow
where
    T: ArrowNativeType,
    U: 'static,
{
    fn read(&mut self, data: Arc<Data<Vec<T>, U>>) {
        /*log::debug!(
            "receive #{} inputs: {:?}",
            data.len(),
            data.iter().map(|x| x.len()).collect::<Vec<usize>>()
        );*/
        self.step += 1;
        if let Some(buffer_data) = self.data::<T, U>() {
            let buffer = &mut *buffer_data;
            buffer.append_slice((**data).as_slice());
        }
    }
}
