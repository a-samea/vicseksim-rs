//! # Ensemble IO Module
//!
//! This module provides input/output functionality for ensemble data persistence and retrieval.
//! It handles the serialization, deserialization, and file management of ensemble collections
//! for the flocking simulation system.
//!
//! ## Overview
//!
//! The ensemble IO module serves as the bridge between in-memory ensemble data structures
//! and persistent storage. It supports:
//! - Binary serialization/deserialization of ensemble data using bincode
//! - Concurrent ensemble saving through receiver threads
//! - Directory discovery and ensemble enumeration
//! - Structured file naming and organization
//!
//! ## File Organization
//!
//! Ensemble files are stored in the `./data/ensemble/` directory with the naming convention:
//! ```
//! {tag}-{id}.bin
//! ```
//! Where:
//! - `tag`: A string identifier for the ensemble type or experiment
//! - `id`: A unique numeric identifier for the specific ensemble instance
//!
//! ## Usage Patterns
//!
//! ### Single Ensemble Operations
//! ```rust
//! use flocking_lib::io::ensemble::{save_ensemble, load_ensemble};
//! use flocking_lib::ensemble::EnsembleResult;
//!
//! // Save an ensemble
//! let ensemble: EnsembleResult = /* ... */;
//! save_ensemble(&ensemble)?;
//!
//! // Load an ensemble
//! let loaded = load_ensemble("test_tag", &42)?;
//! ```
//!
//! ### Batch Processing
//! ```rust
//! use flocking_lib::io::ensemble::{start_receiver_thread, list_ensemble_tags_and_ids};
//! use std::sync::mpsc;
//!
//! // Set up concurrent saving
//! let (tx, rx) = mpsc::channel();
//! let handle = start_receiver_thread(rx);
//!
//! // List all available ensembles
//! let ensembles = list_ensemble_tags_and_ids()?;
//! for (tag, id) in ensembles {
//!     println!("Found ensemble: {} (ID: {})", tag, id);
//! }
//! ```
//!
//! ## Integration Points
//!
//! - **Ensemble Module**: Provides `EnsembleResult` data structures for persistence
//! - **IO Module**: Integrates with directory management through `ensure_data_directories()`
//! - **CLI Interface**: Used for batch ensemble generation and analysis workflows
//! - **Simulation Module**: Loads ensembles as initial conditions for simulations
//!
//! ## Error Handling
//!
//! The module uses `Box<dyn std::error::Error>` for error propagation, allowing
//! flexible error handling across different failure modes:
//! - File system errors (directory creation, file access)
//! - Serialization/deserialization errors
//! - Data validation errors
//!
//! Functions may panic on corrupted data during deserialization, which is the
//! expected behavior for data integrity validation.

use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::ensemble::{EnsembleResult};




/// Gets the file path for an ensemble with the given tag and ID
/// 
/// This is an internal utility function that constructs the standardized file path
/// for ensemble binary files using the format: `./data/ensemble/{tag}-{id}.bin`
/// 
/// # Arguments
/// 
/// * `tag` - The tag identifier for the ensemble
/// * `id` - The unique numeric identifier for the ensemble
/// 
/// # Returns
/// 
/// A `PathBuf` pointing to the ensemble file location
fn get_ensemble_path(tag: &str, id: &usize) -> PathBuf {
    Path::new("./data/ensemble").join(format!("{}-{}.bin", tag, id))
}

/// Saves an EnsembleResult to a binary file on disk
///
/// This function serializes the ensemble data using bincode and saves it to the
/// standardized location. The parent directory is created if it doesn't exist.
/// 
/// # Arguments
///
/// * `ensemble` - The ensemble result to save containing birds, metadata, and parameters
/// 
/// # Returns
/// 
/// * `Ok(())` - Successfully saved the ensemble
/// * `Err(Box<dyn std::error::Error>)` - File system or serialization error
/// 
/// # Examples
/// 
/// ```rust
/// use flocking_lib::io::ensemble::save_ensemble;
/// 
/// let ensemble: EnsembleResult = /* ... */;
/// save_ensemble(&ensemble)?;
/// ```
pub fn save_ensemble(ensemble: &EnsembleResult) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = get_ensemble_path(&ensemble.tag, &ensemble.id);

    // Create parent directory if it doesn't exist
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(&file_path)?;
    let writer = BufWriter::new(file);

    bincode::serialize_into(writer, ensemble)?;

    Ok(())
}

