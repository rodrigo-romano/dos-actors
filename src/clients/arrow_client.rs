//! Actor client for Apache [Arrow](https://docs.rs/arrow)

use arrow::{
    array::{Array, ArrayData, BufferBuilder, ListArray},
    buffer::Buffer,
    datatypes::{ArrowNativeType, DataType, Field, Schema, ToByteSlice},
    record_batch::RecordBatch,
};
use parquet::{arrow::arrow_writer::ArrowWriter, file::properties::WriterProperties};
use std::{collections::HashMap, fmt::Display, fs::File, path::Path, sync::Arc};

type Result<T> = std::result::Result<T, super::ClientError>;

/// Apache [Arrow](https://docs.rs/arrow) client
#[derive(Debug)]
pub struct Arrow<T>
where
    T: ArrowNativeType,
{
    n_step: usize,
    names: Vec<String>,
    capacities: Vec<usize>,
    buffers: Vec<BufferBuilder<T>>,
    metadata: Option<HashMap<String, String>>,
    count_step: usize,
}
impl<T> Arrow<T>
where
    T: ArrowNativeType,
{
    /// Creates a new Apache [Arrow](https://docs.rs/arrow) data logger
    ///
    ///  - `n_step`: the number of time step
    ///  - `names`: the names of the logged data
    ///  - `capacities`: the sizes of the logged data
    pub fn new<S: Into<String>>(n_step: usize, names: Vec<S>, capacities: Vec<usize>) -> Self {
        let buffers = capacities
            .iter()
            .map(|n| BufferBuilder::<T>::new(*n * n_step))
            .collect();
        Self {
            n_step,
            names: names.into_iter().map(|x| x.into()).collect(),
            capacities,
            buffers,
            metadata: None,
            count_step: 0,
        }
    }
}
impl<T> Display for Arrow<T>
where
    T: ArrowNativeType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Arrow logger:")?;
        writeln!(f, " - steps #: {}/{}", self.n_step, self.count_step)?;
        writeln!(f, " - data:")?;
        for (name, capacity) in self.names.iter().zip(self.capacities.iter()) {
            writeln!(f, "   - {:>8}:{:>4}", name, capacity)?;
        }
        Ok(())
    }
}
impl Arrow<f64> {
    /// Saves the data to a [Parquet](https://docs.rs/parquet) data file
    pub fn to_parquet<P: AsRef<Path> + std::fmt::Debug>(&mut self, path: P) -> Result<()> {
        let mut lists: Vec<Arc<dyn Array>> = vec![];
        for (buffer, n) in self.buffers.iter_mut().zip(self.capacities.iter()) {
            let data = ArrayData::builder(DataType::Float64)
                .len(buffer.len())
                .add_buffer(buffer.finish())
                .build()?;
            let offsets = (0..)
                .step_by(*n)
                .take(self.n_step + 1)
                .collect::<Vec<i32>>();
            let list = ArrayData::builder(DataType::List(Box::new(Field::new(
                "values",
                DataType::Float64,
                false,
            ))))
            .len(self.n_step)
            .add_buffer(Buffer::from(&offsets.to_byte_slice()))
            .add_child_data(data)
            .build()?;
            lists.push(Arc::new(ListArray::from(list)))
        }

        let fields: Vec<_> = self
            .names
            .iter()
            .map(|name| {
                Field::new(
                    name,
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
impl<T> super::Client for Arrow<T>
where
    T: ArrowNativeType,
{
    type I = Vec<T>;
    type O = ();
    fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
        log::debug!(
            "receive #{} inputs: {:?}",
            data.len(),
            data.iter().map(|x| x.len()).collect::<Vec<usize>>()
        );
        self.count_step += 1;
        self.buffers
            .iter_mut()
            .zip(data.into_iter())
            .for_each(|(buffer, data)| {
                buffer.append_slice(data.as_slice());
            });
        self
    }
}
