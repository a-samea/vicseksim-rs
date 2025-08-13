//! # IO module for handling data persistence.

use std::path::{Path, PathBuf};

pub mod bin;
pub mod json;

#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Ensemble,
    Simulation,
    Analysis,
}

impl DataType {
    pub fn folder(&self) -> &'static str {
        match self {
            DataType::Ensemble => "ensemble",
            DataType::Simulation => "simulation",
            DataType::Analysis => "analysis",
        }
    }
}

pub trait DataPersistence: serde::Serialize + for<'de> serde::Deserialize<'de> {
    fn data_type() -> DataType;
    fn id(&self) -> usize;
    fn tag(&self) -> usize;

    fn binary_path(&self) -> PathBuf {
        Path::new("./data")
            .join(Self::data_type().folder())
            .join(format!("t{}-i{}.bin", self.tag(), self.id()))
    }

    fn json_path(&self) -> PathBuf {
        Path::new("./plots/data")
            .join(Self::data_type().folder())
            .join(format!("t{}-i{}.json", self.tag(), self.id()))
    }
}
