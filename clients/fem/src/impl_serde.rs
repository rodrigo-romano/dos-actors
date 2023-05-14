use std::{
    env,
    io::{BufReader, BufWriter},
};

use crate::{DiscreteModalSolver, Solver};

impl<T> DiscreteModalSolver<T>
where
    T: Solver + Default + serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    pub fn save<P>(&self, path: P) -> crate::Result<&Self>
    where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let path =
            std::path::Path::new(&env::var("DATA_REPO").unwrap_or_else(|_| String::from(".")))
                .join(&path);
        log::info!("saving FEM state space to {:?}", path);
        let file = std::fs::File::create(path)?;
        let mut buffer = BufWriter::new(file);
        bincode::serialize_into(&mut buffer, self)?;
        Ok(self)
    }
}

impl<T> TryFrom<String> for DiscreteModalSolver<T>
where
    T: Solver + Default + serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    type Error = crate::StateSpaceError;
    fn try_from(path: String) -> crate::Result<Self> {
        let path =
            std::path::Path::new(&env::var("DATA_REPO").unwrap_or_else(|_| String::from(".")))
                .join(&path);
        let file = std::fs::File::open(&path)?;
        log::info!("loading FEM state space from {:?}", path);
        let buffer = BufReader::new(file);
        let this: Self = bincode::deserialize_from(buffer)?;
        Ok(this)
    }
}

impl<T> TryFrom<&str> for DiscreteModalSolver<T>
where
    T: Solver + Default + serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    type Error = crate::StateSpaceError;
    fn try_from(path: &str) -> crate::Result<Self> {
        let path =
            std::path::Path::new(&env::var("DATA_REPO").unwrap_or_else(|_| String::from(".")))
                .join(&path);
        let file = std::fs::File::open(&path)?;
        log::info!("loading FEM state space from {:?}", path);
        let buffer = BufReader::new(file);
        let this: Self = bincode::deserialize_from(buffer)?;
        Ok(this)
    }
}

impl<T> TryFrom<std::path::PathBuf> for DiscreteModalSolver<T>
where
    T: Solver + Default + serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    type Error = crate::StateSpaceError;
    fn try_from(path: std::path::PathBuf) -> crate::Result<Self> {
        let path =
            std::path::Path::new(&env::var("DATA_REPO").unwrap_or_else(|_| String::from(".")))
                .join(&path);
        let file = std::fs::File::open(&path)?;
        log::info!("loading FEM state space from {:?}", path);
        let buffer = BufReader::new(file);
        let this: Self = bincode::deserialize_from(buffer)?;
        Ok(this)
    }
}
