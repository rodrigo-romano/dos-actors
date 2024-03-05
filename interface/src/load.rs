use std::{
    env::{self, VarError},
    fs::File,
    io::{Read, Write},
    path::Path,
};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("load reader error")]
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

pub trait Codec
where
    Self: Sized + serde::ser::Serialize,
    for<'de> Self: serde::de::Deserialize<'de>,
{
    fn load<R>(reader: &mut R) -> Result<Self>
    where
        R: Read,
    {
        Ok(bincode::serde::decode_from_std_read(
            reader,
            bincode::config::standard(),
        )?)
    }

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

pub trait Load: Codec
where
    Self: Sized + serde::ser::Serialize,
    for<'de> Self: serde::de::Deserialize<'de>,
{
    type Builder;

    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::load(&mut File::open(path)?)
    }

    fn from_data_repo(file_name: impl AsRef<Path>) -> Result<Self> {
        let data_repo = env::var("DATA_REPO")?;
        let path = Path::new(&data_repo).join(file_name);
        Self::from_path(path)
    }

    fn to_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.save(&mut File::create(path)?)?;
        Ok(())
    }

    fn to_data_repo(&self, file_name: impl AsRef<Path>) -> Result<()> {
        let data_repo = env::var("DATA_REPO")?;
        let path = Path::new(&data_repo).join(file_name);
        Self::to_path(self, path)
    }

    fn from_path_or_else<P, F>(path: P, builder: F) -> Result<Self>
    where
        P: AsRef<Path>,
        F: FnOnce() -> Self::Builder,
        Self: TryFrom<<Self as Load>::Builder>,
        <Self as TryFrom<<Self as Load>::Builder>>::Error: std::fmt::Debug,
    {
        Self::from_path(path).or_else(|_| {
            Self::try_from(builder()).map_err(|e| LoadError::Builder(format!("{e:?}")))
        })
    }

    fn from_data_repo_or_else<P, F>(path: P, builder: F) -> Result<Self>
    where
        P: AsRef<Path>,
        F: FnOnce() -> Self::Builder,
        Self: TryFrom<<Self as Load>::Builder>,
        <Self as TryFrom<<Self as Load>::Builder>>::Error: std::fmt::Debug,
    {
        Self::from_data_repo(path).or_else(|_| {
            Self::try_from(builder()).map_err(|e| LoadError::Builder(format!("{e:?}")))
        })
    }

    fn from_path_or_default<P, F>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
        Self::Builder: Default,
        Self: TryFrom<<Self as Load>::Builder>,
        <Self as TryFrom<<Self as Load>::Builder>>::Error: std::fmt::Debug,
    {
        Self::from_path_or_else(path, Default::default)
    }

    fn from_data_repo_or_default<P, F>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
        Self::Builder: Default,
        Self: TryFrom<<Self as Load>::Builder>,
        <Self as TryFrom<<Self as Load>::Builder>>::Error: std::fmt::Debug,
    {
        Self::from_data_repo_or_else(path, Default::default)
    }
}
