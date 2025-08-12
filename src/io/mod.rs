use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

// list - load - exporttojson - binary

/// binary data persistence module
pub mod bin;

/// Data category for organizing files
#[derive(Debug, Clone, Copy)]
pub enum DataCategory {
    Ensemble,
    Simulation,
    Analysis,
}
impl DataCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataCategory::Ensemble => "ensemble",
            DataCategory::Simulation => "simulation",
            DataCategory::Analysis => "analysis",
        }
    }
}

/// Trait for data that can be persisted to disk with metadata
pub trait Persistable: serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Clone {
    /// Get the unique identifier for this data entry
    fn id(&self) -> usize;

    /// Get the tag/category for this data entry
    fn tag(&self) -> usize;

    /// Get the data type category for directory organization
    fn data_type(&self) -> DataCategory;

    /// Generate the file path for this data entry
    fn file_path(&self) -> PathBuf {
        Path::new("./data")
            .join(self.data_type().as_str())
            .join(format!("t{}-i{}.bin", self.tag(), self.id()))
    }
}

/// Trait for managing MPSC-based data channels
pub trait DataChannel<T: Persistable> {
    /// Start a receiver thread for handling data
    fn start_receiver_thread(rx: mpsc::Receiver<T>) -> thread::JoinHandle<Result<(), String>>;
}
