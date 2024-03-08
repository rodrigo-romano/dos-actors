//! # Serialization interface
//!
//! Interface to serialize and to deserialize `dos-actors` objects.

use std::{
    env::{self, VarError},
    fmt::Debug,
    fs::File,
    io::{Read, Write},
    path::Path,
};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("filing error")]
    IO(#[from] std::io::Error),
    #[error("decoder error")]
    Decoder(#[from] bincode::error::DecodeError),
    #[error("encoder error")]
    Encoder(#[from] bincode::error::EncodeError),
    #[error("builder error: {0}")]
    Builder(String),
    #[error("DATA_REPO not set")]
    DataRepo(#[from] VarError),
}

pub type Result<T> = std::result::Result<T, LoadError>;

/// Encoding and decoding
pub trait Codec
where
    Self: Sized + serde::ser::Serialize,
    for<'de> Self: serde::de::Deserialize<'de>,
{
    /// Decodes object from [std::io::Read]
    fn load<R>(reader: &mut R) -> Result<Self>
    where
        R: Read,
    {
        Ok(bincode::serde::decode_from_std_read(
            reader,
            bincode::config::standard(),
        )?)
    }

    /// Encodes object to [std::io::Write]
    fn save<W>(&self, writer: &mut W) -> Result<usize>
    where
        W: Write,
    {
        Ok(bincode::serde::encode_into_std_write(
            self,
            writer,
            bincode::config::standard(),
        )?)
    }
}

/// Encoding and decoding to/from [File]
pub trait Filing: Codec
where
    Self: Sized + serde::ser::Serialize,
    for<'de> Self: serde::de::Deserialize<'de>,
{
    /// Decodes object from given path
    fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + Debug,
    {
        log::info!("decoding from {path:?}");
        Self::load(&mut File::open(path)?)
    }

    /// Decodes object from given file
    /// The file is read from the directory specified by the `DATA_REPO` environment variable.
    fn from_data_repo(file_name: impl AsRef<Path>) -> Result<Self> {
        let data_repo = env::var("DATA_REPO")?;
        let path = Path::new(&data_repo).join(file_name);
        Self::from_path(path)
    }

    /// Encodes object from given path
    fn to_path<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path> + Debug,
    {
        log::info!("encoding to {path:?}");
        self.save(&mut File::create(path)?)?;
        Ok(())
    }

    /// Encodes object from given file.
    /// The file is written to the directory specified by the `DATA_REPO` environment variable.
    fn to_data_repo(&self, file_name: impl AsRef<Path>) -> Result<()> {
        let data_repo = env::var("DATA_REPO")?;
        let path = Path::new(&data_repo).join(file_name);
        Self::to_path(self, path)
    }

    /// Decodes object from given path or creates a new object from the builder and encodes the object to the given path
    fn from_path_or_else<P, F, B>(path: P, builder: F) -> Result<Self>
    where
        P: AsRef<Path> + Debug,
        F: FnOnce() -> B,
        Self: TryFrom<B>,
        <Self as TryFrom<B>>::Error: std::fmt::Debug,
    {
        Self::from_path(&path).or_else(|_| {
            let this =
                Self::try_from(builder()).map_err(|e| LoadError::Builder(format!("{e:?}")))?;
            this.to_path(path)?;
            Ok(this)
        })
    }

    /// Decodes object from given file or creates a new object from the builder and encodes the object to the given file.
    /// The file is read from the directory specified by the `DATA_REPO` environment variable.
    fn from_data_repo_or_else<P, F, B>(file_name: P, builder: F) -> Result<Self>
    where
        P: AsRef<Path>,
        F: FnOnce() -> B,
        Self: TryFrom<B>,
        <Self as TryFrom<B>>::Error: std::fmt::Debug,
    {
        let data_repo = env::var("DATA_REPO")?;
        let path = Path::new(&data_repo).join(file_name);
        Self::from_path_or_else(path, builder)
    }

    /// Decodes object from given path or creates a new object from [Default] and encodes the object to the given path
    fn from_path_or_default<P, F, B>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + Debug,
        B: Default,
        F: FnOnce() -> B,
        Self: TryFrom<B>,
        <Self as TryFrom<B>>::Error: std::fmt::Debug,
    {
        Self::from_path_or_else(path, Default::default)
    }

    /// Decodes object from given file or creates a new object from [Default] and encodes the object to the given file.
    /// The file is read from the directory specified by the `DATA_REPO` environment variable.
    fn from_data_repo_or_default<P, F, B>(file_name: P) -> Result<Self>
    where
        P: AsRef<Path>,
        B: Default,
        F: FnOnce() -> B,
        Self: TryFrom<B>,
        <Self as TryFrom<B>>::Error: std::fmt::Debug,
    {
        Self::from_data_repo_or_else(file_name, Default::default)
    }
}