/// Starts a background receiver thread for concurrent ensemble saving
/// 
/// This function spawns a dedicated thread that listens on an MPSC channel for
/// EnsembleResult data and automatically saves each ensemble to disk. It adds
/// timestamp metadata and provides progress feedback through console output.
/// 
/// The receiver thread will run until the channel is closed (all senders dropped).
/// This enables concurrent ensemble generation where multiple worker threads can
/// send completed ensembles for saving without blocking.
///
/// # Arguments
///
/// * `rx` - MPSC receiver channel for EnsembleResult data
///
/// # Returns
///
/// * A join handle for the spawned receiver thread that returns `Result<(), String>`
/// 
/// # Examples
/// 
/// ```rust
/// use flocking_lib::io::ensemble::start_receiver_thread;
/// use std::sync::mpsc;
/// 
/// let (tx, rx) = mpsc::channel();
/// let handle = start_receiver_thread(rx);
/// 
/// // Send ensembles from worker threads
/// // tx.send(ensemble)?;
/// 
/// // Wait for all saves to complete
/// handle.join().unwrap()?;
/// ```
pub fn start_receiver_thread(
    rx: mpsc::Receiver<EnsembleResult>,
) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        // Ensure ensemble directory exists
        crate::io::ensure_data_directories().map_err(|e| e.to_string())?;

        // Process each ensemble result as it arrives
        while let Ok(ensemble_result) = rx.recv() {
            // Add timestamp info
            let ensemble_with_metadata = EnsembleResult {
                created_at: SystemTime::now().duration_since(UNIX_EPOCH)
                    .map_err(|e| e.to_string())?.as_secs(),
                ..ensemble_result
            };

            // Save to file using the tag
            save_ensemble(&ensemble_with_metadata).map_err(|e| e.to_string())?;

            println!(
                "Ensemble '{}' (ID: {}) saved successfully with {} birds",
                ensemble_with_metadata.tag,
                ensemble_with_metadata.id,
                ensemble_with_metadata.birds.len()
            );
        }

        Ok(())
    })
}

/// Lists all ensemble files and extracts their tags and IDs
/// 
/// This function scans the `./data/ensemble/` directory for all `.bin` files,
/// parses their filenames to extract tag and ID information, and validates
/// each file by loading it. Only successfully loadable ensembles are included
/// in the results.
/// 
/// The function expects filenames in the format `{tag}-{id}.bin` and will skip
/// any files that don't match this pattern. Files that cannot be deserialized
/// will cause the function to panic (expected behavior for data validation).
///
/// # Returns
/// 
/// * `Ok(Vec<(String, usize)>)` - A vector of tuples containing (tag, id) for each valid ensemble
/// * `Err(Box<dyn std::error::Error>)` - Error if directory cannot be read
/// 
/// # Examples
/// 
/// ```rust
/// use flocking_lib::io::ensemble::list_ensemble_tags_and_ids;
/// 
/// let ensembles = list_ensemble_tags_and_ids()?;
/// for (tag, id) in ensembles {
///     println!("Found ensemble: {} (ID: {})", tag, id);
/// }
/// ```
/// 
/// # Panics
/// 
/// This function will panic if it encounters corrupted ensemble files that cannot
/// be deserialized. This is the expected behavior for data integrity validation.
pub fn list_ensemble_tags_and_ids() -> Result<Vec<(String, usize)>, Box<dyn std::error::Error>> {
    let ensemble_dir = Path::new("./data/ensemble");
    
    if !ensemble_dir.exists() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    
    for entry in fs::read_dir(ensemble_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Skip if not a .bin file
        if !path.extension().map_or(false, |ext| ext == "bin") {
            continue;
        }

        // Extract filename without extension
        let file_name = match path.file_stem().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => continue,
        };

        // Parse filename format: {tag}-{id}
        let dash_pos = match file_name.rfind('-') {
            Some(pos) => pos,
            None => continue,
        };

        let tag = file_name[..dash_pos].to_string();
        let id_str = &file_name[dash_pos + 1..];
        
        let id = match id_str.parse::<usize>() {
            Ok(id) => id,
            Err(_) => continue,
        };

        // Load the ensemble to verify it's valid and get the actual tag and id
        match load_ensemble(&tag, &id) {
            Ok(ensemble) => {
                results.push((ensemble.tag, ensemble.id));
            }
            Err(_) => {
                unreachable!("Failed to load ensemble")
            }
        }
    }
    
    Ok(results)
}

/// Loads ensemble data from a binary file
/// 
/// This function deserializes an EnsembleResult from disk using the standardized
/// file path format. It performs existence checks and handles file IO errors
/// gracefully while allowing deserialization errors to panic (expected behavior
/// for data integrity validation).
///
/// # Arguments
/// 
/// * `tag` - Tag name of the ensemble to load
/// * `id` - ID of the ensemble to load
/// 
/// # Returns
/// 
/// * `Ok(EnsembleResult)` - Successfully loaded and deserialized ensemble data
/// * `Err(Box<dyn std::error::Error>)` - File not found or IO error
/// 
/// # Examples
/// 
/// ```rust
/// use flocking_lib::io::ensemble::load_ensemble;
/// 
/// let ensemble = load_ensemble("test_ensemble", &42)?;
/// println!("Loaded ensemble with {} birds", ensemble.birds.len());
/// ```
/// 
/// # Panics
/// 
/// This function will panic if the file exists but contains corrupted data that
/// cannot be deserialized. This is the expected behavior for data integrity validation.
pub fn load_ensemble(tag: &str, id: &usize) -> Result<EnsembleResult, Box<dyn std::error::Error>> {
    let file_path = get_ensemble_path(tag, id);

    if !file_path.exists() {
        return Err(format!("Ensemble file not found: {}", file_path.display()).into());
    }

    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    // Try loading as EnsembleResult first (new format)
    Ok(bincode::deserialize_from::<_, EnsembleResult>(reader)?)
}


