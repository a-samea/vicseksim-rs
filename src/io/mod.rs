//! # IO Module - Data Persistence and Pipeline Management
//!
//! This module provides data persistence functionality for the flocking simulation pipeline.
//! It handles saving and loading of ensemble data, simulation results, and analysis outputs
//! to support the three-stage pipeline:
//!
//! 1. **Ensemble Generation** → Save to `./data/ensemble/`
//! 2. **Simulation Execution** → Save to `./data/simulation/`
//! 3. **Analysis Processing** → Save to `./data/analysis/` (future implementation)
//!
//! All data is stored in the `./data/` directory with organized subdirectories for each
//! pipeline stage.

pub mod analysis;
pub mod ensemble;
pub mod simulation;

use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub enum DataType {
    Ensemble,
    Simulation,
    Analysis,
}

/// Ensures the data directory structure exists
pub fn ensure_data_directories() -> Result<(), std::io::Error> {
    fs::create_dir_all("./data/ensemble")?;
    fs::create_dir_all("./data/simulation")?;
    fs::create_dir_all("./data/analysis")?;
    Ok(())
}

/// Common utility function to get file path for data with tag and ID
/// 
/// Constructs standardized file paths for data storage using the format:
/// `./data/{data_type}/{tag}-{id}.bin`
/// 
/// # Arguments
/// 
/// * `data_type` - The type of data (e.g., "ensemble", "simulation", "analysis")
/// * `tag` - The tag identifier for the data
/// * `id` - The unique numeric identifier
/// 
/// # Returns
/// 
/// A `PathBuf` pointing to the data file location
pub fn get_data_path(data_type: DataType, tag: &str, id: &usize) -> PathBuf {
    let parent = match data_type {
        DataType::Ensemble => "ensemble",
        DataType::Simulation => "simulation",
        DataType::Analysis => "analysis",
    };
    Path::new("./data").join(parent).join(format!("{}-{}.bin", tag, id))
}

/// Common utility function to save serializable data to disk
/// 
/// Serializes data using bincode and saves it to the specified path.
/// Creates parent directories if they don't exist.
/// 
/// # Arguments
/// 
/// * `data` - The data to serialize and save
/// * `file_path` - The path where to save the file
/// 
/// # Returns
/// 
/// * `Ok(())` - Successfully saved the data
/// * `Err(Box<dyn std::error::Error>)` - File system or serialization error
pub fn save_data<T: serde::Serialize>(data: &T, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);
    bincode::serialize_into(writer, data)?;

    Ok(())
}

/// Common utility function to load deserializable data from disk
/// 
/// Deserializes data from a binary file using bincode.
/// 
/// # Arguments
/// 
/// * `file_path` - The path to the file to load
/// 
/// # Returns
/// 
/// * `Ok(T)` - Successfully loaded and deserialized data
/// * `Err(Box<dyn std::error::Error>)` - File not found, IO error, or deserialization error
pub fn load_data<T: serde::de::DeserializeOwned>(file_path: &Path) -> Result<T, Box<dyn std::error::Error>> {
    if !file_path.exists() {
        return Err(format!("Data file not found: {}", file_path.display()).into());
    }

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let data = bincode::deserialize_from(reader)?;

    Ok(data)
}

/// Get current Unix timestamp
/// 
/// # Returns
/// 
/// Current Unix timestamp in seconds, or 0 if system time is before Unix epoch
pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
