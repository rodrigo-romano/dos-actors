//! [Actor](crate::Actor)s [Client]s
//!
//! The module holds the trait [Client] which methods are called
//! by the [Actor](crate::Actor)s client that is passed to the
//! [Actor::run](crate::Actor::run) method
//!
//! A few clients are defined:
//!  - [Logging] that accumulates the data received by a [Terminator](crate::Terminator)
//! into a [Vec]tor
//!  - [Sampler] that moves the data unmodified from inputs to outputs, useful for rate transitions.
//!  - [Signals] that generates some predefined signals

#[cfg(feature = "windloads")]
pub mod windloads;

#[cfg(feature = "fem")]
pub mod fem;

#[cfg(feature = "mount-ctrl")]
pub mod mount;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("cannot open a parquet file")]
    ArrowToFile(#[from] std::io::Error),
    #[cfg(feature = "apache-arrow")]
    #[error("cannot build Arrow data")]
    ArrowError(#[from] arrow::error::ArrowError),
    #[cfg(feature = "apache-arrow")]
    #[error("cannot save data to Parquet")]
    ParquetError(#[from] parquet::errors::ParquetError),
}

/// Client method specifications
pub trait Client {
    //: std::fmt::Debug {
    type I;
    type O;
    /// Processes the [Actor](crate::Actor) [inputs](crate::Actor::inputs) for the client
    fn consume(&mut self, _data: Vec<&Self::I>) -> &mut Self {
        self
    }
    /// Generates the [outputs](crate::Actor::outputs) from the client
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        Default::default()
    }
    /// Updates the state of the client
    fn update(&mut self) -> &mut Self {
        self
    }
}

/// Simple data logging
///
/// Accumulates all the inputs in a single [Vec]
#[derive(Default, Debug)]
pub struct Logging<T>(Vec<T>);
impl<T> std::ops::Deref for Logging<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> Client for Logging<Vec<T>>
where
    T: std::fmt::Debug + Clone,
{
    type I = Vec<T>;
    type O = ();
    fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
        log::debug!(
            "receive #{} inputs: {:?}",
            data.len(),
            data.iter().map(|x| x.len()).collect::<Vec<usize>>()
        );
        self.0.push(data.into_iter().flatten().cloned().collect());
        self
    }
}

#[cfg(feature = "apache-arrow")]
pub mod arrow_client {
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
        pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
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

            let file = File::create(path)?;
            let props = WriterProperties::builder().build();
            let mut writer = ArrowWriter::try_new(file, Arc::clone(&schema), Some(props))?;
            writer.write(&batch)?;
            writer.close()?;
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
}

/// Sample-and-hold rate transionner
#[derive(Debug, Default)]
pub struct Sampler<T>(Vec<T>);
impl<T> Client for Sampler<T>
where
    T: std::fmt::Debug + Clone,
{
    type I = T;
    type O = T;
    fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
        self.0 = data.into_iter().cloned().collect();
        self
    }
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        Some(self.0.drain(..).collect())
    }
}

/// Signal types
#[derive(Debug, Clone)]
pub enum Signal {
    /// A constant signal
    Constant(f64),
    /// A sinusoidal signal
    Sinusoid {
        amplitude: f64,
        sampling_frequency_hz: f64,
        frequency_hz: f64,
        phase_s: f64,
    },
}
impl Signal {
    /// Returns the signal value at step `i`
    pub fn get(&self, i: usize) -> f64 {
        use Signal::*;
        match self {
            Constant(val) => *val,
            Sinusoid {
                amplitude,
                sampling_frequency_hz,
                frequency_hz,
                phase_s,
            } => {
                (2f64
                    * std::f64::consts::PI
                    * (phase_s + i as f64 * frequency_hz / sampling_frequency_hz))
                    .sin()
                    * amplitude
            }
        }
    }
}

/// Signals generator
#[derive(Debug, Default)]
pub struct Signals {
    outputs_size: Vec<usize>,
    signals: Vec<Vec<Signal>>,
    pub step: usize,
    pub n_step: usize,
}
impl Signals {
    /// Create new signals
    pub fn new(outputs_size: Vec<usize>, n_step: usize) -> Self {
        let signal: Vec<_> = outputs_size
            .iter()
            .map(|&n| vec![Signal::Constant(0f64); n])
            .collect();
        Self {
            outputs_size,
            signals: signal,
            step: 0,
            n_step,
        }
    }
    /// Sets the type of signals
    pub fn signals(self, signal: Signal) -> Self {
        let signal: Vec<_> = self
            .outputs_size
            .iter()
            .map(|&n| vec![signal.clone(); n])
            .collect();
        Self {
            signals: signal,
            ..self
        }
    }
    /// Sets the type of signals of one output
    pub fn output_signals(self, output: usize, output_signals: Signal) -> Self {
        let mut signals = self.signals;
        signals[output] = vec![output_signals; self.outputs_size[output]];
        Self { signals, ..self }
    }
    /// Sets the type of signals of one output index
    pub fn output_signal(self, output: usize, output_i: usize, signal: Signal) -> Self {
        let mut signals = self.signals;
        signals[output][output_i] = signal;
        Self { signals, ..self }
    }
}

impl Client for Signals {
    type I = ();
    type O = Vec<f64>;
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        if self.step < self.n_step {
            let i = self.step;
            self.step += 1;
            Some(
                self.signals
                    .iter()
                    .map(|signals| {
                        signals
                            .iter()
                            .map(|signal| signal.get(i))
                            .collect::<Vec<_>>()
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
}
